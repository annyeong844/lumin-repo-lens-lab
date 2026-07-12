use std::collections::BTreeMap;

use lumin_rust_source_health::protocol::{
    AstFacts, AstOpaqueMuteReason, AstOpaqueSurfaceVisibility, AstOpaqueVisibility,
    CompactAstSummary,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default)]
pub(in crate::policy::syntax) struct AstOpaqueSurfaceCounts {
    review: usize,
    muted: usize,
}

impl AstOpaqueSurfaceCounts {
    pub(in crate::policy::syntax) fn review(self) -> usize {
        self.review
    }

    pub(in crate::policy::syntax) fn muted(self) -> usize {
        self.muted
    }

    fn count_visibility(&mut self, visibility: AstOpaqueVisibility) {
        match visibility {
            AstOpaqueVisibility::Review => self.review += 1,
            AstOpaqueVisibility::Muted => self.muted += 1,
        }
    }
}

pub(in crate::policy::syntax) fn ast_opaque_surface_counts(
    ast: &AstFacts,
) -> AstOpaqueSurfaceCounts {
    let mut counts = AstOpaqueSurfaceCounts::default();
    for surface in &ast.opaque_surfaces {
        counts.count_visibility(surface.visibility.visibility());
    }
    counts
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy::syntax) struct AstSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    review_opaque_surfaces: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    muted_opaque_surfaces: Option<usize>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    muted_opaque_surfaces_by_reason: BTreeMap<AstOpaqueMuteReason, usize>,
}

impl AstSummary {
    pub(in crate::policy::syntax) fn is_empty(&self) -> bool {
        self.review_opaque_surfaces.is_none()
            && self.muted_opaque_surfaces.is_none()
            && self.muted_opaque_surfaces_by_reason.is_empty()
    }

    pub(in crate::policy::syntax) fn opaque_surface_counts(&self) -> AstOpaqueSurfaceCounts {
        AstOpaqueSurfaceCounts {
            review: self.review_opaque_surfaces.unwrap_or(0),
            muted: self.muted_opaque_surfaces.unwrap_or(0),
        }
    }
}

pub(in crate::policy::syntax) fn ast_summary(ast: &AstFacts) -> AstSummary {
    let mut counts = AstOpaqueSurfaceCounts::default();
    let mut muted_opaque_surfaces_by_reason = BTreeMap::new();
    for surface in &ast.opaque_surfaces {
        counts.count_visibility(surface.visibility.visibility());
        match surface.visibility {
            AstOpaqueSurfaceVisibility::Review => {}
            AstOpaqueSurfaceVisibility::Muted { mute_reason } => {
                *muted_opaque_surfaces_by_reason
                    .entry(mute_reason)
                    .or_insert(0) += 1;
            }
        }
    }

    AstSummary {
        review_opaque_surfaces: (counts.review() > 0).then_some(counts.review()),
        muted_opaque_surfaces: (counts.muted() > 0).then_some(counts.muted()),
        muted_opaque_surfaces_by_reason,
    }
}

pub(in crate::policy::syntax) fn ast_summary_from_compact(
    compact: &CompactAstSummary,
) -> AstSummary {
    AstSummary {
        review_opaque_surfaces: (compact.review_opaque_surfaces > 0)
            .then_some(compact.review_opaque_surfaces),
        muted_opaque_surfaces: (compact.muted_opaque_surfaces > 0)
            .then_some(compact.muted_opaque_surfaces),
        muted_opaque_surfaces_by_reason: compact.muted_opaque_surfaces_by_reason.clone(),
    }
}
