use std::collections::BTreeMap;

use serde::Serialize;

mod ast_summary;
mod file;
mod function_clone_groups;

use crate::protocol::{
    HealthResponse, ResponseMeta, RustUnusedDefinitionAnalysis, SkippedFile, Summary,
};
use file::CompactFileHealth;
use function_clone_groups::CompactFunctionCloneGroups;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CompactHealthResponse<'a> {
    schema_version: u32,
    artifact_profile: &'static str,
    meta: &'a ResponseMeta,
    summary: &'a Summary,
    function_clone_groups: CompactFunctionCloneGroups<'a>,
    unused_definition_analysis: &'a RustUnusedDefinitionAnalysis,
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
            function_clone_groups: CompactFunctionCloneGroups::from_groups(
                &response.function_clone_groups,
            ),
            unused_definition_analysis: &response.unused_definition_analysis,
            skipped_files: &response.skipped_files,
            files,
        }
    }
}
