use serde::Serialize;

use super::DomainCluster;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct FileLookup {
    kind: FileLookupKind,
    pub(in crate::prewrite) intent_file: String,
    result: FileLookupResult,
    loc: Option<usize>,
    inbound_fan_in: Option<usize>,
    inbound_fan_in_confidence: FanInConfidence,
    submodule: Option<String>,
    boundary: FileBoundary,
    tags: Vec<&'static str>,
    domain_cluster: Option<DomainCluster>,
    citations: Vec<String>,
}

impl FileLookup {
    pub(super) fn new(
        intent_file: String,
        result: FileLookupResult,
        tags: Vec<&'static str>,
        domain_cluster: Option<DomainCluster>,
        citations: Vec<String>,
    ) -> Self {
        Self {
            kind: FileLookupKind::File,
            intent_file,
            result,
            loc: None,
            inbound_fan_in: None,
            inbound_fan_in_confidence: FanInConfidence::Unavailable,
            submodule: None,
            boundary: FileBoundary::not_evaluated(),
            tags,
            domain_cluster,
            citations,
        }
    }

    pub(in crate::prewrite) fn has_domain_cluster(&self) -> bool {
        self.domain_cluster.is_some()
    }

    pub(in crate::prewrite) fn exists(&self) -> bool {
        matches!(self.result, FileLookupResult::Exists)
    }

    pub(in crate::prewrite) fn result(&self) -> FileLookupResult {
        self.result
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(super) enum FileLookupKind {
    #[serde(rename = "file")]
    File,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(in crate::prewrite) enum FileLookupResult {
    #[serde(rename = "FILE_EXISTS")]
    Exists,
    #[serde(rename = "NEW_FILE")]
    New,
    #[serde(rename = "FILE_STATUS_UNKNOWN")]
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum FanInConfidence {
    Unavailable,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileBoundary {
    status: BoundaryStatus,
    rule: Option<String>,
}

impl FileBoundary {
    pub(super) fn not_evaluated() -> Self {
        Self {
            status: BoundaryStatus::NotEvaluated,
            rule: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
enum BoundaryStatus {
    #[serde(rename = "NOT_EVALUATED")]
    NotEvaluated,
}
