use lumin_rust_source_health::protocol::{
    IncrementalMeta, InputMeta, ParserEdition, ParserEditionPolicy, ParserEditionSource,
    ParserKind, PolicyMeta as SyntaxPolicyMeta, ResponseMeta as SyntaxMeta,
    RuntimeMeta as SyntaxRuntimeMeta, SidecarMeta as SyntaxSidecarMeta,
    SignalPolicyMeta as SyntaxSignalPolicyMeta, SourceHealthLimit, SourceHealthMode,
    SourceHealthProducer,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SyntaxPhaseMetaBrief<'a> {
    producer: SourceHealthProducer,
    mode: SourceHealthMode,
    parser: SyntaxPhaseParserBrief<'a>,
    policy: SyntaxPhasePolicyBrief<'a>,
    runtime: SyntaxPhaseRuntimeBrief,
    limits: [SourceHealthLimit; 4],
    #[serde(skip_serializing_if = "Option::is_none")]
    sidecar: Option<SyntaxPhaseSidecarBrief<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<&'a InputMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    incremental: Option<&'a IncrementalMeta>,
}

impl<'a> SyntaxPhaseMetaBrief<'a> {
    pub(super) fn from_meta(meta: &'a SyntaxMeta) -> Self {
        Self {
            producer: meta.producer,
            mode: meta.mode,
            parser: SyntaxPhaseParserBrief::from_meta(meta),
            policy: SyntaxPhasePolicyBrief::from_meta(&meta.policy),
            runtime: SyntaxPhaseRuntimeBrief::from_meta(&meta.runtime),
            limits: meta.limits,
            sidecar: meta
                .sidecar
                .as_ref()
                .map(SyntaxPhaseSidecarBrief::from_meta),
            input: meta.input.as_ref(),
            incremental: meta.incremental.as_ref(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPhaseParserBrief<'a> {
    kind: ParserKind,
    version: &'a str,
    edition_policy: ParserEditionPolicy,
    edition: ParserEdition,
    edition_source: ParserEditionSource,
}

impl<'a> SyntaxPhaseParserBrief<'a> {
    fn from_meta(meta: &'a SyntaxMeta) -> Self {
        Self {
            kind: meta.parser.kind,
            version: meta.parser.version.as_str(),
            edition_policy: meta.parser.edition_policy,
            edition: meta.parser.edition,
            edition_source: meta.parser.edition_source,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPhasePolicyBrief<'a> {
    version: &'a str,
    signal_policy: SyntaxPhaseSignalPolicyBrief<'a>,
}

impl<'a> SyntaxPhasePolicyBrief<'a> {
    fn from_meta(meta: &'a SyntaxPolicyMeta) -> Self {
        Self {
            version: meta.version.as_str(),
            signal_policy: SyntaxPhaseSignalPolicyBrief::from_meta(&meta.signal_policy),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPhaseSignalPolicyBrief<'a> {
    id: &'a str,
    version: &'a str,
}

impl<'a> SyntaxPhaseSignalPolicyBrief<'a> {
    fn from_meta(meta: &'a SyntaxSignalPolicyMeta) -> Self {
        Self {
            id: meta.id.as_str(),
            version: meta.version.as_str(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPhaseRuntimeBrief {
    thread_count: usize,
    worker_stack_bytes: usize,
}

impl SyntaxPhaseRuntimeBrief {
    fn from_meta(meta: &SyntaxRuntimeMeta) -> Self {
        Self {
            thread_count: meta.thread_count,
            worker_stack_bytes: meta.worker_stack_bytes,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPhaseSidecarBrief<'a> {
    source_commit: &'a str,
    binary_sha256: &'a str,
}

impl<'a> SyntaxPhaseSidecarBrief<'a> {
    fn from_meta(meta: &'a SyntaxSidecarMeta) -> Self {
        Self {
            source_commit: meta.source_commit.as_str(),
            binary_sha256: meta.binary_sha256.as_str(),
        }
    }
}
