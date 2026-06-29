use crate::analyzer::location::ast_location;
use crate::locations::LineIndex;
use crate::protocol::{
    AstDefinition, AstDefinitionAttribute, AstDefinitionAttributeKind, AstDefinitionKind,
};

use super::visibility_for;
use ra_ap_syntax::{
    ast::{HasAttrs, HasName, HasVisibility},
    AstNode,
};

pub(in crate::analyzer) fn collect_definition<T>(
    definitions: &mut Vec<AstDefinition>,
    kind: AstDefinitionKind,
    item: Option<T>,
    line_index: &LineIndex,
) where
    T: AstNode + HasAttrs + HasName + HasVisibility,
{
    let Some(item) = item else {
        return;
    };
    let Some(name) = item.name() else {
        return;
    };
    definitions.push(AstDefinition {
        kind,
        name: name.text().to_string(),
        visibility: visibility_for(item.visibility()),
        attributes: definition_attributes(&item),
        location: ast_location(line_index, item.syntax().text_range()),
    });
}

fn definition_attributes<T: HasAttrs>(item: &T) -> Vec<AstDefinitionAttribute> {
    item.attrs()
        .map(|attr| {
            let text = crate::analyzer::attrs::normalized_attr_text(&attr);
            AstDefinitionAttribute {
                kind: definition_attribute_kind(&text),
                text,
            }
        })
        .collect()
}

fn definition_attribute_kind(text: &str) -> AstDefinitionAttributeKind {
    if is_cfg_attr(text) {
        AstDefinitionAttributeKind::Cfg
    } else if is_test_attr(text) {
        AstDefinitionAttributeKind::Test
    } else if is_ffi_linker_attr(text) {
        AstDefinitionAttributeKind::FfiLinker
    } else if text.starts_with("#[derive(") {
        AstDefinitionAttributeKind::Derive
    } else {
        AstDefinitionAttributeKind::Other
    }
}

fn is_cfg_attr(text: &str) -> bool {
    text.starts_with("#[cfg(")
        || text.starts_with("#![cfg(")
        || text.starts_with("#[cfg_attr(")
        || text.starts_with("#![cfg_attr(")
}

fn is_test_attr(text: &str) -> bool {
    matches!(
        text,
        "#[test]" | "#[tokio::test]" | "#[async_std::test]" | "#[actix_rt::test]"
    )
}

fn is_ffi_linker_attr(text: &str) -> bool {
    text == "#[no_mangle]"
        || text.starts_with("#[export_name")
        || text.starts_with("#[link_name")
        || text.contains("no_mangle")
        || text.contains("export_name")
        || text.contains("link_name")
}
