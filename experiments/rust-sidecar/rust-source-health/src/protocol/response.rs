use serde::Serialize;
use std::collections::BTreeMap;

use super::{FileHealth, ResponseMeta, SkippedFile, Summary};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub schema_version: u32,
    pub meta: ResponseMeta,
    pub summary: Summary,
    pub skipped_files: Vec<SkippedFile>,
    pub files: BTreeMap<String, FileHealth>,
}
