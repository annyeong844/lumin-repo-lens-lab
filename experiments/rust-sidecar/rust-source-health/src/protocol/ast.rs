use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

mod definitions;
mod function_bodies;
mod functions;
mod impls;
mod inline_patterns;
mod opaque;
mod refs;
mod shapes;

pub use definitions::{
    AstDefinition, AstDefinitionAttribute, AstDefinitionAttributeKind, AstDefinitionKind,
    AstDefinitionOwner, AstVisibility,
};
pub use function_bodies::{AstFunctionBodyFingerprint, AstFunctionBodyFingerprintKind};
pub use functions::{
    AstCallableKind, AstFunctionOwner, AstFunctionParam, AstFunctionReceiver,
    AstFunctionReceiverKind, AstFunctionSignature, AstFunctionSignatureKind,
};
pub use impls::{AstImplBlock, AstImplMethod};
pub use inline_patterns::{AstInlinePattern, AstInlinePatternKind};
pub use opaque::{
    AstCfgGate, AstMacroCall, AstOpaqueMuteReason, AstOpaqueReason, AstOpaqueSurface,
    AstOpaqueSurfaceKind, AstOpaqueSurfaceVisibility, AstOpaqueVisibility,
};
pub use refs::{AstMethodCall, AstNameRef, AstPathRef, AstUseTree};
pub use shapes::{
    AstShapeConfidence, AstShapeField, AstShapeFieldKind, AstShapeHash, AstShapeHashKind,
    AstShapeKind,
};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFacts {
    pub definitions: Vec<AstDefinition>,
    pub shape_hashes: Vec<AstShapeHash>,
    pub function_signatures: Vec<AstFunctionSignature>,
    pub function_body_fingerprints: Vec<AstFunctionBodyFingerprint>,
    pub inline_patterns: Vec<AstInlinePattern>,
    pub impls: Vec<AstImplBlock>,
    pub use_trees: Vec<AstUseTree>,
    pub path_refs: Vec<AstPathRef>,
    pub name_refs: Vec<AstNameRef>,
    #[serde(skip)]
    pub name_ref_count: usize,
    #[serde(skip)]
    pub counts: AstFactCounts,
    pub method_call_counts: BTreeMap<String, usize>,
    pub method_calls: Vec<AstMethodCall>,
    pub macro_calls: Vec<AstMacroCall>,
    pub cfg_gates: Vec<AstCfgGate>,
    pub opaque_surfaces: Vec<AstOpaqueSurface>,
    #[serde(skip)]
    pub local_ref_names: BTreeSet<String>,
    #[serde(skip)]
    pub test_local_ref_names: BTreeSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AstFactCounts {
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
}

impl AstFacts {
    pub(crate) fn refresh_counts(&mut self) {
        self.counts = AstFactCounts {
            definitions: self.counts.definitions.max(self.definitions.len()),
            shape_hashes: self.counts.shape_hashes.max(self.shape_hashes.len()),
            function_signatures: self
                .counts
                .function_signatures
                .max(self.function_signatures.len()),
            function_body_fingerprints: self
                .counts
                .function_body_fingerprints
                .max(self.function_body_fingerprints.len()),
            inline_patterns: self.counts.inline_patterns.max(self.inline_patterns.len()),
            impl_blocks: self.counts.impl_blocks.max(self.impls.len()),
            impl_methods: self.counts.impl_methods.max(
                self.impls
                    .iter()
                    .map(|impl_block| impl_block.methods.len())
                    .sum(),
            ),
            use_trees: self.counts.use_trees.max(self.use_trees.len()),
            path_refs: self.counts.path_refs.max(self.path_refs.len()),
            name_refs: self.counts.name_refs.max(self.name_ref_count),
            method_call_sites: self
                .counts
                .method_call_sites
                .max(self.method_call_counts.values().sum()),
            method_calls: self.counts.method_calls.max(self.method_calls.len()),
            macro_calls: self.counts.macro_calls.max(self.macro_calls.len()),
            cfg_gates: self.counts.cfg_gates.max(self.cfg_gates.len()),
            opaque_surfaces: self.counts.opaque_surfaces.max(self.opaque_surfaces.len()),
        };
    }

    pub(crate) fn prune_raw_lanes_for_compact_source_health(&mut self) {
        self.shape_hashes = Vec::new();
        self.inline_patterns = Vec::new();
        self.use_trees = Vec::new();
        self.path_refs = Vec::new();
        self.name_refs = Vec::new();
        self.method_call_counts = BTreeMap::new();
        self.method_calls = Vec::new();
        self.macro_calls = Vec::new();
        self.cfg_gates = Vec::new();
    }

    pub(crate) fn prune_phase_lanes_for_compact_source_health(&mut self) {
        self.definitions = Vec::new();
        self.function_signatures = Vec::new();
        self.function_body_fingerprints = Vec::new();
        self.impls = Vec::new();
        self.local_ref_names = BTreeSet::new();
        self.test_local_ref_names = BTreeSet::new();
    }

    pub(crate) fn prune_unused_definition_lanes_for_compact_source_health(&mut self) {
        self.definitions = Vec::new();
        self.impls = Vec::new();
        self.local_ref_names = BTreeSet::new();
        self.test_local_ref_names = BTreeSet::new();
    }
}
