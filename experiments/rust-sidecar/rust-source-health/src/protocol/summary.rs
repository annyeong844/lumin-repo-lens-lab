use serde::Serialize;
use std::collections::BTreeMap;

use super::{AstOpaqueMuteReason, SignalKind, SignalMuteReason, SignalVisibility};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub files: usize,
    pub skipped_files: usize,
    pub parse_error_files: usize,
    pub parse_errors: usize,
    pub functions: usize,
    pub unsafe_blocks: usize,
    pub unsafe_functions: usize,
    pub signals: usize,
    pub definitions: usize,
    pub shape_hashes: usize,
    pub function_signatures: usize,
    pub function_body_fingerprints: usize,
    pub function_clone_exact_body_groups: usize,
    pub function_clone_structure_groups: usize,
    pub function_clone_signature_groups: usize,
    pub function_clone_near_candidates: usize,
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
    pub signals_by_kind: BTreeMap<SignalKind, usize>,
    pub review_signals: usize,
    pub muted_signals: usize,
    pub signals_by_visibility: BTreeMap<SignalVisibility, usize>,
    pub review_signals_by_kind: BTreeMap<SignalKind, usize>,
    pub muted_signals_by_reason: BTreeMap<SignalMuteReason, usize>,
}
