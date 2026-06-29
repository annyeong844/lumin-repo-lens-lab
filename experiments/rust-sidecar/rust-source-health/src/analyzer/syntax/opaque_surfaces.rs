mod attribute;

use crate::locations::LineIndex;
use crate::protocol::{
    AstCfgGate, AstMacroCall, AstOpaqueReason, AstOpaqueSurface, AstOpaqueSurfaceKind,
    PathClassification,
};
use ra_ap_syntax::{ast, AstNode, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

use super::{refs, FileSyntax};
use crate::analyzer::attrs::cfg_gate_expr;
use crate::analyzer::facts::macro_path_and_name;
use crate::analyzer::location::ast_location;
use crate::analyzer::opaque::{
    classify_attribute_macro_opaque_surface, classify_cfg_opaque_surface,
    classify_macro_opaque_surface,
};
use crate::analyzer::signal_policy::collect_macro_call_signal;
use attribute::attribute_macro_surface;

pub(super) fn collect_macro_call(
    node: &SyntaxNode,
    line_index: &LineIndex,
    classifications: &[PathClassification],
    syntax: &mut FileSyntax,
) {
    let Some(call) = ast::MacroCall::cast(node.clone()) else {
        return;
    };
    let (path, name) = macro_path_and_name(&call);
    collect_macro_call_signal(node, line_index, &name, &mut syntax.signals);
    let location = ast_location(line_index, call.syntax().text_range());
    let visibility = classify_macro_opaque_surface(&path, &name, call.syntax(), classifications);
    syntax.ast.macro_calls.push(AstMacroCall {
        path: path.clone(),
        name: name.clone(),
        location: location.clone(),
    });
    collect_macro_token_name_refs(call.syntax(), syntax);
    syntax.ast.opaque_surfaces.push(AstOpaqueSurface {
        kind: AstOpaqueSurfaceKind::MacroExpansion,
        reason: AstOpaqueReason::MacroExpansionNotEvaluated,
        visibility,
        detail: path,
        location,
    });
}

fn collect_macro_token_name_refs(node: &SyntaxNode, syntax: &mut FileSyntax) {
    let test_context = refs::syntax_is_in_test_context(node);
    for element in node.descendants_with_tokens() {
        let SyntaxElement::Token(token) = element else {
            continue;
        };
        match token.kind() {
            SyntaxKind::IDENT => refs::record_local_name_ref(syntax, token.text(), test_context),
            SyntaxKind::STRING => {
                for name in format_capture_names(token.text()) {
                    refs::record_local_name_ref(syntax, &name, test_context);
                }
            }
            _ => {}
        }
    }
}

pub(super) fn collect_attr(
    node: &SyntaxNode,
    line_index: &LineIndex,
    classifications: &[PathClassification],
    syntax: &mut FileSyntax,
) {
    let Some(attr) = ast::Attr::cast(node.clone()) else {
        return;
    };
    collect_attribute_string_name_refs(attr.syntax(), syntax);
    if let Some(expr) = cfg_gate_expr(&attr) {
        collect_cfg_opaque_surface(&attr, line_index, classifications, syntax, expr);
        return;
    }
    let Some(surface) = attribute_macro_surface(&attr) else {
        return;
    };
    let location = ast_location(line_index, attr.syntax().text_range());
    let visibility = classify_attribute_macro_opaque_surface(
        surface.derive_mute_reason,
        attr.syntax(),
        classifications,
    );
    syntax.ast.opaque_surfaces.push(AstOpaqueSurface {
        kind: AstOpaqueSurfaceKind::MacroExpansion,
        reason: AstOpaqueReason::MacroExpansionNotEvaluated,
        visibility,
        detail: surface.detail,
        location,
    });
}

fn collect_attribute_string_name_refs(node: &SyntaxNode, syntax: &mut FileSyntax) {
    let test_context = refs::syntax_is_in_test_context(node);
    for element in node.descendants_with_tokens() {
        let SyntaxElement::Token(token) = element else {
            continue;
        };
        if token.kind() != SyntaxKind::STRING {
            continue;
        }
        if !previous_attribute_key_is_reference_slot(&token) {
            continue;
        }
        let Some(body) = string_literal_body(token.text()) else {
            continue;
        };
        let Some(name) = terminal_path_name(body) else {
            continue;
        };
        refs::record_local_name_ref(syntax, name, test_context);
    }
}

fn previous_attribute_key_is_reference_slot(token: &SyntaxToken) -> bool {
    let Some(equals) = previous_non_trivia_token(token) else {
        return false;
    };
    if equals.text() != "=" {
        return false;
    }
    let Some(key) = previous_non_trivia_token(&equals) else {
        return false;
    };
    matches!(
        key.text(),
        "default" | "deserialize_with" | "serialize_with" | "skip_serializing_if" | "with"
    )
}

fn previous_non_trivia_token(token: &SyntaxToken) -> Option<SyntaxToken> {
    let mut previous = token.prev_token();
    while let Some(candidate) = previous {
        if !matches!(
            candidate.kind(),
            SyntaxKind::WHITESPACE | SyntaxKind::COMMENT
        ) {
            return Some(candidate);
        }
        previous = candidate.prev_token();
    }
    None
}

fn format_capture_names(text: &str) -> Vec<String> {
    let Some(body) = string_literal_body(text) else {
        return Vec::new();
    };
    let chars = body.char_indices().collect::<Vec<_>>();
    let mut captures = Vec::new();
    let mut cursor = 0;
    while cursor < chars.len() {
        if chars[cursor].1 != '{' {
            cursor += 1;
            continue;
        }
        if chars.get(cursor + 1).is_some_and(|(_, ch)| *ch == '{') {
            cursor += 2;
            continue;
        }
        let start = cursor + 1;
        let Some((_, first)) = chars.get(start) else {
            break;
        };
        if !is_rust_ident_start(*first) {
            cursor += 1;
            continue;
        }
        let mut end = start + 1;
        while chars
            .get(end)
            .is_some_and(|(_, ch)| is_rust_ident_continue(*ch))
        {
            end += 1;
        }
        captures.push(chars[start].0..chars[end - 1].0 + chars[end - 1].1.len_utf8());
        cursor = end;
    }
    captures
        .into_iter()
        .filter_map(|range| body.get(range).map(str::to_string))
        .collect()
}

fn string_literal_body(text: &str) -> Option<&str> {
    let start = text.find('"')?;
    let end = text.rfind('"')?;
    (start < end).then(|| &text[start + 1..end])
}

fn terminal_path_name(text: &str) -> Option<&str> {
    let mut last = None;
    for segment in text.split("::") {
        if !is_rust_ident(segment) {
            return None;
        }
        last = Some(segment);
    }
    last
}

fn is_rust_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_rust_ident_start(first) && chars.all(is_rust_ident_continue)
}

fn is_rust_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_rust_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn collect_cfg_opaque_surface(
    attr: &ast::Attr,
    line_index: &LineIndex,
    classifications: &[PathClassification],
    syntax: &mut FileSyntax,
    expr: String,
) {
    let location = ast_location(line_index, attr.syntax().text_range());
    let visibility = classify_cfg_opaque_surface(&expr, attr.syntax(), classifications);
    syntax.ast.cfg_gates.push(AstCfgGate {
        expr: expr.clone(),
        location: location.clone(),
    });
    syntax.ast.opaque_surfaces.push(AstOpaqueSurface {
        kind: AstOpaqueSurfaceKind::CfgGate,
        reason: AstOpaqueReason::CfgConditionNotEvaluated,
        visibility,
        detail: expr,
        location,
    });
}
