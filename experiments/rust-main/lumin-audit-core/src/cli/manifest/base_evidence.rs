use anyhow::{Context, Result};
use std::path::PathBuf;

use lumin_audit_core::artifact_read_metrics::ARTIFACT_READ_EVENTS_SCHEMA_VERSION;
use lumin_audit_core::generated_artifacts::GeneratedArtifactsMode;
use lumin_audit_core::manifest_root::ManifestEvidenceUpdateFields;
use lumin_audit_core::rust_analysis::RustAnalysisRunObservation;

use super::evidence::{build_manifest_evidence_summary_with_reads, ManifestEvidenceReadRequest};
use super::protocol::ManifestEvidenceArtifactReadEvents;

pub(super) struct BasePipelineEvidenceRequest {
    pub(super) label: &'static str,
    pub(super) root: String,
    pub(super) output: PathBuf,
    pub(super) include_tests: bool,
    pub(super) production: bool,
    pub(super) excludes: Vec<String>,
    pub(super) auto_excludes: Vec<String>,
    pub(super) generated_artifacts_mode: String,
    pub(super) rust_analysis_ran: bool,
    pub(super) rust_analysis_run: Option<RustAnalysisRunObservation>,
    pub(super) planned: bool,
    pub(super) skip_reason: Option<String>,
}

pub(super) fn manifest_evidence_for_base_pipeline(
    request: BasePipelineEvidenceRequest,
) -> Result<(
    ManifestEvidenceUpdateFields,
    ManifestEvidenceArtifactReadEvents,
)> {
    if !request.planned {
        let reason = required_base_pipeline_skip_reason(request.skip_reason.as_deref())?;
        let unavailable = || {
            serde_json::json!({
                "status": "unavailable",
                "reason": reason,
            })
        };
        return Ok((
            ManifestEvidenceUpdateFields {
                scan_range: serde_json::json!({
                    "status": "unavailable",
                    "reason": reason,
                    "root": request.root,
                    "includeTests": request.include_tests,
                    "production": request.production,
                    "excludes": request.excludes,
                    "autoExcludes": request.auto_excludes,
                }),
                confidence: unavailable(),
                resolver_diagnostics: unavailable(),
                blind_zones: vec![serde_json::json!({
                    "area": "base-audit",
                    "severity": "scan-gap",
                    "effect": "Base audit evidence was not refreshed for this lifecycle-only run; absence and freshness claims are unavailable.",
                    "reason": reason,
                })],
                rust_analysis: unavailable(),
                generated_artifacts: unavailable(),
                framework_resource_surfaces: unavailable(),
                unused_dependencies: unavailable(),
                block_clones: unavailable(),
                sfc_evidence: unavailable(),
                living_audit: unavailable(),
            },
            ManifestEvidenceArtifactReadEvents {
                schema_version: ARTIFACT_READ_EVENTS_SCHEMA_VERSION,
                root_dir: request.output.to_string_lossy().to_string(),
                reads: Vec::new(),
            },
        ));
    }

    let generated_artifacts_mode =
        GeneratedArtifactsMode::parse(&request.generated_artifacts_mode)?;
    let summary = build_manifest_evidence_summary_with_reads(ManifestEvidenceReadRequest {
        root: request.root,
        output: request.output,
        include_tests: request.include_tests,
        production: request.production,
        excludes: request.excludes,
        auto_excludes: request.auto_excludes,
        generated_artifacts_mode,
        rust_analysis_ran: request.rust_analysis_ran,
        rust_analysis_run: request.rust_analysis_run,
        label: request.label.to_string(),
    })?;
    let evidence = serde_json::from_value::<ManifestEvidenceUpdateFields>(
        serde_json::to_value(summary.summary)
            .with_context(|| format!("{}: invalid summary shape", request.label))?,
    )
    .with_context(|| format!("{}: invalid evidence update shape", request.label))?;
    Ok((evidence, summary.artifact_reads))
}

pub(super) fn required_base_pipeline_skip_reason(reason: Option<&str>) -> Result<&str> {
    reason
        .filter(|reason| !reason.trim().is_empty())
        .context("basePipelineSkipReason is required when basePipelinePlanned is false")
}

pub(super) fn mark_base_evidence_not_refreshed(
    manifest: &mut serde_json::Value,
    reason: &str,
    artifacts_produced: &[String],
) -> Result<()> {
    let object = manifest
        .as_object_mut()
        .context("manifest-root-with-evidence: manifest must be an object")?;
    object.insert(
        "baseEvidence".to_string(),
        serde_json::json!({
            "status": "not-refreshed",
            "reason": reason,
            "artifactsProducedStatus": "current-lifecycle-only",
        }),
    );
    object.insert(
        "artifactsProduced".to_string(),
        serde_json::to_value(artifacts_produced)
            .context("manifest-root-with-evidence: invalid lifecycle artifact inventory")?,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skipped_base_pipeline_uses_the_plan_reason_and_standard_gap_severity() -> Result<()> {
        let output = PathBuf::from("C:/tmp/lumin-pre-write-output");
        let reason =
            "pre-write-only mode uses intent-shaped cold-cache instead of full quick audit";
        let (evidence, reads) = manifest_evidence_for_base_pipeline(BasePipelineEvidenceRequest {
            label: "test",
            root: "C:/repo".to_string(),
            output: output.clone(),
            include_tests: true,
            production: false,
            excludes: Vec::new(),
            auto_excludes: Vec::new(),
            generated_artifacts_mode: "default".to_string(),
            rust_analysis_ran: false,
            rust_analysis_run: None,
            planned: false,
            skip_reason: Some(reason.to_string()),
        })?;
        assert_eq!(evidence.scan_range["status"], "unavailable");
        assert_eq!(evidence.confidence["reason"], reason);
        assert_eq!(evidence.blind_zones[0]["severity"], "scan-gap");
        assert_eq!(reads.root_dir, output.to_string_lossy());
        assert!(reads.reads.is_empty());
        Ok(())
    }

    #[test]
    fn skipped_base_pipeline_preserves_scoped_lifecycle_artifacts() -> Result<()> {
        let mut manifest = serde_json::json!({
            "artifactsProduced": ["symbols.json", "triage.json"]
        });
        let artifacts = vec!["pre-write-advisory.latest.json".to_string()];
        mark_base_evidence_not_refreshed(&mut manifest, "pre-write-only", &artifacts)?;
        assert_eq!(manifest["baseEvidence"]["status"], "not-refreshed");
        assert_eq!(manifest["baseEvidence"]["reason"], "pre-write-only");
        assert_eq!(manifest["artifactsProduced"], serde_json::json!(artifacts));
        Ok(())
    }
}
