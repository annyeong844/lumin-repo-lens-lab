use crate::analyzer::facts::{function_is_unsafe, syntax_text, visibility_for};
use crate::analyzer::location::ast_location;
use crate::locations::LineIndex;
use crate::protocol::{
    AstCallableKind, AstFunctionOwner, AstFunctionParam, AstFunctionReceiver,
    AstFunctionReceiverKind, AstFunctionSignature, AstFunctionSignatureKind,
    RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
};
use lumin_rust_common::sha256_text;
use ra_ap_syntax::{
    ast::{self, HasGenericParams, HasName},
    AstNode, SyntaxKind, SyntaxNode,
};
use serde::Serialize;

use super::super::normalize::compact_rust_type_text;

pub(in crate::analyzer::syntax::items) fn function_signature(
    function: &ast::Fn,
    callable_kind: AstCallableKind,
    owner: Option<AstFunctionOwner>,
    visibility: Option<ast::Visibility>,
    line_index: &LineIndex,
) -> Option<AstFunctionSignature> {
    if function_is_unsafe(function.syntax()) || function_is_async(function.syntax()) {
        return None;
    }
    if function.where_clause().is_some() {
        return None;
    }
    let name = function.name()?;
    let param_list = function.param_list()?;
    let receiver = param_list.self_param().map(function_receiver);
    let params = param_list
        .params()
        .map(|param| {
            let ty = param.ty()?;
            Some(AstFunctionParam {
                type_text: compact_rust_type_text(&syntax_text(ty.syntax())),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let return_type = function
        .ret_type()
        .and_then(|ret_type| ret_type.ty())
        .map(|ty| compact_rust_type_text(&syntax_text(ty.syntax())));
    let generics = function
        .generic_param_list()
        .map(|generics| compact_rust_type_text(&syntax_text(generics.syntax())))
        .filter(|text| !text.is_empty());
    let hash = rust_function_signature_hash(
        callable_kind,
        generics.as_deref(),
        receiver.as_ref(),
        &params,
        return_type.as_deref(),
    )?;

    Some(AstFunctionSignature {
        kind: AstFunctionSignatureKind::FunctionSignature,
        hash,
        name: name.text().to_string(),
        visibility: visibility_for(visibility),
        callable_kind,
        owner,
        normalized_version: RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION.to_string(),
        generics,
        receiver,
        params,
        return_type,
        location: ast_location(line_index, function.syntax().text_range()),
    })
}

fn function_receiver(self_param: ast::SelfParam) -> AstFunctionReceiver {
    let kind = match self_param.kind() {
        ast::SelfParamKind::Owned => AstFunctionReceiverKind::Owned,
        ast::SelfParamKind::Ref => AstFunctionReceiverKind::Ref,
        ast::SelfParamKind::MutRef => AstFunctionReceiverKind::MutRef,
    };
    AstFunctionReceiver {
        kind,
        text: compact_rust_type_text(&syntax_text(self_param.syntax())),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NormalizedRustFunctionSignature<'a> {
    schema_version: &'static str,
    callable_kind: AstCallableKind,
    generics: Option<&'a str>,
    receiver: Option<&'a AstFunctionReceiver>,
    params: &'a [AstFunctionParam],
    return_type: Option<&'a str>,
}

fn rust_function_signature_hash(
    callable_kind: AstCallableKind,
    generics: Option<&str>,
    receiver: Option<&AstFunctionReceiver>,
    params: &[AstFunctionParam],
    return_type: Option<&str>,
) -> Option<String> {
    let normalized = NormalizedRustFunctionSignature {
        schema_version: RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
        callable_kind,
        generics,
        receiver,
        params,
        return_type,
    };
    serde_json::to_string(&normalized)
        .ok()
        .map(|text| sha256_text(&text))
}

fn function_is_async(node: &SyntaxNode) -> bool {
    node.children_with_tokens()
        .any(|child| child.kind() == SyntaxKind::ASYNC_KW)
}
