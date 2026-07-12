use crate::locations::LineIndex;
use crate::protocol::{
    AstDefinitionKind, AstShapeConfidence, AstShapeField, AstShapeFieldKind, AstShapeHash,
    AstShapeHashKind, AstShapeKind, AstVisibility, RUST_SHAPE_HASH_NORMALIZED_VERSION,
};
use lumin_rust_common::sha256_text;
use ra_ap_syntax::{
    ast::{self, HasAttrs, HasGenericParams, HasName, HasVisibility, StructKind},
    AstNode, SyntaxNode,
};
use serde::Serialize;

use super::normalize::compact_rust_type_text;
use crate::analyzer::facts::{collect_definition, counted_item_cast, syntax_text, visibility_for};
use crate::analyzer::location::ast_location;
use crate::analyzer::syntax::FileSyntax;

pub(in crate::analyzer::syntax) fn collect_struct(
    node: &SyntaxNode,
    line_index: &LineIndex,
    syntax: &mut FileSyntax,
) {
    let item = counted_item_cast(node, &mut syntax.facts, ast::Struct::cast);
    collect_struct_shape_hash(item.as_ref(), line_index, syntax);
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Struct,
        item,
        line_index,
    );
}

fn collect_struct_shape_hash(
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
    if syntax.retain_raw_ast_lanes {
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
    } else {
        syntax.ast.counts.shape_hashes += 1;
    }
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
        if visibility == AstVisibility::Restricted {
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

fn has_cfg_gate_attr<T: HasAttrs>(owner: &T) -> bool {
    owner.attrs().any(|attr| {
        let text = crate::analyzer::attrs::normalized_attr_text(&attr);
        text.starts_with("#[cfg(")
            || text.starts_with("#![cfg(")
            || text.starts_with("#[cfg_attr(")
            || text.starts_with("#![cfg_attr(")
    })
}
