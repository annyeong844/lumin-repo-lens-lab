use lumin_rust_source_health::protocol::{
    HealthResponse, ParserMeta, PolicyMeta, RuntimeMeta, SidecarMeta,
};
use serde::Serialize;

use crate::prewrite::tokens::{TOKENIZER_VERSION, TOKEN_POLICY_VERSION, WEAK_COMMON_TOKENS};

mod lookup_policy;

use lookup_policy::LookupPolicyMeta;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PreWriteMeta {
    producer: PreWriteProducer,
    source_health: SourceHealthProvenance,
    token_policy: TokenPolicyMeta,
    lookup_policy: LookupPolicyMeta,
}

impl PreWriteMeta {
    pub(super) fn from_syntax(syntax: &HealthResponse) -> Self {
        Self {
            producer: PreWriteProducer::LuminRustAnalyzer,
            source_health: SourceHealthProvenance {
                schema_version: syntax.schema_version,
                parser: syntax.meta.parser.clone(),
                policy: syntax.meta.policy.clone(),
                runtime: syntax.meta.runtime.clone(),
                sidecar: syntax.meta.sidecar.clone(),
            },
            token_policy: TokenPolicyMeta {
                tokenizer_version: TOKENIZER_VERSION,
                token_policy_version: TOKEN_POLICY_VERSION,
                weak_common_tokens: &WEAK_COMMON_TOKENS,
            },
            lookup_policy: LookupPolicyMeta::from_constants(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
enum PreWriteProducer {
    #[serde(rename = "lumin-rust-analyzer")]
    LuminRustAnalyzer,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceHealthProvenance {
    schema_version: u32,
    parser: ParserMeta,
    policy: PolicyMeta,
    runtime: RuntimeMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    sidecar: Option<SidecarMeta>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenPolicyMeta {
    tokenizer_version: &'static str,
    token_policy_version: &'static str,
    weak_common_tokens: &'static [&'static str],
}
