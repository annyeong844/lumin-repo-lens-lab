use crate::protocol::{
    AstOpaqueMuteReason, AstOpaqueSurfaceVisibility, PathClassification, SignalMuteReason,
};

use super::signal_policy::test_context_mute_reason;
use ra_ap_syntax::SyntaxNode;

mod macros;

pub(super) fn classify_macro_opaque_surface(
    path: &str,
    name: &str,
    node: &SyntaxNode,
    classifications: &[PathClassification],
) -> AstOpaqueSurfaceVisibility {
    if classifications.contains(&PathClassification::Generated) {
        return AstOpaqueSurfaceVisibility::Muted {
            mute_reason: AstOpaqueMuteReason::GeneratedPath,
        };
    }
    if classifications.contains(&PathClassification::Test) {
        return AstOpaqueSurfaceVisibility::Muted {
            mute_reason: AstOpaqueMuteReason::TestPath,
        };
    }
    if let Some(reason) = test_context_opaque_mute_reason(node) {
        return AstOpaqueSurfaceVisibility::Muted {
            mute_reason: reason,
        };
    }
    if let Some(reason) = macros::macro_opaque_mute_reason(path, name) {
        return AstOpaqueSurfaceVisibility::Muted {
            mute_reason: reason,
        };
    }

    AstOpaqueSurfaceVisibility::Review
}

pub(super) fn classify_cfg_opaque_surface(
    expr: &str,
    node: &SyntaxNode,
    classifications: &[PathClassification],
) -> AstOpaqueSurfaceVisibility {
    if classifications.contains(&PathClassification::Generated) {
        return AstOpaqueSurfaceVisibility::Muted {
            mute_reason: AstOpaqueMuteReason::GeneratedPath,
        };
    }
    if classifications.contains(&PathClassification::Test) {
        return AstOpaqueSurfaceVisibility::Muted {
            mute_reason: AstOpaqueMuteReason::TestPath,
        };
    }
    if is_cfg_test_expr(expr) {
        return AstOpaqueSurfaceVisibility::Muted {
            mute_reason: AstOpaqueMuteReason::CfgTest,
        };
    }
    if let Some(reason) = test_context_opaque_mute_reason(node) {
        return AstOpaqueSurfaceVisibility::Muted {
            mute_reason: reason,
        };
    }

    AstOpaqueSurfaceVisibility::Review
}

pub(super) fn classify_attribute_macro_opaque_surface(
    derive_mute_reason: Option<AstOpaqueMuteReason>,
    node: &SyntaxNode,
    classifications: &[PathClassification],
) -> AstOpaqueSurfaceVisibility {
    if classifications.contains(&PathClassification::Generated) {
        return AstOpaqueSurfaceVisibility::Muted {
            mute_reason: AstOpaqueMuteReason::GeneratedPath,
        };
    }
    if classifications.contains(&PathClassification::Test) {
        return AstOpaqueSurfaceVisibility::Muted {
            mute_reason: AstOpaqueMuteReason::TestPath,
        };
    }
    if let Some(reason) = test_context_opaque_mute_reason(node) {
        return AstOpaqueSurfaceVisibility::Muted {
            mute_reason: reason,
        };
    }
    if let Some(reason) = derive_mute_reason {
        return AstOpaqueSurfaceVisibility::Muted {
            mute_reason: reason,
        };
    }

    AstOpaqueSurfaceVisibility::Review
}

fn test_context_opaque_mute_reason(node: &SyntaxNode) -> Option<AstOpaqueMuteReason> {
    match test_context_mute_reason(node)? {
        SignalMuteReason::TestPath => Some(AstOpaqueMuteReason::TestPath),
        SignalMuteReason::GeneratedPath => Some(AstOpaqueMuteReason::GeneratedPath),
        SignalMuteReason::TestAttribute => Some(AstOpaqueMuteReason::TestAttribute),
        SignalMuteReason::CfgTest => Some(AstOpaqueMuteReason::CfgTest),
    }
}

fn is_cfg_test_expr(expr: &str) -> bool {
    matches!(expr, "#[cfg(test)]" | "#![cfg(test)]")
        || expr.starts_with("#[cfg_attr(test,")
        || expr.starts_with("#![cfg_attr(test,")
}
