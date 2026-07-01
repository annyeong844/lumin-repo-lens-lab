use anyhow::{bail, Result};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactSummaryKind {
    FrameworkResourceSurfaces,
    UnusedDeps,
    BlockClones,
}

impl ArtifactSummaryKind {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "framework-resource-surfaces" => Ok(Self::FrameworkResourceSurfaces),
            "unused-deps" => Ok(Self::UnusedDeps),
            "block-clones" => Ok(Self::BlockClones),
            kind => bail!(
                "unsupported --artifact-kind: {kind}. Use framework-resource-surfaces|unused-deps|block-clones."
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ArtifactSummary {
    FrameworkResourceSurfaces(FrameworkResourceSurfacesSummary),
    UnusedDeps(UnusedDependenciesSummary),
    BlockClones(Box<BlockClonesSummary>),
}

pub fn summarize_artifact(kind: ArtifactSummaryKind, artifact: &Value) -> Option<ArtifactSummary> {
    match kind {
        ArtifactSummaryKind::FrameworkResourceSurfaces => {
            summarize_framework_resource_surfaces(artifact)
                .map(ArtifactSummary::FrameworkResourceSurfaces)
        }
        ArtifactSummaryKind::UnusedDeps => {
            summarize_unused_dependencies(artifact).map(ArtifactSummary::UnusedDeps)
        }
        ArtifactSummaryKind::BlockClones => summarize_block_clones(artifact)
            .map(|summary| ArtifactSummary::BlockClones(Box::new(summary))),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkResourceSurfacesSummary {
    pub artifact: &'static str,
    pub schema_version: Value,
    pub policy_version: Value,
    pub total_files_with_surfaces: Value,
    pub total_surface_lanes: Value,
    pub by_lane: Value,
    pub by_capability_pack: Value,
    pub by_confidence: Value,
    pub by_reason: Value,
    pub by_framework: Value,
    pub top_examples: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnusedDependenciesSummary {
    pub artifact: &'static str,
    pub schema_version: Value,
    pub policy_version: Value,
    pub status: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Value>,
    pub package_count: Value,
    pub declared_dependency_count: Value,
    pub used_count: Value,
    pub review_unused_count: Value,
    pub muted_count: Value,
    pub confidence_limited_count: Value,
    pub unavailable_count: Value,
    pub by_reason: Value,
    pub top_review_unused: Vec<TopReviewUnusedDependency>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopReviewUnusedDependency {
    pub package_dir: String,
    pub manifest_path: Value,
    pub name: Value,
    pub field: Value,
    pub reason: Value,
    pub confidence: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockClonesSummary {
    pub artifact: &'static str,
    pub schema_version: Value,
    pub policy_version: Value,
    pub status: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Value>,
    pub review_only: bool,
    pub normalization_policy_id: Value,
    pub normalization_mode: Value,
    pub threshold_policy_id: Value,
    pub noise_policy_id: Value,
    pub thresholds: BTreeMap<String, Value>,
    pub file_count: Value,
    pub token_count: Value,
    pub group_count: Value,
    pub instance_count: Value,
    pub review_group_count: Value,
    pub muted_group_count: Value,
    pub muted_by_reason: Value,
    pub skipped_file_count: Value,
    pub unavailable_file_count: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_saturated: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_cap_saturated: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_cap_saturated: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted_cap_saturated: Option<Value>,
}

pub fn summarize_framework_resource_surfaces(
    artifact: &Value,
) -> Option<FrameworkResourceSurfacesSummary> {
    let artifact_object = artifact.as_object()?;
    let files = artifact_object
        .get("files")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let summary = object_field(artifact_object, "summary");
    let top_examples = summary
        .and_then(|summary| summary.get("topExamples"))
        .and_then(Value::as_array)
        .map(|examples| examples.iter().take(10).cloned().collect())
        .unwrap_or_else(|| fallback_framework_resource_examples(files));

    Some(FrameworkResourceSurfacesSummary {
        artifact: "framework-resource-surfaces.json",
        schema_version: field_or_null(artifact_object, "schemaVersion"),
        policy_version: field_or_null(artifact_object, "policyVersion"),
        total_files_with_surfaces: summary_number_or(
            summary,
            "totalFilesWithSurfaces",
            files.len(),
        ),
        total_surface_lanes: summary_number_or_else(summary, "totalSurfaceLanes", || {
            files
                .iter()
                .map(|entry| {
                    entry
                        .get("surfaceLanes")
                        .and_then(Value::as_array)
                        .map(Vec::len)
                        .unwrap_or(0)
                })
                .sum()
        }),
        by_lane: summary_value_or_empty_object(summary, "byLane"),
        by_capability_pack: summary_value_or_empty_object(summary, "byCapabilityPack"),
        by_confidence: summary_value_or_empty_object(summary, "byConfidence"),
        by_reason: summary_value_or_empty_object(summary, "byReason"),
        by_framework: summary_value_or_empty_object(summary, "byFramework"),
        top_examples,
    })
}

pub fn summarize_unused_dependencies(artifact: &Value) -> Option<UnusedDependenciesSummary> {
    let artifact_object = artifact.as_object()?;
    let summary = object_field(artifact_object, "summary");
    let packages = artifact_object
        .get("packages")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let mut top_review_unused = Vec::new();
    for package in packages {
        let Some(package_object) = package.as_object() else {
            continue;
        };
        let dependencies = package_object
            .get("dependencies")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        for dependency in dependencies {
            if dependency.get("status").and_then(Value::as_str) != Some("review-unused") {
                continue;
            }
            top_review_unused.push(TopReviewUnusedDependency {
                package_dir: package_object
                    .get("packageDir")
                    .and_then(Value::as_str)
                    .unwrap_or(".")
                    .to_string(),
                manifest_path: field_or_null(package_object, "manifestPath"),
                name: dependency.get("name").cloned().unwrap_or(Value::Null),
                field: dependency.get("field").cloned().unwrap_or(Value::Null),
                reason: dependency.get("reason").cloned().unwrap_or(Value::Null),
                confidence: dependency.get("confidence").cloned().unwrap_or(Value::Null),
            });
        }
    }
    top_review_unused.sort_by(|left, right| {
        left.package_dir
            .cmp(&right.package_dir)
            .then_with(|| value_string(&left.name).cmp(&value_string(&right.name)))
            .then_with(|| value_string(&left.field).cmp(&value_string(&right.field)))
    });
    top_review_unused.truncate(10);

    Some(UnusedDependenciesSummary {
        artifact: "unused-deps.json",
        schema_version: field_or_null(artifact_object, "schemaVersion"),
        policy_version: field_or_null(artifact_object, "policyVersion"),
        status: field_or_null(artifact_object, "status"),
        reason: truthy_field(artifact_object, "reason"),
        package_count: summary_number_or(summary, "packageCount", packages.len()),
        declared_dependency_count: summary_value_or_zero(summary, "declaredDependencyCount"),
        used_count: summary_value_or_zero(summary, "usedCount"),
        review_unused_count: summary_value_or_zero(summary, "reviewUnusedCount"),
        muted_count: summary_value_or_zero(summary, "mutedCount"),
        confidence_limited_count: summary_value_or_zero(summary, "confidenceLimitedCount"),
        unavailable_count: summary_value_or_zero(summary, "unavailableCount"),
        by_reason: summary_value_or_empty_object(summary, "byReason"),
        top_review_unused,
    })
}

pub fn summarize_block_clones(artifact: &Value) -> Option<BlockClonesSummary> {
    let artifact_object = artifact.as_object()?;
    let summary = object_field(artifact_object, "summary");
    let groups = artifact_object
        .get("groups")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let thresholds = object_field(artifact_object, "thresholds");
    let normalization = object_field(artifact_object, "normalization");
    let noise_policy = object_field(artifact_object, "noisePolicy");
    let group_count = summary
        .and_then(|summary| number_field(summary, "groupCount"))
        .unwrap_or_else(|| json!(groups.len()));
    let instance_count = summary
        .and_then(|summary| number_field(summary, "instanceCount"))
        .unwrap_or_else(|| json!(count_block_clone_instances(groups)));

    Some(BlockClonesSummary {
        artifact: "block-clones.json",
        schema_version: field_or_null(artifact_object, "schemaVersion"),
        policy_version: field_or_null(artifact_object, "policyVersion"),
        status: field_or_null(artifact_object, "status"),
        reason: truthy_field(artifact_object, "reason"),
        review_only: true,
        normalization_policy_id: nested_field_or_null(normalization, "policyId"),
        normalization_mode: nested_field_or_null(normalization, "mode"),
        threshold_policy_id: nested_field_or_null(thresholds, "policyId"),
        noise_policy_id: nested_field_or_null(noise_policy, "policyId"),
        thresholds: threshold_summary(thresholds),
        file_count: summary_value_or_zero(summary, "fileCount"),
        token_count: summary_value_or_zero(summary, "tokenCount"),
        group_count,
        instance_count,
        review_group_count: nested_value_or(summary, noise_policy, "reviewGroupCount"),
        muted_group_count: nested_value_or(summary, noise_policy, "mutedGroupCount"),
        muted_by_reason: nested_value_or_empty_object(noise_policy, "mutedByReason"),
        skipped_file_count: summary_value_or_zero(summary, "skippedFileCount"),
        unavailable_file_count: summary_value_or_zero(summary, "unavailableFileCount"),
        cap_saturated: own_field(noise_policy, "capSaturated"),
        candidate_cap_saturated: own_field(noise_policy, "candidateCapSaturated"),
        review_cap_saturated: own_field(noise_policy, "reviewCapSaturated"),
        muted_cap_saturated: own_field(noise_policy, "mutedCapSaturated"),
    })
}

fn fallback_framework_resource_examples(files: &[Value]) -> Vec<Value> {
    files
        .iter()
        .take(10)
        .map(|entry| {
            let surface_lanes = entry
                .get("surfaceLanes")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            json!({
                "file": entry.get("file").cloned().unwrap_or(Value::Null),
                "lanes": string_values_from_objects(surface_lanes, "lane"),
                "capabilityPacks": string_values_from_objects(surface_lanes, "capabilityPack"),
                "reasons": string_values_from_objects(surface_lanes, "reason"),
            })
        })
        .collect()
}

fn string_values_from_objects(values: &[Value], field: &str) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| value.get(field).and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

fn threshold_summary(thresholds: Option<&Map<String, Value>>) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    for field in [
        "minTokens",
        "minLines",
        "minOccurrences",
        "maxInstancesPerGroup",
        "maxTokensPerFile",
    ] {
        out.insert(field.to_string(), nested_field_or_null(thresholds, field));
    }
    for field in [
        "maxGroups",
        "maxCandidateGroups",
        "maxReviewGroups",
        "maxMutedGroups",
    ] {
        if let Some(value) = own_field(thresholds, field) {
            out.insert(field.to_string(), value);
        }
    }
    out
}

fn count_block_clone_instances(groups: &[Value]) -> usize {
    groups
        .iter()
        .map(|group| {
            group
                .get("instances")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or(0)
        })
        .sum()
}

fn object_field<'a>(object: &'a Map<String, Value>, field: &str) -> Option<&'a Map<String, Value>> {
    object.get(field).and_then(Value::as_object)
}

fn field_or_null(object: &Map<String, Value>, field: &str) -> Value {
    object.get(field).cloned().unwrap_or(Value::Null)
}

fn nested_field_or_null(object: Option<&Map<String, Value>>, field: &str) -> Value {
    object
        .and_then(|object| object.get(field))
        .cloned()
        .unwrap_or(Value::Null)
}

fn number_field(object: &Map<String, Value>, field: &str) -> Option<Value> {
    object.get(field).filter(|value| value.is_number()).cloned()
}

fn summary_number_or(summary: Option<&Map<String, Value>>, field: &str, fallback: usize) -> Value {
    summary
        .and_then(|summary| number_field(summary, field))
        .unwrap_or_else(|| json!(fallback))
}

fn summary_number_or_else(
    summary: Option<&Map<String, Value>>,
    field: &str,
    fallback: impl FnOnce() -> usize,
) -> Value {
    summary
        .and_then(|summary| number_field(summary, field))
        .unwrap_or_else(|| json!(fallback()))
}

fn summary_value_or_zero(summary: Option<&Map<String, Value>>, field: &str) -> Value {
    summary
        .and_then(|summary| summary.get(field))
        .cloned()
        .unwrap_or_else(|| json!(0))
}

fn summary_value_or_empty_object(summary: Option<&Map<String, Value>>, field: &str) -> Value {
    summary
        .and_then(|summary| summary.get(field))
        .cloned()
        .unwrap_or_else(|| json!({}))
}

fn nested_value_or(
    summary: Option<&Map<String, Value>>,
    fallback_object: Option<&Map<String, Value>>,
    field: &str,
) -> Value {
    fallback_object
        .and_then(|object| object.get(field))
        .or_else(|| summary.and_then(|summary| summary.get(field)))
        .cloned()
        .unwrap_or(Value::Null)
}

fn nested_value_or_empty_object(object: Option<&Map<String, Value>>, field: &str) -> Value {
    object
        .and_then(|object| object.get(field))
        .cloned()
        .unwrap_or_else(|| json!({}))
}

fn own_field(object: Option<&Map<String, Value>>, field: &str) -> Option<Value> {
    object.and_then(|object| object.get(field)).cloned()
}

fn truthy_field(object: &Map<String, Value>, field: &str) -> Option<Value> {
    let value = object.get(field)?;
    match value {
        Value::Null => None,
        Value::Bool(false) => None,
        Value::Number(number) if number.as_f64() == Some(0.0) => None,
        Value::String(text) if text.is_empty() => None,
        _ => Some(value.clone()),
    }
}

fn value_string(value: &Value) -> String {
    value.as_str().unwrap_or("").to_string()
}
