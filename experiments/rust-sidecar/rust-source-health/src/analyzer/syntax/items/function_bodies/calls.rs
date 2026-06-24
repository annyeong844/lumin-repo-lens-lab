use std::collections::BTreeSet;

use ra_ap_syntax::{ast, AstNode, SyntaxNode};

use crate::analyzer::facts::syntax_text;

pub(super) fn collect_call_tokens(body: &SyntaxNode) -> Vec<String> {
    let mut tokens = BTreeSet::new();
    for call in body.descendants().filter_map(ast::CallExpr::cast) {
        if let Some(token) = call
            .expr()
            .map(|expr| compact_call_token_source(&syntax_text(expr.syntax())))
        {
            if !token.is_empty() {
                tokens.insert(token);
            }
        }
    }
    for call in body.descendants().filter_map(ast::MethodCallExpr::cast) {
        if let Some(name) = call.name_ref() {
            tokens.insert(name.text().to_string());
        }
    }
    for call in body.descendants().filter_map(ast::MacroCall::cast) {
        if let Some(path) = call.path() {
            tokens.insert(compact_call_token_source(&syntax_text(path.syntax())));
        }
    }
    tokens.into_iter().collect()
}

fn compact_call_token_source(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}
