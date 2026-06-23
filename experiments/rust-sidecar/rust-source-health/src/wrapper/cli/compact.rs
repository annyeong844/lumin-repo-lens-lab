use std::collections::BTreeMap;

use serde::Serialize;

use crate::protocol::{
    AstFacts, AstOpaqueMuteReason, AstOpaqueSurface, AstOpaqueSurfaceVisibility, Facts, FileHealth,
    HealthResponse, ParseStatus, PathMeta, ResponseMeta, Signal, SkippedFile, Summary,
};

const REVIEW_OPAQUE_SURFACE_EXAMPLE_LIMIT: usize = 10;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CompactHealthResponse<'a> {
    schema_version: u32,
    artifact_profile: &'static str,
    meta: &'a ResponseMeta,
    summary: &'a Summary,
    skipped_files: &'a [SkippedFile],
    files: BTreeMap<&'a str, CompactFileHealth<'a>>,
}

impl<'a> CompactHealthResponse<'a> {
    pub(super) fn from_response(response: &'a HealthResponse) -> Self {
        let files = response
            .files
            .iter()
            .map(|(path, file)| (path.as_str(), CompactFileHealth::from_file(file)))
            .collect();

        Self {
            schema_version: response.schema_version,
            artifact_profile: "compact",
            meta: &response.meta,
            summary: &response.summary,
            skipped_files: &response.skipped_files,
            files,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompactFileHealth<'a> {
    sha256: &'a str,
    facts: &'a Facts,
    ast_summary: CompactAstSummary<'a>,
    signals: &'a [Signal],
    parse: &'a ParseStatus,
    path: &'a PathMeta,
}

impl<'a> CompactFileHealth<'a> {
    fn from_file(file: &'a FileHealth) -> Self {
        Self {
            sha256: &file.sha256,
            facts: &file.facts,
            ast_summary: CompactAstSummary::from_ast(&file.ast),
            signals: &file.signals,
            parse: &file.parse,
            path: &file.path,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompactAstSummary<'a> {
    definitions: usize,
    shape_hashes: usize,
    function_signatures: usize,
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
    fn from_ast(ast: &'a AstFacts) -> Self {
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
