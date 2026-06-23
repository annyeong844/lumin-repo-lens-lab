use serde::Serialize;

pub(super) fn artifact_contract_policy() -> ArtifactContractPolicy {
    ArtifactContractPolicy {
        js_ts_precedent: "_lib/rust-topology-prefer.mjs",
        failure_reason: ArtifactContractFailureReason::BlockedArtifactContract,
        status: ArtifactContractStatus::Strict,
        hard_stop: ArtifactContractHardStop::TypedArtifactConstructionBeforeWrite,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ArtifactContractPolicy {
    js_ts_precedent: &'static str,
    failure_reason: ArtifactContractFailureReason,
    status: ArtifactContractStatus,
    hard_stop: ArtifactContractHardStop,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ArtifactContractFailureReason {
    BlockedArtifactContract,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ArtifactContractStatus {
    Strict,
}

#[derive(Debug, Serialize)]
enum ArtifactContractHardStop {
    #[serde(rename = "typed artifact construction before write")]
    TypedArtifactConstructionBeforeWrite,
}
