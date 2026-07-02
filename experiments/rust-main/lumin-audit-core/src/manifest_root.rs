use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::Path;

use crate::artifact_registry::collect_produced_artifacts_for_manifest;
use crate::manifest_meta::{build_manifest_meta, ManifestMeta, ManifestMetaInput};
use crate::orchestration_events::ProducerMemory;
use crate::orchestration_plan::AuditProfile;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestRootInput {
    pub generated: String,
    pub profile: String,
    pub root: String,
    pub output: String,
    #[serde(default)]
    pub commands_run: Vec<ManifestCommandRun>,
    #[serde(default)]
    pub skipped: Vec<ManifestSkippedStep>,
    pub evidence: ManifestEvidenceUpdateFields,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestCommandRun {
    pub step: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<ProducerMemory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analyzer_invocation: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestSkippedStep {
    pub step: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestEvidenceUpdateInput {
    pub evidence: ManifestEvidenceUpdateFields,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestEvidenceUpdateFields {
    pub scan_range: Value,
    pub confidence: Value,
    pub resolver_diagnostics: Value,
    #[serde(default)]
    pub blind_zones: Vec<Value>,
    pub rust_analysis: Value,
    pub generated_artifacts: Value,
    pub framework_resource_surfaces: Value,
    pub unused_dependencies: Value,
    pub block_clones: Value,
    pub sfc_evidence: Value,
    pub living_audit: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestRoot {
    pub meta: ManifestMeta,
    pub profile: AuditProfile,
    pub commands_run: Vec<ManifestCommandRun>,
    pub skipped: Vec<ManifestSkippedStep>,
    pub scan_range: Value,
    pub confidence: Value,
    pub blind_zones: Vec<Value>,
    pub rust_analysis: Value,
    pub generated_artifacts: Value,
    pub resolver_diagnostics: Value,
    pub framework_resource_surfaces: Value,
    pub unused_dependencies: Value,
    pub block_clones: Value,
    pub sfc_evidence: Value,
    pub living_audit: Value,
    pub artifacts_produced: Vec<String>,
}

pub fn build_manifest_root(input: ManifestRootInput) -> Result<ManifestRoot> {
    validate_runtime_observations(&input.commands_run, &input.skipped)?;
    let profile = AuditProfile::parse(&input.profile)
        .context("manifest-root: invalid --profile <quick|full|ci>")?;
    let artifacts_produced = collect_produced_artifacts_for_manifest(
        Path::new(&input.output),
        Some(&input.evidence.rust_analysis),
    )?;
    let meta = build_manifest_meta(ManifestMetaInput {
        generated: input.generated,
        profile: input.profile,
        root: input.root,
        output: input.output,
    })?;

    Ok(ManifestRoot {
        meta,
        profile,
        commands_run: input.commands_run,
        skipped: input.skipped,
        scan_range: input.evidence.scan_range,
        confidence: input.evidence.confidence,
        blind_zones: input.evidence.blind_zones,
        rust_analysis: input.evidence.rust_analysis,
        generated_artifacts: input.evidence.generated_artifacts,
        resolver_diagnostics: input.evidence.resolver_diagnostics,
        framework_resource_surfaces: input.evidence.framework_resource_surfaces,
        unused_dependencies: input.evidence.unused_dependencies,
        block_clones: input.evidence.block_clones,
        sfc_evidence: input.evidence.sfc_evidence,
        living_audit: input.evidence.living_audit,
        artifacts_produced,
    })
}

pub fn build_manifest_evidence_update(
    input: ManifestEvidenceUpdateInput,
) -> ManifestEvidenceUpdateFields {
    input.evidence
}

pub fn apply_manifest_evidence_update(
    manifest: &mut Value,
    evidence: ManifestEvidenceUpdateFields,
) -> Result<()> {
    let manifest_object = manifest
        .as_object_mut()
        .context("manifest-lifecycle-evidence-refresh: manifest must be a JSON object")?;
    let update = serde_json::to_value(evidence)
        .context("manifest-lifecycle-evidence-refresh: failed to serialize evidence update")?;
    let Some(update_object) = update.as_object() else {
        bail!("manifest-lifecycle-evidence-refresh: evidence update must be a JSON object");
    };
    insert_update_fields(manifest_object, update_object);
    Ok(())
}

fn insert_update_fields(manifest: &mut Map<String, Value>, update: &Map<String, Value>) {
    for (key, value) in update {
        manifest.insert(key.clone(), value.clone());
    }
}

fn validate_runtime_observations(
    commands_run: &[ManifestCommandRun],
    skipped: &[ManifestSkippedStep],
) -> Result<()> {
    for command in commands_run {
        validate_required("manifest-root: commandsRun[].step", &command.step)?;
        validate_required("manifest-root: commandsRun[].status", &command.status)?;
    }
    for skipped_step in skipped {
        validate_required("manifest-root: skipped[].step", &skipped_step.step)?;
        validate_required("manifest-root: skipped[].reason", &skipped_step.reason)?;
    }
    Ok(())
}

fn validate_required(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{label} must be a non-empty string");
    }
    Ok(())
}
