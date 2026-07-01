use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::manifest_meta::{build_manifest_meta, ManifestMeta, ManifestMetaInput};
use crate::orchestration_plan::AuditProfile;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestRootInput {
    pub generated: String,
    pub profile: String,
    pub root: String,
    pub output: String,
    #[serde(default)]
    pub commands_run: Vec<Value>,
    #[serde(default)]
    pub skipped: Vec<Value>,
    pub evidence: ManifestRootEvidenceInput,
    #[serde(default)]
    pub artifacts_produced: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestRootEvidenceInput {
    pub scan_range: Value,
    pub confidence: Value,
    #[serde(default)]
    pub blind_zones: Vec<Value>,
    pub rust_analysis: Value,
    pub generated_artifacts: Value,
    pub living_audit: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestRoot {
    pub meta: ManifestMeta,
    pub profile: AuditProfile,
    pub commands_run: Vec<Value>,
    pub skipped: Vec<Value>,
    pub scan_range: Value,
    pub confidence: Value,
    pub blind_zones: Vec<Value>,
    pub rust_analysis: Value,
    pub generated_artifacts: Value,
    pub living_audit: Value,
    pub artifacts_produced: Vec<String>,
}

pub fn build_manifest_root(input: ManifestRootInput) -> Result<ManifestRoot> {
    let profile = AuditProfile::parse(&input.profile)
        .context("manifest-root: invalid --profile <quick|full|ci>")?;
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
        living_audit: input.evidence.living_audit,
        artifacts_produced: input.artifacts_produced,
    })
}
