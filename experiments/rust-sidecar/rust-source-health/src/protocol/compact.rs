use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    AstFacts, AstOpaqueMuteReason, AstOpaqueSurface, AstOpaqueSurfaceVisibility, Facts, FileHealth,
    FileSignalSummary, Location, ParseStatus, PathMeta, Severity, Signal, SignalKind,
    SignalMuteReason, SignalVisibilityState,
};

const REVIEW_OPAQUE_SURFACE_EXAMPLE_LIMIT: usize = 10;
const REVIEW_SIGNAL_EXAMPLE_LIMIT: usize = 10;
const MUTED_SIGNAL_EXAMPLE_LIMIT: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactFileHealth {
    pub sha256: String,
    pub facts: Facts,
    pub ast_summary: CompactAstSummary,
    pub signal_summary: CompactSignalSummary,
    pub parse: ParseStatus,
    pub path: PathMeta,
}

impl CompactFileHealth {
    pub(crate) fn from_file(file: &FileHealth) -> Self {
        Self {
            sha256: file.sha256.clone(),
            facts: file.facts.clone(),
            ast_summary: CompactAstSummary::from_ast(&file.ast),
            signal_summary: CompactSignalSummary::from_signals_and_summary(
                &file.signals,
                &file.signal_summary,
            ),
            parse: file.parse.clone(),
            path: file.path.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactAstSummary {
    pub definitions: usize,
    pub shape_hashes: usize,
    pub function_signatures: usize,
    pub function_body_fingerprints: usize,
    pub inline_patterns: usize,
    pub impl_blocks: usize,
    pub impl_methods: usize,
    pub use_trees: usize,
    pub path_refs: usize,
    pub name_refs: usize,
    pub method_call_sites: usize,
    pub method_calls: usize,
    pub macro_calls: usize,
    pub cfg_gates: usize,
    pub opaque_surfaces: usize,
    pub review_opaque_surfaces: usize,
    pub muted_opaque_surfaces: usize,
    pub muted_opaque_surfaces_by_reason: BTreeMap<AstOpaqueMuteReason, usize>,
    pub review_opaque_surface_sample_limit: usize,
    pub review_opaque_surface_examples: Vec<AstOpaqueSurface>,
}

impl CompactAstSummary {
    pub(crate) fn from_ast(ast: &AstFacts) -> Self {
        let mut review_opaque_surfaces = 0;
        let mut muted_opaque_surfaces = 0;
        let mut muted_opaque_surfaces_by_reason = BTreeMap::new();
        let mut review_opaque_surface_examples = Vec::new();

        for surface in &ast.opaque_surfaces {
            match surface.visibility {
                AstOpaqueSurfaceVisibility::Review => {
                    review_opaque_surfaces += 1;
                    if review_opaque_surface_examples.len() < REVIEW_OPAQUE_SURFACE_EXAMPLE_LIMIT {
                        review_opaque_surface_examples.push(surface.clone());
                    }
                }
                AstOpaqueSurfaceVisibility::Muted { mute_reason } => {
                    muted_opaque_surfaces += 1;
                    *muted_opaque_surfaces_by_reason
                        .entry(mute_reason)
                        .or_insert(0) += 1;
                }
            }
        }

        Self {
            definitions: ast.counts.definitions,
            shape_hashes: ast.counts.shape_hashes,
            function_signatures: ast.counts.function_signatures,
            function_body_fingerprints: ast.counts.function_body_fingerprints,
            inline_patterns: ast.counts.inline_patterns,
            impl_blocks: ast.counts.impl_blocks,
            impl_methods: ast.counts.impl_methods,
            use_trees: ast.counts.use_trees,
            path_refs: ast.counts.path_refs,
            name_refs: ast.counts.name_refs,
            method_call_sites: ast.counts.method_call_sites,
            method_calls: ast.counts.method_calls,
            macro_calls: ast.counts.macro_calls,
            cfg_gates: ast.counts.cfg_gates,
            opaque_surfaces: ast.counts.opaque_surfaces,
            review_opaque_surfaces,
            muted_opaque_surfaces,
            muted_opaque_surfaces_by_reason,
            review_opaque_surface_sample_limit: REVIEW_OPAQUE_SURFACE_EXAMPLE_LIMIT,
            review_opaque_surface_examples,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactSignalSummary {
    pub total: usize,
    pub review: usize,
    pub muted: usize,
    pub by_kind: BTreeMap<SignalKind, usize>,
    pub muted_by_reason: BTreeMap<SignalMuteReason, usize>,
    pub review_signal_sample_limit: usize,
    pub review_signal_examples: Vec<CompactSignalExample>,
    pub muted_signal_sample_limit: usize,
    pub muted_signal_examples: Vec<CompactSignalExample>,
}

impl CompactSignalSummary {
    pub(crate) fn from_signals_and_summary(
        signals: &[Signal],
        summary: &FileSignalSummary,
    ) -> Self {
        Self {
            total: summary.total,
            review: summary.review,
            muted: summary.muted,
            by_kind: summary.signals_by_kind.clone(),
            muted_by_reason: summary.muted_signals_by_reason.clone(),
            review_signal_sample_limit: REVIEW_SIGNAL_EXAMPLE_LIMIT,
            review_signal_examples: signals
                .iter()
                .filter(|signal| signal.visibility == SignalVisibilityState::Review)
                .take(REVIEW_SIGNAL_EXAMPLE_LIMIT)
                .map(CompactSignalExample::from_signal)
                .collect(),
            muted_signal_sample_limit: MUTED_SIGNAL_EXAMPLE_LIMIT,
            muted_signal_examples: signals
                .iter()
                .filter(|signal| signal.visibility.visibility() == super::SignalVisibility::Muted)
                .take(MUTED_SIGNAL_EXAMPLE_LIMIT)
                .map(CompactSignalExample::from_signal)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactSignalExample {
    pub kind: SignalKind,
    pub severity: Severity,
    pub mute_reason: Option<SignalMuteReason>,
    pub location: Location,
}

impl CompactSignalExample {
    fn from_signal(signal: &Signal) -> Self {
        Self {
            kind: signal.kind,
            severity: signal.severity,
            mute_reason: signal.visibility.mute_reason(),
            location: signal.location.clone(),
        }
    }
}
