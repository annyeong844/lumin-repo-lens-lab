use serde::{Deserialize, Serialize};

use super::{ParserEdition, ParserEditionPolicy, ParserEditionSource, DEFAULT_WORKER_STACK_BYTES};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthRequest {
    pub schema_version: u32,
    pub root: String,
    pub files: Vec<RequestFile>,
    pub path_policy: PathPolicy,
    pub parser: ParserRequest,
    #[serde(default)]
    pub runtime: RuntimeRequest,
}

#[derive(Debug, Deserialize)]
pub struct RequestFile {
    pub path: String,
    pub sha256: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PathPolicy {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserRequest {
    pub edition_policy: ParserEditionPolicy,
    pub edition: ParserEdition,
    pub edition_source: ParserEditionSource,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeRequest {
    #[serde(default)]
    pub thread_count: Option<usize>,
    #[serde(default = "default_worker_stack_bytes")]
    pub worker_stack_bytes: usize,
}

impl Default for RuntimeRequest {
    fn default() -> Self {
        Self {
            thread_count: None,
            worker_stack_bytes: DEFAULT_WORKER_STACK_BYTES,
        }
    }
}

fn default_worker_stack_bytes() -> usize {
    DEFAULT_WORKER_STACK_BYTES
}
