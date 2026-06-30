use serde::Serialize;
use std::collections::BTreeMap;

mod function_clone_groups;

use crate::driver::CompactAnalysisResponse;
use crate::protocol::{
    CompactFileHealth as OwnedCompactFileHealth, ResponseMeta, RustUnusedDefinitionAnalysis,
    SkippedFile, Summary,
};
use function_clone_groups::CompactFunctionCloneGroups;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CompactAnalysisHealthResponse<'a> {
    schema_version: u32,
    artifact_profile: &'static str,
    meta: &'a ResponseMeta,
    summary: &'a Summary,
    function_clone_groups: CompactFunctionCloneGroups<'a>,
    unused_definition_analysis: &'a RustUnusedDefinitionAnalysis,
    skipped_files: &'a [SkippedFile],
    files: &'a BTreeMap<String, OwnedCompactFileHealth>,
}

impl<'a> CompactAnalysisHealthResponse<'a> {
    pub(super) fn from_analysis(response: &'a CompactAnalysisResponse) -> Self {
        Self {
            schema_version: response.schema_version,
            artifact_profile: "compact",
            meta: &response.meta,
            summary: &response.summary,
            function_clone_groups: CompactFunctionCloneGroups::from_groups(
                &response.function_clone_groups,
            ),
            unused_definition_analysis: &response.unused_definition_analysis,
            skipped_files: &response.skipped_files,
            files: &response.files,
        }
    }
}
