use crate::analyzer::facts::{syntax_text, visibility_for};
use crate::analyzer::location::{ast_location, line_span};
use crate::locations::LineIndex;
use crate::protocol::{
    AstCallableKind, AstFunctionBodyFingerprint, AstFunctionBodyFingerprintKind, AstFunctionOwner,
    RUST_FUNCTION_BODY_NORMALIZED_VERSION,
};
use lumin_rust_common::sha256_text;
use ra_ap_syntax::{
    ast::{self, HasName, HasVisibility},
    AstNode, SyntaxElement, SyntaxKind, SyntaxNode,
};
use std::collections::BTreeSet;

pub(in crate::analyzer::syntax::items) fn collect_function_body_fingerprint(
    function: &ast::Fn,
    callable_kind: AstCallableKind,
    owner: Option<AstFunctionOwner>,
    line_index: &LineIndex,
) -> Option<AstFunctionBodyFingerprint> {
    let name = function.name()?.text().to_string();
    let body = function.body()?;
    let body_syntax = body.syntax();
    let exact_body = compact_source(&syntax_text(body_syntax));
    let normalized_exact = normalize_body(body_syntax, LiteralPolicy::PreserveValues);
    let normalized_structure = normalize_body(body_syntax, LiteralPolicy::AnonymizeValues);
    let param_count = function
        .param_list()
        .map(|params| params.params().count() + usize::from(params.self_param().is_some()))
        .unwrap_or(0);

    Some(AstFunctionBodyFingerprint {
        kind: AstFunctionBodyFingerprintKind::FunctionBodyFingerprint,
        name,
        visibility: visibility_for(function.visibility()),
        callable_kind,
        owner,
        normalized_version: RUST_FUNCTION_BODY_NORMALIZED_VERSION,
        exact_body_hash: sha256_text(&exact_body),
        normalized_exact_hash: sha256_text(&normalized_exact),
        normalized_structure_hash: sha256_text(&normalized_structure),
        body_loc: line_span(line_index, body_syntax.text_range()),
        statement_count: body
            .stmt_list()
            .map(|statements| statements.statements().count())
            .unwrap_or(0),
        param_count,
        is_async: function_has_token(function.syntax(), SyntaxKind::ASYNC_KW),
        is_unsafe: function_has_token(function.syntax(), SyntaxKind::UNSAFE_KW),
        is_const: function_has_token(function.syntax(), SyntaxKind::CONST_KW),
        call_tokens: collect_call_tokens(body_syntax),
        location: ast_location(line_index, function.syntax().text_range()),
        body_location: ast_location(line_index, body_syntax.text_range()),
    })
}

#[derive(Clone, Copy)]
enum LiteralPolicy {
    PreserveValues,
    AnonymizeValues,
}

fn normalize_body(body: &SyntaxNode, literal_policy: LiteralPolicy) -> String {
    let mut tokens = Vec::new();
    for element in body.descendants_with_tokens() {
        let SyntaxElement::Token(token) = element else {
            continue;
        };
        let Some(normalized) = normalize_token(token.kind(), token.text(), literal_policy) else {
            continue;
        };
        tokens.push(normalized);
    }
    tokens.join(" ")
}

fn normalize_token(kind: SyntaxKind, text: &str, literal_policy: LiteralPolicy) -> Option<String> {
    match kind {
        SyntaxKind::WHITESPACE | SyntaxKind::COMMENT => None,
        SyntaxKind::IDENT | SyntaxKind::LIFETIME_IDENT => Some("<id>".to_string()),
        SyntaxKind::INT_NUMBER | SyntaxKind::FLOAT_NUMBER => Some(match literal_policy {
            LiteralPolicy::PreserveValues => text.to_string(),
            LiteralPolicy::AnonymizeValues => "<number>".to_string(),
        }),
        SyntaxKind::STRING | SyntaxKind::BYTE_STRING | SyntaxKind::C_STRING | SyntaxKind::CHAR => {
            Some(match literal_policy {
                LiteralPolicy::PreserveValues => text.to_string(),
                LiteralPolicy::AnonymizeValues => "<literal>".to_string(),
            })
        }
        SyntaxKind::TRUE_KW | SyntaxKind::FALSE_KW => Some(match literal_policy {
            LiteralPolicy::PreserveValues => text.to_string(),
            LiteralPolicy::AnonymizeValues => "<bool>".to_string(),
        }),
        _ => Some(text.to_string()),
    }
}

fn compact_source(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn collect_call_tokens(body: &SyntaxNode) -> Vec<String> {
    let mut tokens = BTreeSet::new();
    for call in body.descendants().filter_map(ast::CallExpr::cast) {
        if let Some(token) = call
            .expr()
            .map(|expr| compact_source(&syntax_text(expr.syntax())))
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
            tokens.insert(compact_source(&syntax_text(path.syntax())));
        }
    }
    tokens.into_iter().collect()
}

fn function_has_token(function: &SyntaxNode, needle: SyntaxKind) -> bool {
    function
        .children_with_tokens()
        .any(|child| child.kind() == needle)
}
