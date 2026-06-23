mod attribute;

use crate::locations::LineIndex;
use crate::protocol::{
    AstCfgGate, AstMacroCall, AstOpaqueReason, AstOpaqueSurface, AstOpaqueSurfaceKind,
    PathClassification,
};
use ra_ap_syntax::{ast, AstNode, SyntaxNode};

use super::FileSyntax;
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
