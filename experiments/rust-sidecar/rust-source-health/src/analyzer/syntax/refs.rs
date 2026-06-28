use crate::locations::LineIndex;
use crate::protocol::{AstMethodCall, AstPathRef};
use ra_ap_syntax::{
    ast::{self, HasVisibility},
    AstNode, SyntaxNode,
};

use super::FileSyntax;
use crate::analyzer::facts::{
    collect_use_tree_facts, is_qualified_path_ref, is_review_method_call, path_ref_text,
    path_terminal_name, syntax_text, visibility_for,
};
use crate::analyzer::location::ast_location;
use crate::analyzer::signal_policy::collect_method_call_signal;

pub(super) fn collect_use_tree(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    if let Some(use_item) = ast::Use::cast(node.clone()) {
        let visibility = visibility_for(use_item.visibility());
        if let Some(use_tree) = use_item.use_tree() {
            collect_use_tree_facts(&mut syntax.ast.use_trees, &use_tree, visibility, line_index);
        }
    }
}

pub(super) fn collect_path_ref(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    let Some(expr) = ast::PathExpr::cast(node.clone()) else {
        return;
    };
    let Some(path) = expr.path() else {
        return;
    };
    let path_text = path_ref_text(&path);
    if !is_qualified_path_ref(&path_text) {
        return;
    }
    syntax.ast.path_refs.push(AstPathRef {
        name: path_terminal_name(&path),
        path: path_text,
        location: ast_location(line_index, expr.syntax().text_range()),
    });
}

pub(super) fn collect_type_path_ref(
    node: &SyntaxNode,
    line_index: &LineIndex,
    syntax: &mut FileSyntax,
) {
    let Some(path_type) = ast::PathType::cast(node.clone()) else {
        return;
    };
    let Some(path) = path_type.path() else {
        return;
    };
    let path_text = path_ref_text(&path);
    if !is_qualified_path_ref(&path_text) {
        return;
    }
    syntax.ast.path_refs.push(AstPathRef {
        name: path_terminal_name(&path),
        path: path_text,
        location: ast_location(line_index, path_type.syntax().text_range()),
    });
}

pub(super) fn collect_method_call(
    node: &SyntaxNode,
    line_index: &LineIndex,
    syntax: &mut FileSyntax,
) {
    let Some(call) = ast::MethodCallExpr::cast(node.clone()) else {
        return;
    };
    let Some(name_ref) = call.name_ref() else {
        return;
    };
    let method = name_ref.text().to_string();
    collect_method_call_signal(node, line_index, &method, &mut syntax.signals);
    *syntax
        .ast
        .method_call_counts
        .entry(method.clone())
        .or_insert(0) += 1;
    if !is_review_method_call(&method) {
        return;
    }
    let receiver = call
        .receiver()
        .map(|receiver| syntax_text(receiver.syntax()))
        .unwrap_or_else(|| "<unknown>".to_string());
    syntax.ast.method_calls.push(AstMethodCall {
        method,
        receiver,
        location: ast_location(line_index, call.syntax().text_range()),
    });
}
