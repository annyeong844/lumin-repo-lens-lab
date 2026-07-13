use serde_json::{json, Map, Value};

use super::groups::{BlockCloneGroup, Instance};
use super::noise::NoisePolicyResult;
use super::policy::{
    thresholds_json, Thresholds, BLOCK_CLONE_NOISE_POLICY_ID, BLOCK_CLONE_NORMALIZATION_POLICY_ID,
    BLOCK_CLONE_POLICY_VERSION, BLOCK_CLONE_SCHEMA_VERSION,
};
use super::protocol::TokenizedFile;

pub(super) struct ArtifactProjectionInput {
    pub(super) generated: String,
    pub(super) root: String,
    pub(super) include_tests: bool,
    pub(super) exclude: Vec<Value>,
    pub(super) incremental: Option<Value>,
    pub(super) tokenized_files: Vec<TokenizedFile>,
    pub(super) thresholds: Thresholds,
    pub(super) noise_policy: NoisePolicyResult,
    pub(super) skipped: Vec<Value>,
    pub(super) diagnostics: Vec<Value>,
    pub(super) unavailable_file_count: usize,
}

pub(super) fn build_artifact(input: ArtifactProjectionInput) -> Value {
    let status = if input.diagnostics.is_empty() && input.skipped.is_empty() {
        "complete"
    } else {
        "confidence-limited"
    };
    let token_count: usize = input
        .tokenized_files
        .iter()
        .map(|file| file.tokens.len())
        .sum();
    let instance_count: usize = input
        .noise_policy
        .groups
        .iter()
        .map(|group| group.instances.len())
        .sum();

    let mut artifact = Map::new();
    artifact.insert(
        "schemaVersion".to_string(),
        json!(BLOCK_CLONE_SCHEMA_VERSION),
    );
    artifact.insert(
        "policyVersion".to_string(),
        json!(BLOCK_CLONE_POLICY_VERSION),
    );
    artifact.insert("status".to_string(), json!(status));
    artifact.insert("generated".to_string(), json!(input.generated));
    artifact.insert("root".to_string(), json!(input.root));
    artifact.insert(
        "scanRange".to_string(),
        json!({
            "includeTests": input.include_tests,
            "exclude": input.exclude,
        }),
    );
    artifact.insert(
        "normalization".to_string(),
        json!({
            "policyId": BLOCK_CLONE_NORMALIZATION_POLICY_ID,
            "mode": "alpha-identifier",
            "preservePropertyNames": true,
            "preserveImportSpecifiers": true,
            "literalPolicy": "classify",
            "importDeclarationPolicy": "skip",
        }),
    );
    artifact.insert("thresholds".to_string(), thresholds_json(&input.thresholds));
    artifact.insert(
        "summary".to_string(),
        json!({
            "fileCount": input.tokenized_files.len(),
            "tokenCount": token_count,
            "groupCount": input.noise_policy.groups.len(),
            "instanceCount": instance_count,
            "skippedFileCount": input.skipped.len(),
            "unavailableFileCount": input.unavailable_file_count,
            "reviewGroupCount": input.noise_policy.review_group_count,
            "mutedGroupCount": input.noise_policy.muted_group_count,
        }),
    );
    artifact.insert(
        "noisePolicy".to_string(),
        json!({
            "policyId": BLOCK_CLONE_NOISE_POLICY_ID,
            "reviewGroupCount": input.noise_policy.review_group_count,
            "mutedGroupCount": input.noise_policy.muted_group_count,
            "mutedByReason": input.noise_policy.muted_by_reason,
            "candidateCapSaturated": input.noise_policy.candidate_cap_saturated,
            "reviewCapSaturated": input.noise_policy.review_cap_saturated,
            "mutedCapSaturated": input.noise_policy.muted_cap_saturated,
        }),
    );
    artifact.insert(
        "groups".to_string(),
        Value::Array(input.noise_policy.groups.iter().map(group_json).collect()),
    );
    artifact.insert("skipped".to_string(), Value::Array(input.skipped));
    artifact.insert("diagnostics".to_string(), Value::Array(input.diagnostics));
    artifact.insert(
        "meta".to_string(),
        json!({
            "generated": input.generated,
            "root": input.root,
            "incremental": input.incremental,
        }),
    );

    Value::Object(artifact)
}

fn group_json(group: &BlockCloneGroup) -> Value {
    let mut object = Map::new();
    object.insert("id".to_string(), json!(group.id));
    object.insert("claim".to_string(), json!(group.claim));
    object.insert("confidence".to_string(), json!(group.confidence));
    object.insert("tokenCount".to_string(), json!(group.token_count));
    object.insert("lineCount".to_string(), json!(group.line_count));
    object.insert("occurrenceCount".to_string(), json!(group.occurrence_count));
    object.insert(
        "normalizationMode".to_string(),
        json!(group.normalization_mode),
    );
    object.insert("reasons".to_string(), json!(group.reasons));
    object.insert(
        "instances".to_string(),
        Value::Array(group.instances.iter().map(instance_json).collect()),
    );
    object.insert("reviewOnly".to_string(), json!(group.review_only));
    object.insert(
        "eligibleForSafeFix".to_string(),
        json!(group.eligible_for_safe_fix),
    );
    if let Some(visibility) = &group.visibility {
        object.insert("visibility".to_string(), json!(visibility));
    }
    if let Some(reason) = &group.mute_reason {
        object.insert("muteReason".to_string(), json!(reason));
    }
    Value::Object(object)
}

fn instance_json(instance: &Instance) -> Value {
    json!({
        "file": instance.file,
        "startLine": instance.start_line,
        "endLine": instance.end_line,
        "startToken": instance.start_token,
        "endToken": instance.end_token,
        "container": instance.container.clone().unwrap_or(Value::Null),
    })
}
