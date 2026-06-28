use ra_ap_syntax::{ast, AstNode, SyntaxNode};

pub(in crate::analyzer) fn is_qualified_path_ref(path: &str) -> bool {
    path.contains("::")
}

pub(in crate::analyzer) fn macro_path_and_name(call: &ast::MacroCall) -> (String, String) {
    let path = call.path();
    let path_text = path
        .as_ref()
        .map(|path| syntax_text(path.syntax()))
        .unwrap_or_else(|| "<unknown>".to_string());
    let name = path
        .and_then(|path| path.segment())
        .and_then(|segment| segment.name_ref())
        .map(|name_ref| name_ref.text().to_string())
        .unwrap_or_else(|| path_text.clone());
    (path_text, name)
}

pub(in crate::analyzer) fn path_terminal_name(path: &ast::Path) -> String {
    path.segment()
        .and_then(|segment| segment.name_ref())
        .map(|name_ref| name_ref.text().to_string())
        .unwrap_or_else(|| syntax_text(path.syntax()))
}

pub(in crate::analyzer) fn path_ref_text(path: &ast::Path) -> String {
    let mut segments = Vec::new();
    collect_path_ref_segments(path, &mut segments);
    if segments.is_empty() {
        syntax_text(path.syntax())
    } else {
        segments.join("::")
    }
}

fn collect_path_ref_segments(path: &ast::Path, segments: &mut Vec<String>) {
    if let Some(qualifier) = path.qualifier() {
        collect_path_ref_segments(&qualifier, segments);
    }
    if let Some(name_ref) = path.segment().and_then(|segment| segment.name_ref()) {
        segments.push(name_ref.text().to_string());
    }
}

pub(in crate::analyzer) fn syntax_text(node: &SyntaxNode) -> String {
    node.text().to_string()
}
