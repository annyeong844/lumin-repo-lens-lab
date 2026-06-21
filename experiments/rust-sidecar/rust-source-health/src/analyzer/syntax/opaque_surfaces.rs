use crate::locations::LineIndex;
use crate::protocol::{
    AstCfgGate, AstMacroCall, AstOpaqueMuteReason, AstOpaqueReason, AstOpaqueSurface,
    AstOpaqueSurfaceKind, PathClassification,
};
use ra_ap_syntax::{ast, AstNode, SyntaxNode};

use super::FileSyntax;
use crate::analyzer::attrs::{cfg_gate_expr, normalized_attr_text};
use crate::analyzer::facts::macro_path_and_name;
use crate::analyzer::location::ast_location;
use crate::analyzer::opaque::{
    classify_attribute_macro_opaque_surface, classify_cfg_opaque_surface,
    classify_macro_opaque_surface,
};
use crate::analyzer::signal_policy::collect_macro_call_signal;

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
    syntax.ast.opaque_surfaces.push(AstOpaqueSurface {
        kind: AstOpaqueSurfaceKind::MacroExpansion,
        reason: AstOpaqueReason::MacroExpansionNotEvaluated,
        visibility,
        detail: path,
        location,
    });
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

struct AttributeMacroSurface {
    detail: String,
    derive_mute_reason: Option<AstOpaqueMuteReason>,
}

fn attribute_macro_surface(attr: &ast::Attr) -> Option<AttributeMacroSurface> {
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

fn derive_items(body: &str) -> Option<Vec<&str>> {
    let inner = body.strip_prefix("derive(")?.strip_suffix(')')?;
    let items = inner
        .split(',')
        .map(|item| item.rsplit("::").next().unwrap_or(item).trim())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    (!items.is_empty()).then_some(items)
}

fn attr_path(body: &str) -> Option<&str> {
    let path = body
        .split(['(', '='])
        .next()
        .map(str::trim)
        .filter(|path| !path.is_empty())?;
    Some(path)
}

fn is_builtin_derive(name: &str) -> bool {
    matches!(
        name,
        "Clone" | "Copy" | "Debug" | "Default" | "Eq" | "Hash" | "Ord" | "PartialEq" | "PartialOrd"
    )
}

fn derive_mute_reason(body: &str, items: &[&str]) -> Option<AstOpaqueMuteReason> {
    if items.iter().all(|item| is_builtin_derive(item)) {
        return Some(AstOpaqueMuteReason::BuiltinDeriveMacro);
    }
    if items
        .iter()
        .all(|item| is_builtin_derive(item) || is_known_data_derive(body, item))
    {
        return Some(AstOpaqueMuteReason::KnownDataDeriveMacro);
    }
    None
}

fn is_known_data_derive(body: &str, name: &str) -> bool {
    matches!(
        name,
        "Deserialize" | "ExperimentalApi" | "JsonSchema" | "Serialize" | "TS"
    ) || (name == "Message" && body.contains("prost::Message"))
}

fn is_inert_attribute(path: &str) -> bool {
    if is_inert_tool_attribute(path) {
        return true;
    }
    if is_known_derive_helper_attribute(path) {
        return true;
    }
    matches!(
        path,
        "allow"
            | "automatically_derived"
            | "bench"
            | "cold"
            | "deny"
            | "deprecated"
            | "doc"
            | "export_name"
            | "expect"
            | "forbid"
            | "feature"
            | "default"
            | "global_allocator"
            | "ignore"
            | "inline"
            | "link"
            | "link_name"
            | "link_section"
            | "macro_use"
            | "must_use"
            | "no_mangle"
            | "no_main"
            | "no_std"
            | "non_exhaustive"
            | "panic_handler"
            | "path"
            | "proc_macro"
            | "proc_macro_attribute"
            | "proc_macro_derive"
            | "recursion_limit"
            | "repr"
            | "should_panic"
            | "test"
            | "target_feature"
            | "track_caller"
            | "type_length_limit"
            | "used"
            | "warn"
            | "windows_subsystem"
    )
}

fn is_inert_tool_attribute(path: &str) -> bool {
    ["clippy::", "diagnostic::", "miri::", "rustfmt::"]
        .iter()
        .any(|prefix| path.starts_with(prefix))
}

fn is_known_derive_helper_attribute(path: &str) -> bool {
    matches!(
        path,
        "arg"
            | "backtrace"
            | "clap"
            | "command"
            | "error"
            | "from"
            | "prost"
            | "schemars"
            | "serde"
            | "source"
            | "strum"
            | "ts"
    )
}
