use serde::Serialize;

use super::{ParserEdition, ParserEditionPolicy, ParserEditionSource, ParserKind, PathPolicy};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMeta {
    pub producer: SourceHealthProducer,
    pub mode: SourceHealthMode,
    pub parser: ParserMeta,
    pub policy: PolicyMeta,
    pub runtime: RuntimeMeta,
    pub limits: [SourceHealthLimit; 4],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidecar: Option<SidecarMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<InputMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental: Option<IncrementalMeta>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub enum SourceHealthProducer {
    #[serde(rename = "rust-source-health")]
    RustSourceHealth,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceHealthMode {
    SyntaxOnly,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceHealthLimit {
    SyntaxOnly,
    NoTypeInfo,
    NoTraitSolving,
    NoBorrowCheck,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserMeta {
    pub kind: ParserKind,
    pub version: String,
    pub edition_policy: ParserEditionPolicy,
    pub edition: ParserEdition,
    pub edition_source: ParserEditionSource,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyMeta {
    pub version: String,
    pub signal_policy: SignalPolicyMeta,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalPolicyMeta {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeMeta {
    pub thread_count: usize,
    pub worker_stack_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarMeta {
    pub source_commit: String,
    pub binary_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputMeta {
    pub path_policy: PathPolicy,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalMeta {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_status: Option<String>,
    pub changed_files: usize,
    pub reused_files: usize,
    pub dropped_files: usize,
    pub invalidated_files: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkippedFile {
    pub path: String,
    pub reason: SkippedFileReason,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkippedFileReason {
    InvalidUtf8,
}
