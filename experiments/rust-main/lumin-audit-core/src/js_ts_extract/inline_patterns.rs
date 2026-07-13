use super::{line_for_span, property_key_name, shape_hash::generated_file_evidence};
use lumin_rust_common::sha256_text;
use oxc_ast::ast::{
    ArrowFunctionExpression, CallExpression, ChainElement, ExportDefaultDeclaration,
    ExportDefaultDeclarationKind, Expression, Function, MethodDefinition, ObjectProperty, Program,
    Statement, TryStatement, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub(crate) const NORMALIZER_VERSION: &str = "inline-statement-normalizer-v1";
pub(crate) const MAX_CATCH_STATEMENTS: usize = 2;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InlinePatternOccurrence {
    pub pattern_hash: String,
    pub kind: String,
    pub normalized_pattern: String,
    pub file: String,
    pub line: usize,
    pub end_line: usize,
    pub enclosing_function: String,
}

pub(super) struct InlinePatternFileFacts {
    pub(super) occurrences: Vec<InlinePatternOccurrence>,
    pub(super) diagnostics: Vec<Value>,
}

pub(super) fn collect_inline_pattern_facts(
    program: &Program<'_>,
    source: &str,
    owner_file: &str,
    line_starts: &[usize],
) -> InlinePatternFileFacts {
    if let Some(generated_file) = generated_file_evidence(owner_file, source) {
        return InlinePatternFileFacts {
            occurrences: Vec::new(),
            diagnostics: vec![json!({
                "kind": "generated-file-skipped",
                "file": owner_file,
                "generatedFile": generated_file,
            })],
        };
    }
    let mut visitor = InlinePatternVisitor {
        owner_file,
        line_starts,
        occurrences: Vec::new(),
        function_stack: Vec::new(),
        pending_function_name: None,
    };
    visitor.visit_program(program);
    visitor.occurrences.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.end_line.cmp(&right.end_line))
            .then_with(|| left.enclosing_function.cmp(&right.enclosing_function))
    });
    InlinePatternFileFacts {
        occurrences: visitor.occurrences,
        diagnostics: Vec::new(),
    }
}

struct InlinePatternVisitor<'a> {
    owner_file: &'a str,
    line_starts: &'a [usize],
    occurrences: Vec<InlinePatternOccurrence>,
    function_stack: Vec<String>,
    pending_function_name: Option<String>,
}

impl InlinePatternVisitor<'_> {
    fn with_function(&mut self, name: String, visit: impl FnOnce(&mut Self)) {
        self.function_stack.push(name);
        visit(self);
        self.function_stack.pop();
    }

    fn next_function_name(&mut self, declared: Option<&str>) -> String {
        declared
            .map(str::to_string)
            .or_else(|| self.pending_function_name.take())
            .unwrap_or_else(|| "<anonymous>".to_string())
    }

    fn set_pending_name(&mut self, name: Option<String>, visit: impl FnOnce(&mut Self)) {
        let previous = self.pending_function_name.take();
        self.pending_function_name = name;
        visit(self);
        self.pending_function_name = previous;
    }
}

impl<'a> Visit<'a> for InlinePatternVisitor<'_> {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        let name = self.next_function_name(function.id.as_ref().map(|id| id.name.as_str()));
        self.with_function(name, |visitor| {
            walk::walk_function(visitor, function, flags)
        });
    }

    fn visit_arrow_function_expression(&mut self, function: &ArrowFunctionExpression<'a>) {
        let name = self.next_function_name(None);
        self.with_function(name, |visitor| {
            walk::walk_arrow_function_expression(visitor, function)
        });
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        let name = super::binding_identifier_name_ref(&declarator.id).map(str::to_string);
        self.set_pending_name(name, |visitor| {
            walk::walk_variable_declarator(visitor, declarator)
        });
    }

    fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
        let name = property_key_name(&method.key, method.computed);
        self.set_pending_name(name, |visitor| {
            walk::walk_method_definition(visitor, method)
        });
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        let name = property_key_name(&property.key, property.computed);
        self.set_pending_name(name, |visitor| {
            walk::walk_object_property(visitor, property)
        });
    }

    fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'a>) {
        let pending = matches!(
            export.declaration,
            ExportDefaultDeclarationKind::FunctionDeclaration(_)
                | ExportDefaultDeclarationKind::ArrowFunctionExpression(_)
                | ExportDefaultDeclarationKind::FunctionExpression(_)
        )
        .then(|| "default".to_string());
        self.set_pending_name(pending, |visitor| {
            walk::walk_export_default_declaration(visitor, export)
        });
    }

    fn visit_try_statement(&mut self, statement: &TryStatement<'a>) {
        if let Some(handler) = &statement.handler {
            if let Some(normalized_pattern) = normalize_catch_statements(&handler.body.body) {
                self.occurrences.push(InlinePatternOccurrence {
                    pattern_hash: sha256_text(&normalized_pattern),
                    kind: "catch-block".to_string(),
                    normalized_pattern,
                    file: self.owner_file.to_string(),
                    line: line_for_span(self.line_starts, handler.span),
                    end_line: line_for_span(
                        self.line_starts,
                        oxc_span::Span::new(handler.span.end, handler.span.end),
                    ),
                    enclosing_function: self
                        .function_stack
                        .last()
                        .cloned()
                        .unwrap_or_else(|| "<top-level>".to_string()),
                });
            }
        }
        walk::walk_try_statement(self, statement);
    }
}

fn normalize_catch_statements(statements: &[Statement<'_>]) -> Option<String> {
    if statements.is_empty() || statements.len() > MAX_CATCH_STATEMENTS {
        return None;
    }
    let mut normalized = Vec::with_capacity(statements.len());
    for statement in statements {
        let Statement::ExpressionStatement(statement) = statement else {
            return None;
        };
        let Expression::CallExpression(call) = &statement.expression else {
            return None;
        };
        normalized.push(normalize_call(call)?);
    }
    Some(format!("catch {{ {} }}", normalized.join(" ")))
}

fn normalize_call(call: &CallExpression<'_>) -> Option<String> {
    if !call.arguments.is_empty() {
        return None;
    }
    let callee = normalize_expression(&call.callee)?;
    Some(format!("{callee}();"))
}

fn normalize_expression(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::Identifier(_) => Some("<id>".to_string()),
        Expression::ThisExpression(_) => Some("this".to_string()),
        Expression::Super(_) => Some("super".to_string()),
        Expression::ComputedMemberExpression(_) => None,
        Expression::StaticMemberExpression(member) if !member.optional => {
            let object = normalize_expression(&member.object)?;
            Some(format!("{object}.{}", member.property.name))
        }
        Expression::PrivateFieldExpression(member) if !member.optional => {
            let object = normalize_expression(&member.object)?;
            Some(format!("{object}.#{}", member.field.name))
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::StaticMemberExpression(member) if !member.optional => {
                let object = normalize_expression(&member.object)?;
                Some(format!("{object}.{}", member.property.name))
            }
            ChainElement::PrivateFieldExpression(member) if !member.optional => {
                let object = normalize_expression(&member.object)?;
                Some(format!("{object}.#{}", member.field.name))
            }
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::collect_inline_pattern_facts;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    #[test]
    fn collects_only_small_zero_argument_catch_sequences() {
        let source = r#"
            function first() { try { work(); } catch { cleanup(); release.close(); } }
            const second = () => { try { work(); } catch { cleanup(); release.close(); } };
            try { work(); } catch { cleanup(1); }
        "#;
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty());
        let facts = collect_inline_pattern_facts(&parsed.program, source, "src/example.ts", &[0]);
        assert_eq!(facts.occurrences.len(), 2);
        assert_eq!(
            facts.occurrences[0].normalized_pattern,
            "catch { <id>(); <id>.close(); }"
        );
        assert_eq!(
            facts.occurrences[0].pattern_hash,
            facts.occurrences[1].pattern_hash
        );
        assert_eq!(facts.occurrences[0].enclosing_function, "first");
        assert_eq!(facts.occurrences[1].enclosing_function, "second");
    }

    #[test]
    fn skips_generated_files() {
        let source = "function generated() { try {} catch { cleanup(); } }";
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
        let facts =
            collect_inline_pattern_facts(&parsed.program, source, "src/generated/value.ts", &[0]);
        assert!(facts.occurrences.is_empty());
        assert_eq!(
            facts.diagnostics[0]
                .get("kind")
                .and_then(serde_json::Value::as_str),
            Some("generated-file-skipped")
        );
    }
}
