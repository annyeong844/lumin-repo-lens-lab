use ra_ap_syntax::{ast, AstNode};

pub(super) fn has_direct_test_attr<T: ast::HasAttrs>(owner: &T) -> bool {
    owner.attrs().any(|attr| {
        let text = normalized_attr_text(&attr);
        text == "#[test]" || attr_path(&text).is_some_and(is_test_attr_path)
    })
}

pub(super) fn has_direct_cfg_test_attr<T: ast::HasAttrs>(owner: &T) -> bool {
    owner.attrs().any(|attr| {
        matches!(
            normalized_attr_text(&attr).as_str(),
            "#[cfg(test)]" | "#![cfg(test)]"
        )
    })
}

pub(super) fn cfg_gate_expr(attr: &ast::Attr) -> Option<String> {
    let text = normalized_attr_text(attr);
    if text.starts_with("#[cfg(")
        || text.starts_with("#![cfg(")
        || text.starts_with("#[cfg_attr(")
        || text.starts_with("#![cfg_attr(")
    {
        Some(text)
    } else {
        None
    }
}

pub(super) fn normalized_attr_text(attr: &ast::Attr) -> String {
    attr.syntax()
        .text()
        .to_string()
        .chars()
        .filter(|value| !value.is_whitespace())
        .collect()
}

fn attr_path(text: &str) -> Option<&str> {
    text.strip_prefix("#![")
        .or_else(|| text.strip_prefix("#["))
        .and_then(|text| text.strip_suffix(']'))
        .and_then(|body| body.split(['(', '=']).next())
        .map(str::trim)
        .filter(|path| !path.is_empty())
}

fn is_test_attr_path(path: &str) -> bool {
    matches!(path, "tokio::test" | "async_std::test" | "actix_rt::test")
}
