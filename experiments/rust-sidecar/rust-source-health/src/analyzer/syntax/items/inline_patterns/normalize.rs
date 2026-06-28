use crate::analyzer::facts::syntax_text;
use ra_ap_syntax::{
    ast::{self, HasArgList, HasGenericArgs},
    AstNode,
};

use super::super::normalize::compact_rust_type_text;

pub(super) fn normalize_statement(statement: &ast::Stmt) -> Option<String> {
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
