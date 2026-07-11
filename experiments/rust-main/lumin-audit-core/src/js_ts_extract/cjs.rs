use super::{
    assignment_target_identifier_name, assignment_target_member_object_identifier_name,
    expression_identifier_name, line_for_span, member_object_identifier_name, property_key_name,
    simple_assignment_target_identifier_name,
    simple_assignment_target_member_object_identifier_name, static_member_property_name,
    CjsExportExactRecord, CjsExportOpaqueRecord, CjsExportSurface, CjsRequireOpacityRecord,
    UseRecord,
};
use oxc_ast::ast::{
    Argument, AssignmentExpression, BindingIdentifier, BindingPattern, CallExpression,
    ChainElement, Expression, ExpressionStatement, FormalParameter, IdentifierReference,
    MemberExpression, ObjectExpression, ObjectPattern, ObjectPropertyKind, Program, PropertyKey,
    SimpleAssignmentTarget, Statement, UpdateExpression, VariableDeclarationKind,
    VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use oxc_syntax::operator::AssignmentOperator;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
fn expression_call_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a CallExpression<'a>> {
    match expression {
        Expression::CallExpression(call) => Some(call),
        Expression::ParenthesizedExpression(expression) => {
            expression_call_expression(&expression.expression)
        }
        Expression::ChainExpression(expression) => match &expression.expression {
            ChainElement::CallExpression(call) => Some(call),
            ChainElement::TSNonNullExpression(expression) => {
                expression_call_expression(&expression.expression)
            }
            _ => None,
        },
        Expression::TSAsExpression(expression) => {
            expression_call_expression(&expression.expression)
        }
        Expression::TSSatisfiesExpression(expression) => {
            expression_call_expression(&expression.expression)
        }
        Expression::TSNonNullExpression(expression) => {
            expression_call_expression(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => {
            expression_call_expression(&expression.expression)
        }
        Expression::TSInstantiationExpression(expression) => {
            expression_call_expression(&expression.expression)
        }
        _ => None,
    }
}

fn is_require_call(call: &CallExpression<'_>) -> bool {
    matches!(&call.callee, Expression::Identifier(identifier) if identifier.name == "require")
}

fn literal_require_source(call: &CallExpression<'_>) -> Option<String> {
    if !is_require_call(call) {
        return None;
    }
    match call.arguments.first() {
        Some(Argument::StringLiteral(literal)) => Some(literal.value.to_string()),
        _ => None,
    }
}

fn literal_argument_string(argument: Option<&Argument<'_>>) -> Option<String> {
    match argument {
        Some(Argument::StringLiteral(literal)) => Some(literal.value.to_string()),
        _ => None,
    }
}

fn is_json_path_fragment(value: Option<String>) -> bool {
    value
        .map(|value| {
            value
                .replace('\\', "/")
                .to_ascii_lowercase()
                .ends_with(".json")
        })
        .unwrap_or(false)
}

fn is_path_join_or_resolve_call(call: &CallExpression<'_>) -> bool {
    let Some(member) = call.callee.as_member_expression() else {
        return false;
    };
    if !matches!(
        static_member_property_name(member).as_deref(),
        Some("join" | "resolve")
    ) {
        return false;
    }
    matches!(
        member_object_identifier_name(member).as_deref(),
        Some("path")
    )
}

fn is_static_json_require_argument(argument: Option<&Argument<'_>>) -> bool {
    if is_json_path_fragment(literal_argument_string(argument)) {
        return true;
    }
    let Some(Argument::CallExpression(call)) = argument else {
        return false;
    };
    is_path_join_or_resolve_call(call)
        && is_json_path_fragment(literal_argument_string(call.arguments.last()))
}

fn member_object_call_expression<'a>(
    member: &'a MemberExpression<'a>,
) -> Option<&'a CallExpression<'a>> {
    match member {
        MemberExpression::StaticMemberExpression(member) => {
            expression_call_expression(&member.object)
        }
        MemberExpression::ComputedMemberExpression(member) => {
            expression_call_expression(&member.object)
        }
        MemberExpression::PrivateFieldExpression(member) => {
            expression_call_expression(&member.object)
        }
    }
}

fn assignment_target_is_module_exports(target: &oxc_ast::ast::AssignmentTarget<'_>) -> bool {
    let Some(member) = target
        .as_simple_assignment_target()
        .and_then(SimpleAssignmentTarget::as_member_expression)
    else {
        return false;
    };
    let object_name = member_object_identifier_name(member);
    matches!(object_name.as_deref(), Some("exports"))
        || (matches!(object_name.as_deref(), Some("module"))
            && matches!(
                static_member_property_name(member).as_deref(),
                Some("exports")
            ))
}

pub(super) fn collect_cjs_export_surface(
    program: &Program<'_>,
    line_starts: &[usize],
) -> Option<CjsExportSurface> {
    let mut surface = CjsExportSurface {
        exact: Vec::new(),
        opaque: Vec::new(),
    };

    for statement in &program.body {
        let Statement::ExpressionStatement(statement) = statement else {
            continue;
        };
        let Expression::AssignmentExpression(assignment) = &statement.expression else {
            continue;
        };
        if assignment.operator != AssignmentOperator::Assign {
            continue;
        }
        collect_cjs_export_assignment(assignment, &mut surface, line_starts);
    }

    surface.exact.sort_by(|left, right| {
        (&left.name, left.kind, left.line).cmp(&(&right.name, right.kind, right.line))
    });
    surface
        .opaque
        .sort_by(|left, right| (left.kind, left.line).cmp(&(right.kind, right.line)));

    (!surface.exact.is_empty() || !surface.opaque.is_empty()).then_some(surface)
}

fn collect_cjs_export_assignment(
    assignment: &AssignmentExpression<'_>,
    surface: &mut CjsExportSurface,
    line_starts: &[usize],
) {
    if let Some(member) = cjs_export_member_assignment(&assignment.left) {
        if let Some(name) = member.name {
            surface.exact.push(CjsExportExactRecord {
                name,
                kind: member.kind,
                line: line_for_span(line_starts, assignment.left.span()),
            });
        } else {
            surface.opaque.push(CjsExportOpaqueRecord {
                kind: "computed-export-name",
                line: line_for_span(line_starts, assignment.left.span()),
            });
        }
        return;
    }

    if !assignment_target_is_exact_module_exports(&assignment.left) {
        return;
    }
    let Expression::ObjectExpression(object) = &assignment.right else {
        surface.opaque.push(CjsExportOpaqueRecord {
            kind: "module-exports-assignment",
            line: line_for_span(line_starts, assignment.left.span()),
        });
        return;
    };
    collect_module_exports_object_properties(object, surface, line_starts);
}

struct CjsExportMember {
    name: Option<String>,
    kind: &'static str,
}

fn cjs_export_member_assignment(
    target: &oxc_ast::ast::AssignmentTarget<'_>,
) -> Option<CjsExportMember> {
    let member = target
        .as_simple_assignment_target()
        .and_then(SimpleAssignmentTarget::as_member_expression)?;
    let object_name = member_object_identifier_name(member);
    if matches!(object_name.as_deref(), Some("exports")) {
        return Some(CjsExportMember {
            name: static_member_property_name(member),
            kind: "exports-member",
        });
    }
    if member_object_is_exact_module_exports(member) {
        return Some(CjsExportMember {
            name: static_member_property_name(member),
            kind: "module-exports-member",
        });
    }
    None
}

fn assignment_target_is_exact_module_exports(target: &oxc_ast::ast::AssignmentTarget<'_>) -> bool {
    target
        .as_simple_assignment_target()
        .and_then(SimpleAssignmentTarget::as_member_expression)
        .is_some_and(is_exact_module_exports_member)
}

fn member_object_is_exact_module_exports(member: &MemberExpression<'_>) -> bool {
    match member {
        MemberExpression::StaticMemberExpression(member) => {
            expression_is_exact_module_exports(&member.object)
        }
        MemberExpression::ComputedMemberExpression(member) => {
            expression_is_exact_module_exports(&member.object)
        }
        MemberExpression::PrivateFieldExpression(member) => {
            expression_is_exact_module_exports(&member.object)
        }
    }
}

fn expression_is_exact_module_exports(expression: &Expression<'_>) -> bool {
    expression
        .as_member_expression()
        .is_some_and(is_exact_module_exports_member)
}

fn is_exact_module_exports_member(member: &MemberExpression<'_>) -> bool {
    matches!(
        member_object_identifier_name(member).as_deref(),
        Some("module")
    ) && matches!(
        static_member_property_name(member).as_deref(),
        Some("exports")
    ) && !matches!(member, MemberExpression::ComputedMemberExpression(_))
}

fn collect_module_exports_object_properties(
    object: &ObjectExpression<'_>,
    surface: &mut CjsExportSurface,
    line_starts: &[usize],
) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            surface.opaque.push(CjsExportOpaqueRecord {
                kind: "module-exports-object-opaque",
                line: line_for_span(line_starts, property.span()),
            });
            continue;
        };
        if let Some(name) = cjs_object_property_name(&property.key, property.computed) {
            surface.exact.push(CjsExportExactRecord {
                name,
                kind: "module-exports-object",
                line: line_for_span(line_starts, property.span),
            });
        } else {
            surface.opaque.push(CjsExportOpaqueRecord {
                kind: "computed-export-name",
                line: line_for_span(line_starts, property.span),
            });
        }
    }
}

fn cjs_object_property_name(key: &PropertyKey<'_>, computed: bool) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) if !computed => Some(identifier.name.to_string()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.to_string()),
        PropertyKey::Identifier(identifier) if !computed => Some(identifier.name.to_string()),
        _ => None,
    }
}

pub(super) struct CjsRequireFacts {
    pub(super) uses: Vec<UseRecord>,
    pub(super) opacity: Vec<CjsRequireOpacityRecord>,
}

#[derive(Debug)]
struct CjsRequireRecord {
    from_spec: String,
    local_name: String,
    line: usize,
    members: Vec<(String, usize)>,
    degraded: bool,
}

pub(super) fn collect_cjs_require_uses(
    program: &Program<'_>,
    line_starts: &[usize],
) -> CjsRequireFacts {
    let mut visitor = CjsRequireVisitor {
        line_starts,
        scopes: vec![BTreeMap::new()],
        records: Vec::new(),
        handled_require_starts: BTreeSet::new(),
        uses: Vec::new(),
        opacity: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.finish()
}

struct CjsRequireVisitor<'a> {
    line_starts: &'a [usize],
    scopes: Vec<BTreeMap<String, Option<usize>>>,
    records: Vec<CjsRequireRecord>,
    handled_require_starts: BTreeSet<u32>,
    uses: Vec<UseRecord>,
    opacity: Vec<CjsRequireOpacityRecord>,
}

impl CjsRequireVisitor<'_> {
    fn finish(mut self) -> CjsRequireFacts {
        for record in self.records {
            if !record.members.is_empty() && !record.degraded {
                for (member_name, line) in record.members {
                    self.uses.push(UseRecord {
                        from_spec: record.from_spec.clone(),
                        name: member_name,
                        member_name: None,
                        kind: "cjs-namespace-member".to_string(),
                        type_only: false,
                        line,
                        local_name: Some(record.local_name.clone()),
                        degraded: false,
                        resolved_file: None,
                        resolver_stage: None,
                    });
                }
            } else if record.degraded {
                self.uses.push(UseRecord {
                    from_spec: record.from_spec,
                    name: "*".to_string(),
                    member_name: None,
                    kind: "cjs-namespace-escape".to_string(),
                    type_only: false,
                    line: record.line,
                    local_name: Some(record.local_name),
                    degraded: true,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
        }

        CjsRequireFacts {
            uses: self.uses,
            opacity: self.opacity,
        }
    }

    fn current_scope_mut(&mut self) -> &mut BTreeMap<String, Option<usize>> {
        if self.scopes.is_empty() {
            self.scopes.push(BTreeMap::new());
        }
        let index = self.scopes.len() - 1;
        &mut self.scopes[index]
    }

    fn bind_local(&mut self, name: &str) {
        self.current_scope_mut().insert(name.to_string(), None);
    }

    fn bind_cjs(&mut self, name: &str, index: usize) {
        self.current_scope_mut()
            .insert(name.to_string(), Some(index));
    }

    fn resolve_cjs(&self, name: &str) -> Option<usize> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied().flatten())
    }

    fn record_member(&mut self, local_name: &str, member_name: String, line: usize) {
        if let Some(index) = self.resolve_cjs(local_name) {
            self.records[index].members.push((member_name, line));
        }
    }

    fn degrade(&mut self, local_name: &str) {
        if let Some(index) = self.resolve_cjs(local_name) {
            self.records[index].degraded = true;
        }
    }

    fn push_fallback(&mut self, require: &CallExpression<'_>, from_spec: String, kind: &str) {
        if self.handled_require_starts.contains(&require.span.start) {
            return;
        }
        self.handled_require_starts.insert(require.span.start);
        let degraded = kind != "cjs-side-effect-only";
        self.uses.push(UseRecord {
            from_spec,
            name: "*".to_string(),
            member_name: None,
            kind: kind.to_string(),
            type_only: false,
            line: line_for_span(self.line_starts, require.span),
            local_name: None,
            degraded,
            resolved_file: None,
            resolver_stage: None,
        });
    }

    fn push_opacity(&mut self, require: &CallExpression<'_>) {
        if self.handled_require_starts.contains(&require.span.start)
            || is_static_json_require_argument(require.arguments.first())
        {
            return;
        }
        self.handled_require_starts.insert(require.span.start);
        self.opacity.push(CjsRequireOpacityRecord {
            line: line_for_span(self.line_starts, require.span),
            kind: "dynamic-require",
        });
    }

    fn collect_object_pattern_members(
        &mut self,
        pattern: &ObjectPattern<'_>,
        from_spec: &str,
        line: usize,
    ) {
        let mut degraded = pattern.rest.is_some();
        for property in &pattern.properties {
            if let Some(name) = property_key_name(&property.key, property.computed) {
                self.uses.push(UseRecord {
                    from_spec: from_spec.to_string(),
                    name,
                    member_name: None,
                    kind: "cjs-require-exact".to_string(),
                    type_only: false,
                    line,
                    local_name: None,
                    degraded: false,
                    resolved_file: None,
                    resolver_stage: None,
                });
            } else {
                degraded = true;
            }
        }
        if degraded {
            self.uses.push(UseRecord {
                from_spec: from_spec.to_string(),
                name: "*".to_string(),
                member_name: None,
                kind: "cjs-namespace-escape".to_string(),
                type_only: false,
                line,
                local_name: None,
                degraded: true,
                resolved_file: None,
                resolver_stage: None,
            });
        }
    }
}

impl<'a> Visit<'a> for CjsRequireVisitor<'_> {
    fn enter_scope(
        &mut self,
        _flags: oxc_syntax::scope::ScopeFlags,
        _scope_id: &Cell<Option<oxc_syntax::scope::ScopeId>>,
    ) {
        self.scopes.push(BTreeMap::new());
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
        if self.scopes.is_empty() {
            self.scopes.push(BTreeMap::new());
        }
    }

    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        self.bind_local(it.name.as_str());
    }

    fn visit_formal_parameter(&mut self, it: &FormalParameter<'a>) {
        self.visit_binding_pattern(&it.pattern);
    }

    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        if let Some(init) = &it.init {
            if let Some(require) = expression_call_expression(init) {
                if let Some(from_spec) = literal_require_source(require) {
                    self.handled_require_starts.insert(require.span.start);
                    match &it.id {
                        BindingPattern::BindingIdentifier(identifier)
                            if it.kind == VariableDeclarationKind::Const =>
                        {
                            let local_name = identifier.name.as_str();
                            let index = self.records.len();
                            self.records.push(CjsRequireRecord {
                                from_spec,
                                local_name: local_name.to_string(),
                                line: line_for_span(self.line_starts, require.span),
                                members: Vec::new(),
                                degraded: false,
                            });
                            self.bind_cjs(local_name, index);
                        }
                        BindingPattern::BindingIdentifier(identifier) => {
                            let local_name = identifier.name.as_str();
                            self.uses.push(UseRecord {
                                from_spec,
                                name: "*".to_string(),
                                member_name: None,
                                kind: "cjs-namespace-escape".to_string(),
                                type_only: false,
                                line: line_for_span(self.line_starts, require.span),
                                local_name: Some(local_name.to_string()),
                                degraded: true,
                                resolved_file: None,
                                resolver_stage: None,
                            });
                            self.bind_local(local_name);
                        }
                        BindingPattern::ObjectPattern(pattern) => {
                            self.collect_object_pattern_members(
                                pattern,
                                &from_spec,
                                line_for_span(self.line_starts, require.span),
                            );
                            self.visit_binding_pattern(&it.id);
                        }
                        _ => {
                            self.uses.push(UseRecord {
                                from_spec,
                                name: "*".to_string(),
                                member_name: None,
                                kind: "cjs-namespace-escape".to_string(),
                                type_only: false,
                                line: line_for_span(self.line_starts, require.span),
                                local_name: None,
                                degraded: true,
                                resolved_file: None,
                                resolver_stage: None,
                            });
                            self.visit_binding_pattern(&it.id);
                        }
                    }
                    return;
                }
                if is_require_call(require) {
                    self.push_opacity(require);
                    self.visit_binding_pattern(&it.id);
                    return;
                }
            }

            if let Some(local_name) = expression_identifier_name(init) {
                if let Some(index) = self.resolve_cjs(&local_name) {
                    if let BindingPattern::ObjectPattern(pattern) = &it.id {
                        let from_spec = self.records[index].from_spec.clone();
                        let line = self.records[index].line;
                        self.collect_object_pattern_members(pattern, &from_spec, line);
                        self.visit_binding_pattern(&it.id);
                        return;
                    }
                }
            }
        }

        self.visit_binding_pattern(&it.id);
        if let Some(init) = &it.init {
            self.visit_expression(init);
        }
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        self.degrade(it.name.as_str());
    }

    fn visit_member_expression(&mut self, it: &MemberExpression<'a>) {
        if let Some(require) = member_object_call_expression(it) {
            if let Some(from_spec) = literal_require_source(require) {
                self.handled_require_starts.insert(require.span.start);
                let line = line_for_span(self.line_starts, it.span());
                if let Some(name) = static_member_property_name(it) {
                    self.uses.push(UseRecord {
                        from_spec,
                        name,
                        member_name: None,
                        kind: "cjs-namespace-member".to_string(),
                        type_only: false,
                        line,
                        local_name: None,
                        degraded: false,
                        resolved_file: None,
                        resolver_stage: None,
                    });
                } else {
                    self.uses.push(UseRecord {
                        from_spec,
                        name: "*".to_string(),
                        member_name: None,
                        kind: "cjs-namespace-escape".to_string(),
                        type_only: false,
                        line,
                        local_name: None,
                        degraded: true,
                        resolved_file: None,
                        resolver_stage: None,
                    });
                }
                return;
            }
        }
        if let Some(local_name) = member_object_identifier_name(it) {
            if let Some(member_name) = static_member_property_name(it) {
                let line = line_for_span(self.line_starts, it.span());
                self.record_member(&local_name, member_name, line);
            } else {
                self.degrade(&local_name);
            }
            return;
        }
        walk::walk_member_expression(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        if let Some(require) = expression_call_expression(&it.right) {
            if let Some(from_spec) = literal_require_source(require) {
                if assignment_target_is_module_exports(&it.left) {
                    self.handled_require_starts.insert(require.span.start);
                    self.uses.push(UseRecord {
                        from_spec,
                        name: "*".to_string(),
                        member_name: None,
                        kind: "cjs-reexport-broad".to_string(),
                        type_only: false,
                        line: line_for_span(self.line_starts, require.span),
                        local_name: None,
                        degraded: true,
                        resolved_file: None,
                        resolver_stage: None,
                    });
                    return;
                }
            } else if is_require_call(require) {
                self.push_opacity(require);
                return;
            }
        }
        if let Some(local_name) = assignment_target_identifier_name(&it.left) {
            self.degrade(&local_name);
            self.visit_expression(&it.right);
            return;
        }
        if let Some(local_name) = assignment_target_member_object_identifier_name(&it.left) {
            self.degrade(&local_name);
            self.visit_expression(&it.right);
            return;
        }
        walk::walk_assignment_expression(self, it);
    }

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        if let Some(local_name) = simple_assignment_target_identifier_name(&it.argument) {
            self.degrade(&local_name);
            return;
        }
        if let Some(local_name) =
            simple_assignment_target_member_object_identifier_name(&it.argument)
        {
            self.degrade(&local_name);
            return;
        }
        walk::walk_update_expression(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(member) = it.callee.as_member_expression() {
            if let Some(local_name) = member_object_identifier_name(member) {
                if let Some(member_name) = static_member_property_name(member) {
                    let line = line_for_span(self.line_starts, member.span());
                    self.record_member(&local_name, member_name, line);
                } else {
                    self.degrade(&local_name);
                    walk::walk_member_expression(self, member);
                }
                for argument in &it.arguments {
                    self.visit_argument(argument);
                }
                return;
            }
        }

        if let Some(from_spec) = literal_require_source(it) {
            self.push_fallback(it, from_spec, "cjs-namespace-escape");
            return;
        }
        if is_require_call(it) {
            self.push_opacity(it);
            return;
        }
        walk::walk_call_expression(self, it);
    }

    fn visit_expression_statement(&mut self, it: &ExpressionStatement<'a>) {
        if let Some(require) = expression_call_expression(&it.expression) {
            if let Some(from_spec) = literal_require_source(require) {
                self.push_fallback(require, from_spec, "cjs-side-effect-only");
                return;
            }
        }
        walk::walk_expression_statement(self, it);
    }
}
