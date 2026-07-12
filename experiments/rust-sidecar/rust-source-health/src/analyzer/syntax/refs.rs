use crate::locations::LineIndex;
use crate::protocol::{AstMethodCall, AstNameRef, AstPathRef, AstUseTree};
use ra_ap_syntax::{
    ast::{self, HasName, HasVisibility},
    AstNode, SyntaxNode,
};

use super::FileSyntax;
use crate::analyzer::attrs::{has_direct_cfg_test_attr, has_direct_test_attr};
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
            if syntax.retain_raw_ast_lanes {
                collect_use_tree_facts(
                    &mut syntax.ast.use_trees,
                    &use_tree,
                    visibility,
                    line_index,
                );
            } else {
                let mut use_trees = Vec::<AstUseTree>::new();
                collect_use_tree_facts(&mut use_trees, &use_tree, visibility, line_index);
                syntax.ast.counts.use_trees += use_trees.len();
            }
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
    let name = path_terminal_name(&path);
    let test_context = syntax_is_in_test_context(node);
    record_local_name_ref(syntax, &name, test_context);
    if syntax.retain_raw_ast_lanes {
        syntax.ast.path_refs.push(AstPathRef {
            name,
            path: path_text,
            test_context,
            location: ast_location(line_index, expr.syntax().text_range()),
        });
    } else {
        syntax.ast.counts.path_refs += 1;
    }
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
    let name = path_terminal_name(&path);
    let test_context = syntax_is_in_test_context(node);
    record_local_name_ref(syntax, &name, test_context);
    if syntax.retain_raw_ast_lanes {
        syntax.ast.path_refs.push(AstPathRef {
            name,
            path: path_text,
            test_context,
            location: ast_location(line_index, path_type.syntax().text_range()),
        });
    } else {
        syntax.ast.counts.path_refs += 1;
    }
}

pub(super) fn collect_name_ref(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    let Some(name_ref) = ast::NameRef::cast(node.clone()) else {
        return;
    };
    let name = name_ref.text().to_string();
    let test_context = syntax_is_in_test_context(node);
    record_local_name_ref(syntax, &name, test_context);
    syntax.ast.name_ref_count += 1;
    if syntax.retain_raw_name_refs {
        syntax.ast.name_refs.push(AstNameRef {
            name,
            test_context,
            location: ast_location(line_index, name_ref.syntax().text_range()),
        });
    }
}

pub(super) fn collect_potential_const_pattern_ref(node: &SyntaxNode, syntax: &mut FileSyntax) {
    let Some(pattern) = ast::IdentPat::cast(node.clone()) else {
        return;
    };
    let Some(name) = pattern.name() else {
        return;
    };
    let name = name.text().to_string();
    if is_screaming_snake_identifier(&name) {
        record_local_name_ref(syntax, &name, syntax_is_in_test_context(node));
    }
}

pub(super) fn collect_path_pattern_ref(node: &SyntaxNode, syntax: &mut FileSyntax) {
    let Some(pattern) = ast::PathPat::cast(node.clone()) else {
        return;
    };
    let Some(path) = pattern.path() else {
        return;
    };
    let name = path_terminal_name(&path);
    if is_screaming_snake_identifier(&name) {
        record_local_name_ref(syntax, &name, syntax_is_in_test_context(node));
    }
}

pub(super) fn record_local_name_ref(syntax: &mut FileSyntax, name: &str, test_context: bool) {
    if test_context {
        syntax.ast.test_local_ref_names.insert(name.to_string());
    } else {
        syntax.ast.local_ref_names.insert(name.to_string());
    }
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
    if syntax.retain_raw_ast_lanes {
        *syntax
            .ast
            .method_call_counts
            .entry(method.clone())
            .or_insert(0) += 1;
    } else {
        syntax.ast.counts.method_call_sites += 1;
    }
    if !is_review_method_call(&method) {
        return;
    }
    if !syntax.retain_raw_ast_lanes {
        syntax.ast.counts.method_calls += 1;
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

pub(super) fn syntax_is_in_test_context(node: &SyntaxNode) -> bool {
    node.ancestors().any(|ancestor| {
        ast::Fn::cast(ancestor.clone()).is_some_and(|function| {
            has_direct_test_attr(&function) || has_direct_cfg_test_attr(&function)
        }) || ast::Module::cast(ancestor.clone())
            .is_some_and(|module| has_direct_cfg_test_attr(&module))
            || ast::Impl::cast(ancestor)
                .is_some_and(|impl_block| has_direct_cfg_test_attr(&impl_block))
    })
}

fn is_screaming_snake_identifier(name: &str) -> bool {
    let mut has_uppercase = false;
    for ch in name.chars() {
        if ch.is_ascii_lowercase() {
            return false;
        }
        if ch.is_ascii_uppercase() {
            has_uppercase = true;
            continue;
        }
        if !(ch.is_ascii_digit() || ch == '_') {
            return false;
        }
    }
    has_uppercase
}
