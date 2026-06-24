use std::collections::BTreeMap;

use serde::Serialize;

use crate::protocol::{
    AstFacts, AstOpaqueMuteReason, AstOpaqueSurface, AstOpaqueSurfaceVisibility,
};

const REVIEW_OPAQUE_SURFACE_EXAMPLE_LIMIT: usize = 10;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CompactAstSummary<'a> {
    definitions: usize,
    shape_hashes: usize,
    function_signatures: usize,
    function_body_fingerprints: usize,
    inline_patterns: usize,
    impl_blocks: usize,
    impl_methods: usize,
    use_trees: usize,
    path_refs: usize,
    method_call_sites: usize,
    method_calls: usize,
    macro_calls: usize,
    cfg_gates: usize,
    opaque_surfaces: usize,
    review_opaque_surfaces: usize,
    muted_opaque_surfaces: usize,
    muted_opaque_surfaces_by_reason: BTreeMap<AstOpaqueMuteReason, usize>,
    review_opaque_surface_sample_limit: usize,
    review_opaque_surface_examples: Vec<&'a AstOpaqueSurface>,
}

impl<'a> CompactAstSummary<'a> {
    pub(super) fn from_ast(ast: &'a AstFacts) -> Self {
        let mut review_opaque_surfaces = 0;
        let mut muted_opaque_surfaces = 0;
        let mut muted_opaque_surfaces_by_reason = BTreeMap::new();
        let mut review_opaque_surface_examples = Vec::new();

        for surface in &ast.opaque_surfaces {
            match surface.visibility {
                AstOpaqueSurfaceVisibility::Review => {
                    review_opaque_surfaces += 1;
                    if review_opaque_surface_examples.len() < REVIEW_OPAQUE_SURFACE_EXAMPLE_LIMIT {
                        review_opaque_surface_examples.push(surface);
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
            definitions: ast.definitions.len(),
            shape_hashes: ast.shape_hashes.len(),
            function_signatures: ast.function_signatures.len(),
            function_body_fingerprints: ast.function_body_fingerprints.len(),
            inline_patterns: ast.inline_patterns.len(),
            impl_blocks: ast.impls.len(),
            impl_methods: ast
                .impls
                .iter()
                .map(|impl_block| impl_block.methods.len())
                .sum(),
            use_trees: ast.use_trees.len(),
            path_refs: ast.path_refs.len(),
            method_call_sites: ast.method_call_counts.values().sum(),
            method_calls: ast.method_calls.len(),
            macro_calls: ast.macro_calls.len(),
            cfg_gates: ast.cfg_gates.len(),
            opaque_surfaces: ast.opaque_surfaces.len(),
            review_opaque_surfaces,
            muted_opaque_surfaces,
            muted_opaque_surfaces_by_reason,
            review_opaque_surface_sample_limit: REVIEW_OPAQUE_SURFACE_EXAMPLE_LIMIT,
            review_opaque_surface_examples,
        }
    }
}
