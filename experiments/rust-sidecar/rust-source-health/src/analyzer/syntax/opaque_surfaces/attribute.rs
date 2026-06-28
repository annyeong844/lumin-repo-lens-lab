mod derive;
mod inert;

use crate::analyzer::attrs::normalized_attr_text;
use crate::protocol::AstOpaqueMuteReason;
use ra_ap_syntax::ast;

use derive::{derive_items, derive_mute_reason};
use inert::is_inert_attribute;

pub(super) struct AttributeMacroSurface {
    pub(super) detail: String,
    pub(super) derive_mute_reason: Option<AstOpaqueMuteReason>,
}

pub(super) fn attribute_macro_surface(attr: &ast::Attr) -> Option<AttributeMacroSurface> {
    let text = normalized_attr_text(attr);
    let body = attr_body(&text)?;
    if let Some(derive_items) = derive_items(body) {
        return Some(AttributeMacroSurface {
            detail: body.to_string(),
            derive_mute_reason: derive_mute_reason(body, &derive_items),
        });
    }
    let path = attr_path(body)?;
    if is_inert_attribute(path) {
        return None;
    }
    Some(AttributeMacroSurface {
        detail: path.to_string(),
        derive_mute_reason: None,
    })
}

fn attr_body(text: &str) -> Option<&str> {
    text.strip_prefix("#![")
        .or_else(|| text.strip_prefix("#["))
        .and_then(|text| text.strip_suffix(']'))
}

fn attr_path(body: &str) -> Option<&str> {
    let path = body
        .split(['(', '='])
        .next()
        .map(str::trim)
        .filter(|path| !path.is_empty())?;
    Some(path)
}
