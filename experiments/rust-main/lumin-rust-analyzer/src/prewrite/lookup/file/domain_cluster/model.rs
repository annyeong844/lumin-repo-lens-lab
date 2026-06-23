use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct DomainCluster {
    pub(super) kind: DomainClusterKind,
    pub(super) directory: String,
    pub(super) basename_prefix: String,
    pub(super) match_kind: DomainClusterMatchKind,
    pub(super) prefix_path: String,
    pub(super) match_count: usize,
    pub(super) total_loc: Option<usize>,
    pub(super) examples: Vec<DomainClusterExample>,
    pub(super) omitted_count: usize,
    pub(super) citations: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(super) enum DomainClusterKind {
    #[serde(rename = "DOMAIN_CLUSTER_DETECTED")]
    Detected,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum DomainClusterMatchKind {
    Prefix,
    DomainToken,
}

#[derive(Debug, Serialize)]
pub(super) struct DomainClusterExample {
    pub(super) file: String,
    pub(super) loc: Option<usize>,
}

pub(super) struct DomainClusterCandidate {
    pub(super) display: String,
    pub(super) key: String,
    pub(super) token_count: usize,
}

pub(super) struct DomainClusterEntry<'a> {
    pub(super) file: &'a str,
    pub(super) loc: Option<usize>,
}
