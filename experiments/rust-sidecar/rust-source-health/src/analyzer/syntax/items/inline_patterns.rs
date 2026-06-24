use crate::analyzer::facts::syntax_text;
use crate::analyzer::location::ast_location;
use crate::analyzer::syntax::FileSyntax;
use crate::locations::LineIndex;
use crate::protocol::{
    AstInlinePattern, AstInlinePatternKind, RUST_INLINE_PATTERN_MAX_STATEMENTS,
    RUST_INLINE_PATTERN_NORMALIZED_VERSION,
};
use lumin_rust_common::sha256_text;
use ra_ap_syntax::{
    ast::{self, HasArgList, HasGenericArgs, HasName},
    AstNode, SyntaxKind, SyntaxNode,
};

use super::normalize::compact_rust_type_text;

pub(in crate::analyzer::syntax) fn collect_inline_patterns(
    node: &SyntaxNode,
    line_index: &LineIndex,
    syntax: &mut FileSyntax,
) {
    let Some(stmt_list) = ast::StmtList::cast(node.clone()) else {
        return;
    };
    if stmt_list.tail_expr().is_some() || has_attribute_or_cfg_surface(stmt_list.syntax()) {
        return;
    }
    let statements = stmt_list.statements().collect::<Vec<_>>();
    if statements.is_empty() || statements.len() > RUST_INLINE_PATTERN_MAX_STATEMENTS {
        return;
    }
    let mut normalized_statements = Vec::new();
    for statement in statements {
        let Some(normalized) = normalize_statement(&statement) else {
            return;
        };
        normalized_statements.push(normalized);
    }
    let normalized_pattern = format!("block {{ {} }}", normalized_statements.join(" "));
    syntax.ast.inline_patterns.push(AstInlinePattern {
        kind: AstInlinePatternKind::StatementSequence,
        pattern_hash: sha256_text(&normalized_pattern),
        normalized_pattern,
        normalized_version: RUST_INLINE_PATTERN_NORMALIZED_VERSION,
        statement_count: normalized_statements.len(),
        enclosing_function: enclosing_function_name(stmt_list.syntax()),
        location: ast_location(line_index, stmt_list.syntax().text_range()),
    });
}

fn normalize_statement(statement: &ast::Stmt) -> Option<String> {
    let ast::Stmt::ExprStmt(statement) = statement else {
        return None;
    };
    statement.semicolon_token()?;
    normalize_expr(statement.expr()?)
}

fn normalize_expr(expr: ast::Expr) -> Option<String> {
    match expr {
        ast::Expr::CallExpr(call) => normalize_call_expr(&call),
        ast::Expr::MethodCallExpr(call) => normalize_method_call_expr(&call),
        _ => None,
    }
}

fn normalize_call_expr(call: &ast::CallExpr) -> Option<String> {
    if !arg_list_is_empty(call.arg_list()?) {
        return None;
    }
    let callee = normalize_callee(call.expr()?)?;
    Some(format!("{callee}();"))
}

fn normalize_method_call_expr(call: &ast::MethodCallExpr) -> Option<String> {
    if call.generic_arg_list().is_some() || !arg_list_is_empty(call.arg_list()?) {
        return None;
    }
    let receiver = normalize_receiver(call.receiver()?)?;
    let method = call.name_ref()?.text().to_string();
    Some(format!("{receiver}.{method}();"))
}

fn arg_list_is_empty(arg_list: ast::ArgList) -> bool {
    arg_list.args().next().is_none()
}

fn normalize_callee(expr: ast::Expr) -> Option<String> {
    match expr {
        ast::Expr::PathExpr(path) => normalize_path_text(&syntax_text(path.syntax())),
        _ => None,
    }
}

fn normalize_receiver(expr: ast::Expr) -> Option<String> {
    let text = compact_rust_type_text(&syntax_text(expr.syntax()));
    if text == "self" {
        return Some("self".to_string());
    }
    simple_identifier(&text).then(|| "<id>".to_string())
}

fn normalize_path_text(raw: &str) -> Option<String> {
    let text = compact_rust_type_text(raw);
    if simple_identifier(&text) {
        return Some("<id>".to_string());
    }
    let segments = text.split("::").collect::<Vec<_>>();
    if segments.len() > 1 && segments.iter().all(|segment| simple_identifier(segment)) {
        return Some(text);
    }
    None
}

fn simple_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn enclosing_function_name(node: &SyntaxNode) -> String {
    node.ancestors()
        .skip(1)
        .find_map(|ancestor| ast::Fn::cast(ancestor).and_then(|function| function.name()))
        .map(|name| name.text().to_string())
        .unwrap_or_else(|| "<top-level>".to_string())
}

fn has_attribute_or_cfg_surface(node: &SyntaxNode) -> bool {
    node.descendants()
        .any(|descendant| descendant.kind() == SyntaxKind::ATTR)
}
