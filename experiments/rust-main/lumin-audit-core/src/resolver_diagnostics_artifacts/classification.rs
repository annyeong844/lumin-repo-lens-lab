use super::protocol::Record;
use super::value_support::{slash, unique_sorted};
use super::{
    GENERATED_ARTIFACT_MISSING_REASON, GENERATED_BLIND_ZONE_RELEVANCE_POLICY_VERSION,
    RESOLVER_BLIND_ZONE_RELEVANCE_POLICY_VERSION, UNKNOWN_INTERNAL_RESOLUTION,
};
use serde_json::{json, Value};

pub(super) fn family_for_record(record: &Record<'_>) -> &'static str {
    let reason = record.str("reason");
    let stage = record.str("resolverStage");
    let specifier = record.str("specifier").unwrap_or_default();
    if reason == Some(GENERATED_ARTIFACT_MISSING_REASON)
        || record.str("hint") == Some("generated-artifact-missing")
        || record.get("generatedArtifact").is_some()
    {
        return "generated-artifacts";
    }
    if stage == Some("import-meta-glob")
        || record.str("unsupportedFamily") == Some("dynamic-modules")
    {
        return "dynamic-modules";
    }
    if record.str("unsupportedFamily") == Some("output-to-source-mapping")
        || reason == Some("output-source-layout-unsupported")
    {
        return "output-to-source-mapping";
    }
    if stage == Some("hash-imports") || specifier.starts_with('#') {
        return "node-imports";
    }
    if stage == Some("relative") {
        return "relative-paths";
    }
    if stage == Some("tsconfig-baseurl") || reason == Some("baseurl-target-missing") {
        return "absolute-project-paths";
    }
    if reason == Some("workspace-package-subpath-target-missing") {
        return "workspace-packages";
    }
    if matches!(
        stage,
        Some("tsconfig-paths" | "exact-alias" | "wildcard-alias")
    ) {
        return "tsconfig-paths";
    }
    if reason == Some(UNKNOWN_INTERNAL_RESOLUTION) {
        return "unknown-internal-resolution";
    }
    "unknown"
}

pub(super) fn resolver_blind_zone_blocking_policy(record: &Record<'_>) -> Value {
    let candidates = target_candidates(record);
    let has_explicit_scope =
        record.str("affectedPackageScope").is_some() || record.str("packageRoot").is_some();
    let mut candidate_relevant_when = Vec::new();
    if !candidates.is_empty() {
        candidate_relevant_when.extend([
            "target-candidate-file",
            "target-candidate-package-scope",
            "target-candidate-submodule",
        ]);
    }
    if has_explicit_scope {
        candidate_relevant_when.push("affected-package-scope");
    }
    if !candidate_relevant_when.is_empty() {
        return json!({
            "policyVersion": RESOLVER_BLIND_ZONE_RELEVANCE_POLICY_VERSION,
            "blockingScope": "candidate-relevant",
            "candidateRelevantWhen": unique_sorted(candidate_relevant_when),
            "mustNotBlockUnrelatedCandidates": true,
        });
    }
    json!({
        "policyVersion": RESOLVER_BLIND_ZONE_RELEVANCE_POLICY_VERSION,
        "blockingScope": "repo-confidence-limited",
        "candidateRelevantWhen": ["owner-unknown-internal"],
        "mustNotBlockUnrelatedCandidates": false,
    })
}

pub(super) fn generated_blind_zone_blocking_policy(consumer_zone: bool) -> Value {
    json!({
        "policyVersion": GENERATED_BLIND_ZONE_RELEVANCE_POLICY_VERSION,
        "blockingScope": "candidate-relevant",
        "candidateRelevantWhen": if consumer_zone {
            json!(["generated-consumer-scope", "generated-consumer-target-submodule"])
        } else {
            json!(["matched-package-root", "target-candidate-submodule"])
        },
        "mustNotBlockUnrelatedCandidates": true,
    })
}

pub(super) fn affected_package_scope_for_record(record: &Record<'_>) -> Option<String> {
    if let Some(scope) = record.str("affectedPackageScope") {
        return Some(scope.to_string());
    }
    let artifact = record.get("generatedArtifact").and_then(Value::as_object);
    for field in ["packageRoot", "packageDir", "workspaceRoot"] {
        if let Some(value) = artifact
            .and_then(|artifact| artifact.get(field))
            .and_then(Value::as_str)
        {
            return Some(value.to_string());
        }
    }
    for candidate in target_candidates(record) {
        if let Some(root) = package_root_from_path(&candidate) {
            return Some(root);
        }
    }
    record
        .str("consumerFile")
        .or_else(|| record.str("fromHint"))
        .and_then(package_root_from_path)
}

pub(super) fn target_candidates(record: &Record<'_>) -> Vec<String> {
    record
        .get("targetCandidates")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn package_root_from_path(candidate_path: &str) -> Option<String> {
    let normalized = slash(candidate_path).trim_start_matches("./").to_string();
    let parts = normalized.split('/').collect::<Vec<_>>();
    if matches!(parts.first(), Some(&"apps" | &"packages")) && parts.len() >= 2 {
        return Some(format!("{}/{}", parts[0], parts[1]));
    }
    None
}

pub(super) fn unresolved_specifier_root(specifier: &str) -> Option<String> {
    if specifier.is_empty() {
        return None;
    }
    if specifier.starts_with("@/") || specifier.starts_with("~/") || specifier.starts_with("#/") {
        return Some(specifier[..2].to_string());
    }
    if specifier.starts_with('#') {
        return Some("#".to_string());
    }
    if specifier.starts_with('@') {
        let parts = specifier.split('/').collect::<Vec<_>>();
        if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Some(format!("{}/{}", parts[0], parts[1]));
        }
    }
    specifier
        .split('/')
        .next()
        .filter(|first| !first.is_empty())
        .map(ToOwned::to_owned)
}
