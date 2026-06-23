use crate::locations::LineIndex;
use crate::protocol::{
    AstCallableKind, AstDefinitionKind, AstFunctionOwner, AstFunctionParam, AstFunctionReceiver,
    AstFunctionReceiverKind, AstFunctionSignature, AstFunctionSignatureKind, AstImplBlock,
    AstImplMethod, AstShapeConfidence, AstShapeField, AstShapeFieldKind, AstShapeHash,
    AstShapeHashKind, AstShapeKind, SignalKind, RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
    RUST_SHAPE_HASH_NORMALIZED_VERSION,
};
use lumin_rust_common::sha256_text;
use ra_ap_syntax::{
    ast::{self, HasAttrs, HasGenericParams, HasName, HasVisibility, StructKind},
    AstNode, SyntaxKind, SyntaxNode,
};
use serde::Serialize;

use super::FileSyntax;
use crate::analyzer::facts::{
    collect_definition, counted_item_cast, function_is_unsafe, is_unsafe_block_expr, syntax_text,
    visibility_for,
};
use crate::analyzer::location::ast_location;
use crate::analyzer::location::line_span;
use crate::analyzer::signal_policy::contextual_review_signal;

pub(super) fn collect_function(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    syntax.facts.items += 1;
    syntax.facts.functions += 1;
    if function_is_unsafe(node) {
        syntax.facts.unsafe_functions += 1;
    }
    let lines = line_span(line_index, node.text_range());
    syntax.facts.max_function_lines = syntax.facts.max_function_lines.max(lines);
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Function,
        ast::Fn::cast(node.clone()),
        line_index,
    );
    let Some(function) = ast::Fn::cast(node.clone()) else {
        return;
    };
    if function_is_nested_or_associated(node) {
        return;
    }
    if let Some(signature) = function_signature(
        &function,
        AstCallableKind::Function,
        None,
        function.visibility(),
        line_index,
    ) {
        syntax.ast.function_signatures.push(signature);
    }
}

pub(super) fn collect_struct(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    let item = counted_item_cast(node, &mut syntax.facts, ast::Struct::cast);
    collect_struct_shape_hash(item.as_ref(), line_index, syntax);
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Struct,
        item,
        line_index,
    );
}

pub(super) fn collect_struct_shape_hash(
    item: Option<&ast::Struct>,
    line_index: &LineIndex,
    syntax: &mut FileSyntax,
) {
    let Some(item) = item else {
        return;
    };
    if item.generic_param_list().is_some() {
        return;
    }
    if has_cfg_gate_attr(item) {
        return;
    }
    let Some(name) = item.name() else {
        return;
    };
    let StructKind::Record(record_fields) = item.kind() else {
        return;
    };
    let Some(mut fields) = record_shape_fields(&record_fields) else {
        return;
    };
    fields.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.kind.cmp(&right.kind))
            .then(left.type_text.cmp(&right.type_text))
            .then(left.visibility.cmp(&right.visibility))
    });
    let Some(hash) = rust_shape_hash(&fields) else {
        return;
    };
    syntax.ast.shape_hashes.push(AstShapeHash {
        kind: AstShapeHashKind::ShapeHash,
        hash,
        name: name.text().to_string(),
        visibility: visibility_for(item.visibility()),
        shape_kind: AstShapeKind::RecordStruct,
        normalized_version: RUST_SHAPE_HASH_NORMALIZED_VERSION,
        confidence: AstShapeConfidence::High,
        fields,
        location: ast_location(line_index, item.syntax().text_range()),
    });
}

fn record_shape_fields(record_fields: &ast::RecordFieldList) -> Option<Vec<AstShapeField>> {
    let mut fields = Vec::new();
    for field in record_fields.fields() {
        if has_cfg_gate_attr(&field) {
            return None;
        }
        let name = field.name()?;
        let ty = field.ty()?;
        let visibility = visibility_for(field.visibility());
        if visibility == crate::protocol::AstVisibility::Restricted {
            return None;
        }
        fields.push(AstShapeField {
            kind: AstShapeFieldKind::Property,
            name: name.text().to_string(),
            type_text: compact_rust_type_text(&syntax_text(ty.syntax())),
            visibility,
        });
    }
    Some(fields)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NormalizedRustShape<'a> {
    schema_version: &'static str,
    shape_kind: AstShapeKind,
    fields: &'a [AstShapeField],
}

fn rust_shape_hash(fields: &[AstShapeField]) -> Option<String> {
    let normalized = NormalizedRustShape {
        schema_version: RUST_SHAPE_HASH_NORMALIZED_VERSION,
        shape_kind: AstShapeKind::RecordStruct,
        fields,
    };
    serde_json::to_string(&normalized)
        .ok()
        .map(|text| sha256_text(&text))
}

fn function_signature(
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
        normalized_version: RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
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

fn function_is_nested_or_associated(node: &SyntaxNode) -> bool {
    node.ancestors().skip(1).any(|ancestor| {
        matches!(
            ancestor.kind(),
            SyntaxKind::FN | SyntaxKind::ASSOC_ITEM_LIST
        )
    })
}

fn compact_rust_type_text(raw: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    let mut skip_space_after_punctuation = false;
    for ch in raw.chars() {
        if ch.is_whitespace() {
            if !skip_space_after_punctuation {
                pending_space = true;
            }
            continue;
        }
        if compact_type_punctuation(ch) {
            if out.ends_with(' ') {
                out.pop();
            }
            out.push(ch);
            pending_space = false;
            skip_space_after_punctuation = true;
            continue;
        }
        if pending_space && !out.is_empty() {
            out.push(' ');
        }
        out.push(ch);
        pending_space = false;
        skip_space_after_punctuation = false;
    }
    out.trim().to_string()
}

fn has_cfg_gate_attr<T: HasAttrs>(owner: &T) -> bool {
    owner.attrs().any(|attr| {
        let text = crate::analyzer::attrs::normalized_attr_text(&attr);
        text.starts_with("#[cfg(")
            || text.starts_with("#![cfg(")
            || text.starts_with("#[cfg_attr(")
            || text.starts_with("#![cfg_attr(")
    })
}

fn function_is_async(node: &SyntaxNode) -> bool {
    node.children_with_tokens()
        .any(|child| child.kind() == SyntaxKind::ASYNC_KW)
}

fn compact_type_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '&' | ':' | ',' | ';' | '=' | '?'
    )
}

pub(super) fn collect_enum(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Enum,
        counted_item_cast(node, &mut syntax.facts, ast::Enum::cast),
        line_index,
    );
}

pub(super) fn collect_trait(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Trait,
        counted_item_cast(node, &mut syntax.facts, ast::Trait::cast),
        line_index,
    );
}

pub(super) fn collect_impl(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    syntax.facts.items += 1;
    let Some(impl_block) = ast::Impl::cast(node.clone()) else {
        return;
    };
    let Some(target) = impl_block.self_ty() else {
        return;
    };
    let owner = AstFunctionOwner {
        target: syntax_text(target.syntax()),
        trait_path: impl_block
            .trait_()
            .map(|trait_path| syntax_text(trait_path.syntax())),
    };
    let (methods, signatures) = impl_methods_and_signatures(&impl_block, &owner, line_index);
    syntax.ast.function_signatures.extend(signatures);
    syntax.ast.impls.push(AstImplBlock {
        target: owner.target,
        trait_path: owner.trait_path,
        methods,
        location: ast_location(line_index, impl_block.syntax().text_range()),
    });
}

fn impl_methods_and_signatures(
    impl_block: &ast::Impl,
    owner: &AstFunctionOwner,
    line_index: &LineIndex,
) -> (Vec<AstImplMethod>, Vec<AstFunctionSignature>) {
    let Some(items) = impl_block.assoc_item_list() else {
        return (Vec::new(), Vec::new());
    };
    let mut methods = Vec::new();
    let mut signatures = Vec::new();
    for item in items.assoc_items() {
        match item {
            ast::AssocItem::Fn(function) => {
                let Some(name) = function.name() else {
                    continue;
                };
                if let Some(signature) = function_signature(
                    &function,
                    AstCallableKind::ImplMethod,
                    Some(owner.clone()),
                    function.visibility(),
                    line_index,
                ) {
                    signatures.push(signature);
                }
                methods.push(AstImplMethod {
                    name: name.text().to_string(),
                    visibility: visibility_for(function.visibility()),
                    has_receiver: function
                        .param_list()
                        .and_then(|params| params.self_param())
                        .is_some(),
                    location: ast_location(line_index, function.syntax().text_range()),
                });
            }
            ast::AssocItem::Const(_)
            | ast::AssocItem::MacroCall(_)
            | ast::AssocItem::TypeAlias(_) => {}
        }
    }
    (methods, signatures)
}

pub(super) fn collect_module(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Module,
        counted_item_cast(node, &mut syntax.facts, ast::Module::cast),
        line_index,
    );
}

pub(super) fn collect_const(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Const,
        counted_item_cast(node, &mut syntax.facts, ast::Const::cast),
        line_index,
    );
}

pub(super) fn collect_static(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Static,
        counted_item_cast(node, &mut syntax.facts, ast::Static::cast),
        line_index,
    );
}

pub(super) fn collect_type_alias(
    node: &SyntaxNode,
    line_index: &LineIndex,
    syntax: &mut FileSyntax,
) {
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::TypeAlias,
        counted_item_cast(node, &mut syntax.facts, ast::TypeAlias::cast),
        line_index,
    );
}

pub(super) fn collect_unsafe_block(
    node: &SyntaxNode,
    line_index: &LineIndex,
    syntax: &mut FileSyntax,
) -> bool {
    if !is_unsafe_block_expr(node) {
        return false;
    }
    syntax.facts.unsafe_blocks += 1;
    syntax.signals.push(contextual_review_signal(
        SignalKind::UnsafeBlock,
        line_index,
        node,
    ));
    true
}
