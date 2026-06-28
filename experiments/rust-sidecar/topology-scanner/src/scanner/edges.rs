use crate::protocol::ModuleEdge;

pub(super) fn quoted_after(text: &str, needle: &str) -> Option<String> {
    let idx = text.find(needle)?;
    let rest = &text[idx + needle.len()..];
    let quote_pos = rest.find(['\'', '"'])?;
    let quote = rest.as_bytes()[quote_pos] as char;
    let after = &rest[quote_pos + 1..];
    let end = after.find(quote)?;
    Some(after[..end].to_string())
}

pub(super) fn push_edge(
    edges: &mut Vec<ModuleEdge>,
    source: String,
    line: usize,
    type_only: bool,
    re_export: bool,
) {
    edges.push(ModuleEdge {
        source,
        line,
        type_only,
        re_export,
        dynamic: false,
    });
}

pub(super) fn push_dynamic_edge(edges: &mut Vec<ModuleEdge>, source: String, line: usize) {
    edges.push(ModuleEdge {
        source,
        line,
        type_only: false,
        re_export: false,
        dynamic: true,
    });
}

pub(super) fn export_specifiers_type_only(statement: &str) -> bool {
    let Some(open) = statement.find('{') else {
        return false;
    };
    let Some(close_offset) = statement[open + 1..].find('}') else {
        return false;
    };
    let body = &statement[open + 1..open + 1 + close_offset];
    let mut saw_specifier = false;
    for item in body
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        saw_specifier = true;
        if !item.starts_with("type ") {
            return false;
        }
    }
    saw_specifier
}
