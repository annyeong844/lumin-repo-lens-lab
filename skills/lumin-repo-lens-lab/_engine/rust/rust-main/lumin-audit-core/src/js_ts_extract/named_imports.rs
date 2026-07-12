use super::{
    assignment_target_identifier_name, assignment_target_member_object_identifier_name,
    expression_identifier_name, expression_member_object_identifier_name, is_type_only,
    line_for_span, member_object_identifier_name, module_export_identifier_name,
    simple_assignment_target_identifier_name,
    simple_assignment_target_member_object_identifier_name, static_member_property_name, UseRecord,
};
use oxc_ast::ast::{
    AssignmentExpression, AssignmentPattern, BindingIdentifier, Expression, FormalParameter,
    IdentifierReference, IfStatement, ImportDeclaration, ImportDeclarationSpecifier,
    LogicalExpression, MemberExpression, Program, Statement, UnaryExpression, UpdateExpression,
    VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use oxc_syntax::operator::UnaryOperator;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug)]
struct NamedImportSeed {
    from_spec: String,
    imported_name: String,
    local_name: String,
    type_only: bool,
    line: usize,
}

#[derive(Debug)]
struct NamedImportMemberUse {
    name: String,
    line: usize,
}

#[derive(Debug)]
struct NamedImportPrecisionRecord {
    from_spec: String,
    imported_name: String,
    local_name: String,
    type_only: bool,
    line: usize,
    members: Vec<NamedImportMemberUse>,
    degraded: bool,
}

fn collect_named_import_seeds(
    program: &Program<'_>,
    line_starts: &[usize],
) -> Vec<NamedImportSeed> {
    let mut out = Vec::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        let specifiers = import
            .specifiers
            .as_ref()
            .map_or(&[][..], |items| items.as_slice());
        for specifier in specifiers {
            let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                continue;
            };
            let imported_name = module_export_identifier_name(&specifier.imported)
                .unwrap_or_else(|| specifier.local.name.to_string());
            out.push(NamedImportSeed {
                from_spec: import.source.value.to_string(),
                imported_name,
                local_name: specifier.local.name.to_string(),
                type_only: is_type_only(import.import_kind) || is_type_only(specifier.import_kind),
                line: line_for_span(line_starts, specifier.span),
            });
        }
    }
    out
}

pub(super) fn collect_named_import_precision_uses(
    program: &Program<'_>,
    line_starts: &[usize],
) -> Vec<UseRecord> {
    let seeds = collect_named_import_seeds(program, line_starts);
    if seeds.is_empty() {
        return Vec::new();
    }
    let mut visitor = NamedImportPrecisionVisitor::new(seeds, line_starts);
    visitor.visit_program(program);
    visitor.into_uses()
}

struct NamedImportPrecisionVisitor<'a> {
    records: Vec<NamedImportPrecisionRecord>,
    index_by_local: BTreeMap<String, usize>,
    scopes: Vec<BTreeSet<String>>,
    line_starts: &'a [usize],
    non_escaping_identifier_depth: usize,
}

impl<'a> NamedImportPrecisionVisitor<'a> {
    fn new(seeds: Vec<NamedImportSeed>, line_starts: &'a [usize]) -> Self {
        let mut records = Vec::with_capacity(seeds.len());
        let mut index_by_local = BTreeMap::new();
        for seed in seeds {
            let index = records.len();
            index_by_local
                .entry(seed.local_name.clone())
                .or_insert(index);
            records.push(NamedImportPrecisionRecord {
                from_spec: seed.from_spec,
                imported_name: seed.imported_name,
                local_name: seed.local_name,
                type_only: seed.type_only,
                line: seed.line,
                members: Vec::new(),
                degraded: false,
            });
        }
        Self {
            records,
            index_by_local,
            scopes: vec![BTreeSet::new()],
            line_starts,
            non_escaping_identifier_depth: 0,
        }
    }

    fn into_uses(self) -> Vec<UseRecord> {
        let mut uses = Vec::new();
        for record in self.records {
            if !record.members.is_empty() && !record.degraded {
                for member in record.members {
                    uses.push(UseRecord {
                        from_spec: record.from_spec.clone(),
                        name: record.imported_name.clone(),
                        member_name: Some(member.name),
                        kind: "imported-namespace-member".to_string(),
                        type_only: record.type_only,
                        line: member.line,
                        local_name: Some(record.local_name.clone()),
                        degraded: false,
                        resolved_file: None,
                        resolver_stage: None,
                    });
                }
            } else if record.degraded {
                uses.push(UseRecord {
                    from_spec: record.from_spec,
                    name: record.imported_name,
                    member_name: None,
                    kind: "imported-namespace-escape".to_string(),
                    type_only: record.type_only,
                    line: record.line,
                    local_name: Some(record.local_name),
                    degraded: true,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
        }
        uses
    }

    fn active_record_index(&self, local_name: &str) -> Option<usize> {
        if self
            .scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(local_name))
        {
            return None;
        }
        self.index_by_local.get(local_name).copied()
    }

    fn add_binding(&mut self, binding: &BindingIdentifier<'_>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(binding.name.to_string());
        }
    }

    fn record_member(&mut self, local_name: &str, member_name: String, line: usize) {
        if let Some(index) = self.active_record_index(local_name) {
            self.records[index].members.push(NamedImportMemberUse {
                name: member_name,
                line,
            });
        }
    }

    fn degrade(&mut self, local_name: &str) {
        if let Some(index) = self.active_record_index(local_name) {
            self.records[index].degraded = true;
        }
    }

    fn with_non_escaping_identifiers(&mut self, f: impl FnOnce(&mut Self)) {
        self.non_escaping_identifier_depth += 1;
        f(self);
        self.non_escaping_identifier_depth -= 1;
    }
}

impl<'a> Visit<'a> for NamedImportPrecisionVisitor<'_> {
    fn enter_scope(
        &mut self,
        _flags: oxc_syntax::scope::ScopeFlags,
        _scope_id: &Cell<Option<oxc_syntax::scope::ScopeId>>,
    ) {
        self.scopes.push(BTreeSet::new());
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
        if self.scopes.is_empty() {
            self.scopes.push(BTreeSet::new());
        }
    }

    fn visit_import_declaration(&mut self, _it: &ImportDeclaration<'a>) {}

    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        self.add_binding(it);
    }

    fn visit_formal_parameter(&mut self, it: &FormalParameter<'a>) {
        self.visit_binding_pattern(&it.pattern);
    }

    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        self.visit_binding_pattern(&it.id);
        if let Some(init) = &it.init {
            self.visit_expression(init);
        }
    }

    fn visit_assignment_pattern(&mut self, it: &AssignmentPattern<'a>) {
        self.visit_binding_pattern(&it.left);
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if self.non_escaping_identifier_depth == 0 {
            self.degrade(it.name.as_str());
        }
    }

    fn visit_member_expression(&mut self, it: &MemberExpression<'a>) {
        if let Some(local_name) = member_object_identifier_name(it) {
            if let Some(member_name) = static_member_property_name(it) {
                let line = line_for_span(self.line_starts, it.span());
                self.record_member(&local_name, member_name, line);
            } else {
                self.degrade(&local_name);
                if let MemberExpression::ComputedMemberExpression(member) = it {
                    self.visit_expression(&member.expression);
                }
            }
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

    fn visit_unary_expression(&mut self, it: &UnaryExpression<'a>) {
        if it.operator == UnaryOperator::Typeof {
            self.visit_maybe_non_escaping_identifier(&it.argument);
            return;
        }
        if it.operator == UnaryOperator::Delete {
            if let Some(local_name) = expression_identifier_name(&it.argument) {
                self.degrade(&local_name);
                return;
            }
            if let Some(local_name) = expression_member_object_identifier_name(&it.argument) {
                self.degrade(&local_name);
                return;
            }
        }
        walk::walk_unary_expression(self, it);
    }

    fn visit_if_statement(&mut self, it: &IfStatement<'a>) {
        self.visit_maybe_non_escaping_identifier(&it.test);
        self.visit_statement(&it.consequent);
        if let Some(alternate) = &it.alternate {
            self.visit_statement(alternate);
        }
    }

    fn visit_logical_expression(&mut self, it: &LogicalExpression<'a>) {
        self.visit_maybe_non_escaping_identifier(&it.left);
        self.visit_expression(&it.right);
    }
}

impl NamedImportPrecisionVisitor<'_> {
    fn visit_maybe_non_escaping_identifier(&mut self, expression: &Expression<'_>) {
        if expression_identifier_name(expression).is_some() {
            self.with_non_escaping_identifiers(|visitor| {
                visitor.visit_expression(expression);
            });
        } else {
            self.visit_expression(expression);
        }
    }
}
