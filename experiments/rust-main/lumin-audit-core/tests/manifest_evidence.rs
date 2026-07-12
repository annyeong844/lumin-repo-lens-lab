use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

use lumin_audit_core::generated_artifacts::GeneratedArtifactsMode;
use lumin_audit_core::manifest_evidence::{
    summarize_manifest_evidence, ManifestEvidenceArtifacts, ManifestEvidenceOptions,
};

#[test]
fn manifest_evidence_composes_rust_owned_fields_with_blind_zones() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("LUMIN_AUDIT.md"), "audit")?;
    let root_text = root.path().to_string_lossy().to_string();
    let triage = json!({
        "byLanguage": {
            "ts": 6,
            "rs": 2
        },
        "shape": {
            "totalFiles": 8,
            "tsFiles": 5
        }
    });
    let symbols = json!({
        "uses": {
            "external": 2,
            "resolvedInternal": 3,
            "unresolvedInternal": 1,
            "unresolvedInternalRatio": 0.25,
            "sfcTemplateComponentRefs": 2
        },
        "generatedArtifactBlindZones": [
            {
                "path": "src/generated/client.ts",
                "reason": "generated-client"
            }
        ],
        "unresolvedInternalSpecifierRecords": [
            {
                "specifier": "@/missing",
                "consumerFile": "src/app.ts",
                "reason": "alias-prefix-unresolved"
            }
        ]
    });
    let resolver_diagnostics = json!({
        "resolverVersion": "resolver-v1",
        "summary": {
            "blindZoneCount": 1,
            "blockedCandidateHintCount": 0,
            "candidateTargetCount": 3
        },
        "blockedCandidateHints": []
    });
    let framework_resource_surfaces = json!({
        "schemaVersion": "framework-resource-surfaces.v1",
        "policyVersion": "framework-resource-surface-policy-v1",
        "files": [
            {
                "file": "src/App.stories.tsx",
                "surfaceLanes": [
                    {
                        "lane": "framework-dispatch-entry",
                        "capabilityPack": "framework.storybook",
                        "reason": "storybook-story-file"
                    }
                ]
            }
        ]
    });
    let unused_deps = json!({
        "schemaVersion": "unused-deps.v1",
        "policyVersion": "unused-deps-review-policy-v1",
        "status": "complete",
        "summary": {
            "packageCount": 1,
            "declaredDependencyCount": 1,
            "usedCount": 1,
            "reviewUnusedCount": 0,
            "mutedCount": 0,
            "confidenceLimitedCount": 0,
            "unavailableCount": 0,
            "byReason": {}
        },
        "packages": []
    });
    let block_clones = json!({
        "schemaVersion": "block-clones.v1",
        "policyVersion": "block-clone-review-policy-v1",
        "status": "complete",
        "normalization": {
            "policyId": "block-clone-normalization-v1",
            "mode": "alpha-identifier"
        },
        "thresholds": {
            "policyId": "block-clone-threshold-policy-v1",
            "minTokens": 50,
            "minLines": 5,
            "minOccurrences": 2,
            "maxInstancesPerGroup": 20,
            "maxTokensPerFile": 200000
        },
        "summary": {
            "fileCount": 1,
            "tokenCount": 100,
            "groupCount": 0,
            "instanceCount": 0
        },
        "groups": []
    });

    let summary = serde_json::to_value(summarize_manifest_evidence(
        ManifestEvidenceOptions {
            root: root_text,
            include_tests: false,
            production: true,
            excludes: vec!["dist".to_string()],
            auto_excludes: vec![".audit".to_string()],
            generated_artifacts_mode: GeneratedArtifactsMode::Present,
            rust_analysis_ran: false,
            rust_analysis_run: None,
        },
        ManifestEvidenceArtifacts {
            triage: Some(&triage),
            symbols: Some(&symbols),
            resolver_capabilities: None,
            resolver_diagnostics: Some(&resolver_diagnostics),
            framework_resource_surfaces: Some(&framework_resource_surfaces),
            unused_deps: Some(&unused_deps),
            block_clones: Some(&block_clones),
            dead_classify: None,
            entry_surface: None,
            rust_analysis: None,
        },
    )?)?;

    let blind_zones = summary["blindZones"].as_array().ok_or_else(|| {
        anyhow::anyhow!("blindZones should be serialized by manifest evidence summary")
    })?;
    assert!(blind_zones.iter().any(|zone| {
        zone.get("area").and_then(|area| area.as_str()) == Some("rs")
            && zone.get("severity").and_then(|severity| severity.as_str()) == Some("scan-gap")
    }));
    assert_eq!(summary["scanRange"]["includeTests"], false);
    assert_eq!(summary["confidence"]["unresolvedInternal"], 1);
    assert_eq!(
        summary["resolverDiagnostics"]["resolverVersion"],
        "resolver-v1"
    );
    assert_eq!(summary["generatedArtifacts"]["mode"], "present");
    assert_eq!(
        summary["frameworkResourceSurfaces"]["artifact"],
        "framework-resource-surfaces.json"
    );
    assert_eq!(
        summary["unusedDependencies"]["artifact"],
        "unused-deps.json"
    );
    assert_eq!(summary["blockClones"]["artifact"], "block-clones.json");
    assert_eq!(summary["sfcEvidence"]["totalEvidenceCount"], 2);
    assert_eq!(
        summary["livingAudit"]["existingDocs"][0]["path"],
        "LUMIN_AUDIT.md"
    );
    Ok(())
}

#[test]
fn cli_manifest_evidence_summary_reads_output_artifacts() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output_dir = root.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(
        output_dir.join("triage.json"),
        serde_json::to_vec(&json!({
            "summary": { "files": 3 },
            "shape": { "jsFiles": 1 },
            "byLanguage": { "rs": 1 }
        }))?,
    )?;
    fs::write(
        output_dir.join("symbols.json"),
        serde_json::to_vec(&json!({
            "uses": {
                "external": 1,
                "resolvedInternal": 1,
                "unresolvedInternal": 0,
                "unresolvedInternalRatio": 0
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-summary")
        .arg("--root")
        .arg(root.path())
        .arg("--output")
        .arg(&output_dir)
        .arg("--include-tests")
        .arg("--no-production")
        .arg("--generated-artifacts")
        .arg("default")
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["scanRange"]["files"], 3);
    assert_eq!(stdout["confidence"]["externalImports"], 1);
    assert_eq!(
        stdout["resolverDiagnostics"]["resolverVersion"],
        json!(null)
    );
    assert_eq!(stdout["rustAnalysis"], json!(null));
    assert_eq!(stdout["frameworkResourceSurfaces"], json!(null));
    assert_eq!(
        stdout["livingAudit"]["action"],
        "create-only-on-explicit-tracking-request"
    );
    assert!(stdout["blindZones"].as_array().is_some_and(|zones| {
        zones
            .iter()
            .any(|zone| zone.get("area").and_then(|area| area.as_str()) == Some("rs"))
    }));
    Ok(())
}

#[test]
fn cli_manifest_evidence_summary_preserves_current_run_rust_blind_zone_gate() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output_dir = root.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(
        output_dir.join("triage.json"),
        serde_json::to_vec(&json!({ "byLanguage": { "ts": 3, "rs": 2 } }))?,
    )?;
    fs::write(
        output_dir.join("rust-analyzer-health.latest.json"),
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-rust-analyzer.v1",
            "policyVersion": "lumin-rust-analyzer-policy.v1",
            "meta": {
                "producer": "lumin-rust-analyzer",
                "mode": "rust-main",
                "input": { "root": root.path().display().to_string() }
            },
            "summary": { "files": 2, "syntaxReviewSignals": 0 }
        }))?,
    )?;

    let stale_output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-summary")
        .arg("--root")
        .arg(root.path())
        .arg("--output")
        .arg(&output_dir)
        .output()?;
    assert!(
        stale_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&stale_output.stderr)
    );
    let stale = serde_json::from_slice::<serde_json::Value>(&stale_output.stdout)?;
    assert!(stale["blindZones"].as_array().is_some_and(|zones| {
        zones
            .iter()
            .any(|zone| zone.get("area").and_then(|area| area.as_str()) == Some("rs"))
    }));

    let current_output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-summary")
        .arg("--root")
        .arg(root.path())
        .arg("--output")
        .arg(&output_dir)
        .arg("--rust-analysis-ran")
        .output()?;
    assert!(
        current_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&current_output.stderr)
    );
    let current = serde_json::from_slice::<serde_json::Value>(&current_output.stdout)?;
    assert!(!current["blindZones"].as_array().is_some_and(|zones| {
        zones
            .iter()
            .any(|zone| zone.get("area").and_then(|area| area.as_str()) == Some("rs"))
    }));
    assert_eq!(current["rustAnalysis"]["status"], "complete");
    Ok(())
}

#[test]
fn cli_manifest_evidence_summary_merges_rust_analysis_run_block() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output_dir = root.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(
        output_dir.join("triage.json"),
        serde_json::to_vec(&json!({ "byLanguage": { "rs": 1 } }))?,
    )?;
    fs::write(
        output_dir.join("rust-analyzer-health.latest.json"),
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-rust-analyzer.v1",
            "policyVersion": "lumin-rust-analyzer-policy.v1",
            "meta": {
                "producer": "lumin-rust-analyzer",
                "mode": "rust-main",
                "input": { "root": root.path().display().to_string() }
            },
            "summary": { "files": 1, "syntaxReviewSignals": 0 }
        }))?,
    )?;
    let run_path = output_dir.join("rust-run.json");
    fs::write(
        &run_path,
        serde_json::to_vec(&json!({
            "requested": true,
            "ran": true,
            "status": "complete",
            "rustFiles": 1,
            "sourceCommit": "abc123",
            "producer": "lumin-rust-analyzer"
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-summary")
        .arg("--root")
        .arg(root.path())
        .arg("--output")
        .arg(&output_dir)
        .arg("--rust-analysis-run-block")
        .arg(&run_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["rustAnalysis"]["requested"], true);
    assert_eq!(stdout["rustAnalysis"]["ran"], true);
    assert_eq!(stdout["rustAnalysis"]["status"], "complete");
    assert_eq!(stdout["rustAnalysis"]["available"], true);
    assert_eq!(stdout["rustAnalysis"]["files"], 1);
    assert_eq!(stdout["rustAnalysis"]["sourceCommit"], "abc123");
    assert!(!stdout["blindZones"].as_array().is_some_and(|zones| {
        zones
            .iter()
            .any(|zone| zone.get("area").and_then(|area| area.as_str()) == Some("rs"))
    }));
    Ok(())
}

#[test]
fn cli_manifest_evidence_summary_hard_stops_on_malformed_existing_artifact() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output_dir = root.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(output_dir.join("symbols.json"), "{not-json")?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-summary")
        .arg("--root")
        .arg(root.path())
        .arg("--output")
        .arg(&output_dir)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("manifest-evidence-summary: invalid JSON"));
    assert!(stderr.contains("symbols.json"));
    Ok(())
}

#[test]
fn cli_manifest_evidence_summary_degrades_malformed_optional_artifacts() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output_dir = root.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(
        output_dir.join("triage.json"),
        serde_json::to_vec(&json!({ "summary": { "files": 1 } }))?,
    )?;
    fs::write(
        output_dir.join("symbols.json"),
        serde_json::to_vec(&json!({ "uses": { "external": 0 } }))?,
    )?;
    fs::write(output_dir.join("unused-deps.json"), "{not-json")?;
    fs::write(output_dir.join("block-clones.json"), "{not-json")?;
    fs::write(output_dir.join("resolver-diagnostics.json"), "{not-json")?;
    fs::write(
        output_dir.join("framework-resource-surfaces.json"),
        "{not-json",
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-summary")
        .arg("--root")
        .arg(root.path())
        .arg("--output")
        .arg(&output_dir)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["unusedDependencies"]["status"], "unavailable");
    assert_eq!(
        stdout["unusedDependencies"]["reason"]["kind"],
        "malformed-json"
    );
    assert_eq!(stdout["blockClones"]["status"], "unavailable");
    assert_eq!(stdout["blockClones"]["reason"]["kind"], "malformed-json");
    assert_eq!(stdout["frameworkResourceSurfaces"]["status"], "unavailable");
    assert_eq!(stdout["frameworkResourceSurfaces"]["available"], false);
    assert_eq!(
        stdout["frameworkResourceSurfaces"]["reason"]["kind"],
        "malformed-json"
    );
    assert_eq!(
        stdout["frameworkResourceSurfaces"]["totalFilesWithSurfaces"],
        json!(null)
    );
    assert_eq!(stdout["resolverDiagnostics"]["status"], "unavailable");
    assert_eq!(
        stdout["resolverDiagnostics"]["reason"]["kind"],
        "malformed-json"
    );
    assert_eq!(
        stdout["resolverDiagnostics"]["resolverVersion"],
        json!(null)
    );
    Ok(())
}

#[test]
fn cli_manifest_evidence_summary_with_reads_reports_artifact_read_events() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output_dir = root.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(
        output_dir.join("triage.json"),
        serde_json::to_vec(&json!({ "summary": { "files": 1 } }))?,
    )?;
    fs::write(
        output_dir.join("symbols.json"),
        serde_json::to_vec(&json!({ "uses": { "external": 0 } }))?,
    )?;
    fs::write(
        output_dir.join("framework-resource-surfaces.json"),
        "{not-json",
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-summary-with-reads")
        .arg("--root")
        .arg(root.path())
        .arg("--output")
        .arg(&output_dir)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(
        stdout["schemaVersion"],
        "lumin-manifest-evidence-with-artifact-reads.v1"
    );
    assert_eq!(
        stdout["evidence"]["frameworkResourceSurfaces"]["status"],
        "unavailable"
    );
    assert_eq!(
        stdout["artifactReads"]["schemaVersion"],
        "lumin-audit-artifact-read-events.v1"
    );
    let reads = stdout["artifactReads"]["reads"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("artifact reads are serialized as an array"))?;
    assert!(reads.iter().any(|read| {
        read["filePath"]
            .as_str()
            .is_some_and(|path| path.ends_with("triage.json"))
            && read["ok"] == true
    }));
    assert!(reads.iter().any(|read| {
        read["filePath"]
            .as_str()
            .is_some_and(|path| path.ends_with("framework-resource-surfaces.json"))
            && read["ok"] == false
            && read["bytes"].as_u64().unwrap_or(0) > 0
    }));

    let result_path = output_dir.join("manifest-evidence-summary-with-reads-result.json");
    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-summary-with-reads")
        .arg("--root")
        .arg(root.path())
        .arg("--output")
        .arg(&output_dir)
        .arg("--result-output")
        .arg(&result_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "result-output mode should not echo evidence summaries to stdout"
    );
    let result_file = fs::read_to_string(&result_path)?;
    let result_file = serde_json::from_str::<serde_json::Value>(&result_file)?;
    assert_eq!(
        result_file["evidence"]["frameworkResourceSurfaces"]["status"],
        "unavailable"
    );
    assert!(result_file["artifactReads"]["reads"]
        .as_array()
        .is_some_and(
            |result_reads| result_reads.iter().any(|read| read["filePath"]
                .as_str()
                .is_some_and(|path| path.ends_with("triage.json")))
        ));
    Ok(())
}

#[test]
fn cli_manifest_evidence_refresh_emits_manifest_patch_shape() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output_dir = root.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(
        output_dir.join("triage.json"),
        serde_json::to_vec(&json!({
            "shape": {
                "totalFiles": 2,
                "tsFiles": 1,
                "rsFiles": 1
            }
        }))?,
    )?;
    fs::write(
        output_dir.join("symbols.json"),
        serde_json::to_vec(&json!({
            "uses": {
                "external": 0,
                "resolvedInternal": 0,
                "unresolvedInternal": 0,
                "unresolvedInternalRatio": 0
            }
        }))?,
    )?;
    fs::write(
        output_dir.join("framework-resource-surfaces.json"),
        "{not-json",
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-refresh")
        .arg("--root")
        .arg(root.path())
        .arg("--output")
        .arg(&output_dir)
        .arg("--no-include-tests")
        .arg("--production")
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["scanRange"]["files"], 2);
    assert_eq!(stdout["scanRange"]["includeTests"], false);
    assert_eq!(stdout["scanRange"]["production"], true);
    assert!(stdout["blindZones"].is_array());
    assert_eq!(stdout["frameworkResourceSurfaces"]["status"], "unavailable");
    assert_eq!(stdout["frameworkResourceSurfaces"]["available"], false);
    assert_eq!(
        stdout["frameworkResourceSurfaces"]["reason"]["kind"],
        "malformed-json"
    );
    assert_eq!(
        stdout["frameworkResourceSurfaces"]["totalFilesWithSurfaces"],
        json!(null)
    );

    let result_path = output_dir.join("manifest-evidence-refresh-with-reads-result.json");
    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-refresh-with-reads")
        .arg("--root")
        .arg(root.path())
        .arg("--output")
        .arg(&output_dir)
        .arg("--no-include-tests")
        .arg("--production")
        .arg("--result-output")
        .arg(&result_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "result-output mode should not echo evidence refresh patches to stdout"
    );
    let result_file = fs::read_to_string(&result_path)?;
    let result_file = serde_json::from_str::<serde_json::Value>(&result_file)?;
    assert_eq!(result_file["evidence"]["scanRange"]["files"], 2);
    assert_eq!(result_file["evidence"]["scanRange"]["production"], true);
    assert!(result_file["artifactReads"]["reads"]
        .as_array()
        .is_some_and(
            |result_reads| result_reads.iter().any(|read| read["filePath"]
                .as_str()
                .is_some_and(|path| path.ends_with("triage.json")))
        ));
    Ok(())
}

#[test]
fn cli_manifest_lifecycle_evidence_refresh_applies_updates_to_manifest() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output_dir = root.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(
        output_dir.join("triage.json"),
        serde_json::to_vec(&json!({
            "shape": {
                "totalFiles": 2,
                "tsFiles": 1,
                "rsFiles": 1
            },
            "byLanguage": { "rs": 1 }
        }))?,
    )?;
    fs::write(
        output_dir.join("symbols.json"),
        serde_json::to_vec(&json!({
            "uses": {
                "external": 0,
                "resolvedInternal": 0,
                "unresolvedInternal": 0,
                "unresolvedInternalRatio": 0
            }
        }))?,
    )?;
    let input_path = output_dir.join("request.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "manifest": {
                "meta": { "generated": "2026-07-02T00:00:00.000Z" },
                "artifactsProduced": []
            },
            "lifecycle": {
                "preWrite": {
                    "requested": true,
                    "ran": true,
                    "engine": "rust",
                    "language": "rust"
                }
            },
            "evidence": {
                "root": root.path().display().to_string(),
                "output": output_dir,
                "includeTests": false,
                "production": true,
                "generatedArtifactsMode": "default"
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-lifecycle-evidence-refresh")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["manifest"]["preWrite"]["engine"], "rust");
    assert_eq!(stdout["manifest"]["lifecycle"]["ranCount"], 1);
    assert_eq!(stdout["manifest"]["scanRange"]["files"], 2);
    assert_eq!(stdout["manifest"]["scanRange"]["includeTests"], false);
    assert_eq!(stdout["manifest"]["scanRange"]["production"], true);
    assert!(stdout["artifactReads"]["reads"]
        .as_array()
        .is_some_and(|reads| {
            reads.iter().any(|read| {
                read["filePath"]
                    .as_str()
                    .is_some_and(|path| path.ends_with("triage.json"))
                    && read["ok"] == true
            })
        }));

    let result_path = output_dir.join("manifest-lifecycle-evidence-refresh-result.json");
    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-lifecycle-evidence-refresh")
        .arg("--input")
        .arg(&input_path)
        .arg("--result-output")
        .arg(&result_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "result-output mode should not echo refreshed manifests to stdout"
    );
    let result_file = fs::read_to_string(&result_path)?;
    let result_file = serde_json::from_str::<serde_json::Value>(&result_file)?;
    assert_eq!(result_file["manifest"]["lifecycle"]["ranCount"], 1);
    assert_eq!(result_file["manifest"]["scanRange"]["files"], 2);
    Ok(())
}
