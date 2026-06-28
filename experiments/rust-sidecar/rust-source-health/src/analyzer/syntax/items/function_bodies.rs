mod calls;
mod metrics;
mod normalization;
mod numeric;

use crate::analyzer::facts::visibility_for;
use crate::analyzer::location::{ast_location, line_span};
use crate::locations::LineIndex;
use crate::protocol::{
    AstCallableKind, AstFunctionBodyFingerprint, AstFunctionBodyFingerprintKind, AstFunctionOwner,
    RUST_FUNCTION_BODY_NORMALIZED_VERSION,
};
use lumin_rust_common::sha256_text;
use ra_ap_syntax::{
    ast::{self, HasName, HasVisibility},
    AstNode, SyntaxKind,
};

use calls::collect_call_tokens;
use metrics::{body_statement_count, function_has_token};
use normalization::{compact_token_source, normalize_body, LiteralPolicy};

pub(in crate::analyzer::syntax::items) fn collect_function_body_fingerprint(
    function: &ast::Fn,
    callable_kind: AstCallableKind,
    owner: Option<AstFunctionOwner>,
    line_index: &LineIndex,
) -> Option<AstFunctionBodyFingerprint> {
    let name = function.name()?.text().to_string();
    let body = function.body()?;
    let body_syntax = body.syntax();
    let exact_body = compact_token_source(body_syntax);
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
        statement_count: body_statement_count(&body),
        param_count,
        is_async: function_has_token(function.syntax(), SyntaxKind::ASYNC_KW),
        is_unsafe: function_has_token(function.syntax(), SyntaxKind::UNSAFE_KW),
        is_const: function_has_token(function.syntax(), SyntaxKind::CONST_KW),
        call_tokens: collect_call_tokens(body_syntax),
        location: ast_location(line_index, function.syntax().text_range()),
        body_location: ast_location(line_index, body_syntax.text_range()),
    })
}
