use super::{
    assignment_target_identifier_name, assignment_target_member_object_identifier_name,
    binding_identifier_name_ref, line_for_span, member_object_identifier_name,
    simple_assignment_target_identifier_name,
    simple_assignment_target_member_object_identifier_name, static_member_property_name,
    DynamicImportOpacityRecord, UseRecord,
};
use oxc_ast::ast::{
    Argument, AssignmentExpression, BindingIdentifier, CallExpression, Expression, FormalParameter,
    IdentifierReference, ImportExpression, MemberExpression, Program, TemplateLiteral,
    UpdateExpression, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn collect_import_meta_glob_uses(
    program: &Program<'_>,
    line_starts: &[usize],
) -> Vec<UseRecord> {
    let mut visitor = ImportMetaGlobVisitor {
        line_starts,
        uses: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.uses
}

struct ImportMetaGlobVisitor<'a> {
    line_starts: &'a [usize],
    uses: Vec<UseRecord>,
}

impl<'a> Visit<'a> for ImportMetaGlobVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if is_import_meta_glob_call(it) {
            self.uses.push(UseRecord {
                from_spec: import_meta_glob_pattern(it)
                    .unwrap_or_else(|| "import.meta.glob(<nonliteral>)".to_string()),
                name: "*".to_string(),
                member_name: None,
                kind: "import-meta-glob".to_string(),
                type_only: false,
                line: line_for_span(self.line_starts, it.span),
                local_name: None,
                degraded: true,
                resolved_file: None,
                resolver_stage: Some("import-meta-glob".to_string()),
            });
        }
        walk::walk_call_expression(self, it);
    }
}

fn is_import_meta_glob_call(call: &CallExpression<'_>) -> bool {
    let Some(member) = call.callee.as_member_expression() else {
        return false;
    };
    if member.static_property_name() != Some("glob") {
        return false;
    }
    matches!(
        member.object(),
        Expression::MetaProperty(meta) if meta.meta.name == "import" && meta.property.name == "meta"
    )
}

fn import_meta_glob_pattern(call: &CallExpression<'_>) -> Option<String> {
    match call.arguments.first() {
        Some(Argument::StringLiteral(literal)) => Some(literal.value.to_string()),
        _ => None,
    }
}

pub(super) struct DynamicImportFacts {
    pub(super) uses: Vec<UseRecord>,
    pub(super) opacity: Vec<DynamicImportOpacityRecord>,
}

#[derive(Debug)]
struct DynamicImportRecord {
    from_spec: String,
    local_name: String,
    line: usize,
    members: Vec<(String, usize)>,
    degraded: bool,
}

pub(super) fn collect_dynamic_import_uses(
    program: &Program<'_>,
    line_starts: &[usize],
) -> DynamicImportFacts {
    let mut visitor = DynamicImportVisitor {
        line_starts,
        scopes: vec![BTreeMap::new()],
        records: Vec::new(),
        handled_import_starts: BTreeSet::new(),
        uses: Vec::new(),
        opacity: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.finish()
}

struct DynamicImportVisitor<'a> {
    line_starts: &'a [usize],
    scopes: Vec<BTreeMap<String, Option<usize>>>,
    records: Vec<DynamicImportRecord>,
    handled_import_starts: BTreeSet<u32>,
    uses: Vec<UseRecord>,
    opacity: Vec<DynamicImportOpacityRecord>,
}

impl DynamicImportVisitor<'_> {
    fn finish(mut self) -> DynamicImportFacts {
        for record in self.records {
            if !record.members.is_empty() && !record.degraded {
                for (member_name, line) in record.members {
                    self.uses.push(UseRecord {
                        from_spec: record.from_spec.clone(),
                        name: member_name,
                        member_name: None,
                        kind: "dynamic-member".to_string(),
                        type_only: false,
                        line,
                        local_name: Some(record.local_name.clone()),
                        degraded: false,
                        resolved_file: None,
                        resolver_stage: None,
                    });
                }
            } else {
                self.uses.push(UseRecord {
                    from_spec: record.from_spec,
                    name: "*".to_string(),
                    member_name: None,
                    kind: "dynamic".to_string(),
                    type_only: false,
                    line: record.line,
                    local_name: Some(record.local_name),
                    degraded: true,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
        }

        DynamicImportFacts {
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

    fn bind_dynamic(&mut self, name: &str, index: usize) {
        self.current_scope_mut()
            .insert(name.to_string(), Some(index));
    }

    fn resolve_dynamic(&self, name: &str) -> Option<usize> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied().flatten())
    }

    fn record_member(&mut self, local_name: &str, member_name: String, line: usize) {
        if let Some(index) = self.resolve_dynamic(local_name) {
            self.records[index].members.push((member_name, line));
        }
    }

    fn degrade(&mut self, local_name: &str) {
        if let Some(index) = self.resolve_dynamic(local_name) {
            self.records[index].degraded = true;
        }
    }

    fn push_dynamic_fallback(&mut self, import: &ImportExpression<'_>, from_spec: String) {
        if self.handled_import_starts.contains(&import.span.start) {
            return;
        }
        self.handled_import_starts.insert(import.span.start);
        self.uses.push(UseRecord {
            from_spec,
            name: "*".to_string(),
            member_name: None,
            kind: "dynamic".to_string(),
            type_only: false,
            line: line_for_span(self.line_starts, import.span),
            local_name: None,
            degraded: true,
            resolved_file: None,
            resolver_stage: None,
        });
    }

    fn push_opacity(&mut self, import: &ImportExpression<'_>) {
        if self.handled_import_starts.contains(&import.span.start) {
            return;
        }
        self.handled_import_starts.insert(import.span.start);
        self.opacity
            .push(dynamic_import_opacity_record(import, self.line_starts));
    }
}

impl<'a> Visit<'a> for DynamicImportVisitor<'_> {
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
        let local_name = binding_identifier_name_ref(&it.id);
        let import = it
            .init
            .as_ref()
            .map(unwrap_await_expression)
            .and_then(expression_import_expression);
        if let (Some(local_name), Some(import), Some(from_spec)) =
            (local_name, import, import.and_then(dynamic_import_source))
        {
            let index = self.records.len();
            self.records.push(DynamicImportRecord {
                from_spec,
                local_name: local_name.to_string(),
                line: line_for_span(self.line_starts, import.span),
                members: Vec::new(),
                degraded: false,
            });
            self.handled_import_starts.insert(import.span.start);
            self.bind_dynamic(local_name, index);
            return;
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
        if let Some(local_name) = member_object_identifier_name(it) {
            self.degrade(&local_name);
            return;
        }
        walk::walk_member_expression(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
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
        walk::walk_call_expression(self, it);
    }

    fn visit_import_expression(&mut self, it: &ImportExpression<'a>) {
        if let Some(from_spec) = dynamic_import_source(it) {
            self.push_dynamic_fallback(it, from_spec);
        } else {
            self.push_opacity(it);
            walk::walk_import_expression(self, it);
        }
    }
}

fn unwrap_await_expression<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::AwaitExpression(await_expression) => &await_expression.argument,
        _ => expression,
    }
}

fn expression_import_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ImportExpression<'a>> {
    match expression {
        Expression::ImportExpression(import) => Some(import),
        _ => None,
    }
}

fn dynamic_import_source(import: &ImportExpression<'_>) -> Option<String> {
    match &import.source {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        _ => None,
    }
}

fn dynamic_import_opacity_record(
    import: &ImportExpression<'_>,
    line_starts: &[usize],
) -> DynamicImportOpacityRecord {
    if let Expression::TemplateLiteral(template) = &import.source {
        if let Some(prefix) = dynamic_import_template_prefix(template) {
            return DynamicImportOpacityRecord {
                line: line_for_span(line_starts, import.span),
                kind: "template-prefix".to_string(),
                prefix: Some(prefix),
            };
        }
    }

    DynamicImportOpacityRecord {
        line: line_for_span(line_starts, import.span),
        kind: "nonliteral".to_string(),
        prefix: None,
    }
}

fn dynamic_import_template_prefix(template: &TemplateLiteral<'_>) -> Option<String> {
    if template.expressions.is_empty() {
        return None;
    }
    let prefix = template.quasis.first()?.value.cooked.as_ref()?.as_str();
    let relative = prefix.starts_with("./") || prefix.starts_with("../");
    let has_body = prefix.get(2..).is_some_and(|tail| !tail.is_empty());
    let has_trailing_separator = prefix.ends_with('/') || prefix.ends_with('\\');
    (relative && has_body && has_trailing_separator).then(|| prefix.to_string())
}
