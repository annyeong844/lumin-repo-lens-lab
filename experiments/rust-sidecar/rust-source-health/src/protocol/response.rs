use serde::Serialize;
use std::collections::BTreeMap;

use super::{AstFunctionCloneGroups, FileHealth, ResponseMeta, SkippedFile, Summary};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub schema_version: u32,
    pub meta: ResponseMeta,
    pub summary: Summary,
    pub function_clone_groups: AstFunctionCloneGroups,
    pub skipped_files: Vec<SkippedFile>,
    pub files: BTreeMap<String, FileHealth>,
}
