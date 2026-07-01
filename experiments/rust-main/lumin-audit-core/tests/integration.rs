use anyhow::Result;
use serde_json::json;
use std::fs;

use lumin_audit_core::artifact_registry::collect_produced_artifacts;
use lumin_audit_core::rust_analysis::{summarize_rust_analysis_artifact, RustAnalysisStatus};

#[test]
fn produced_artifacts_include_static_and_dynamic_names_in_order() -> Result<()> {
    let temp = tempfile::tempdir()?;
    for name in [
        "symbols.json",
        "pre-write-advisory.json",
        "pre-write-advisory.abc.json",
        "canon-drift.type-ownership.md",
        "post-write-delta.json",
        "post-write-delta.xyz.json",
        "any-inventory.pre.123.json",
        "any-inventory.post.456.json",
        "audit-summary.latest.md",
    ] {
        fs::write(temp.path().join(name), "{}\n")?;
    }

    let artifacts = collect_produced_artifacts(temp.path(), true)?;

    assert_eq!(
        artifacts,
        names(&[
            "any-inventory.post.456.json",
            "any-inventory.pre.123.json",
            "audit-summary.latest.md",
            "canon-drift.type-ownership.md",
            "post-write-delta.json",
            "post-write-delta.xyz.json",
            "pre-write-advisory.abc.json",
            "pre-write-advisory.json",
            "symbols.json",
        ])
    );
    Ok(())
}

#[test]
fn malformed_dynamic_artifact_names_are_not_reported() -> Result<()> {
    let temp = tempfile::tempdir()?;
    for name in [
        "canon-drift.md",
        "pre-write-advisory.txt",
        "post-write-delta.txt",
        "any-inventory.pre.json",
        "any-inventory.post.json",
    ] {
        fs::write(temp.path().join(name), "{}\n")?;
    }

    let artifacts = collect_produced_artifacts(temp.path(), true)?;

    assert!(artifacts.is_empty());
    Ok(())
}

#[test]
fn missing_output_directory_reports_no_artifacts() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let missing = temp.path().join("missing-output");

    let artifacts = collect_produced_artifacts(&missing, true)?;

    assert!(artifacts.is_empty());
    Ok(())
}

#[test]
fn stale_rust_analyzer_artifact_is_not_produced_when_current_run_did_not_use_it() -> Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(temp.path().join("rust-analyzer-health.latest.json"), "{}\n")?;

    let artifacts = collect_produced_artifacts(temp.path(), false)?;

    assert!(!artifacts.contains(&"rust-analyzer-health.latest.json".to_string()));
    Ok(())
}

#[test]
fn current_rust_analyzer_artifact_is_produced_when_current_run_used_it() -> Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(temp.path().join("rust-analyzer-health.latest.json"), "{}\n")?;

    let artifacts = collect_produced_artifacts(temp.path(), true)?;

    assert_eq!(artifacts, names(&["rust-analyzer-health.latest.json"]));
    Ok(())
}

#[test]
fn non_object_rust_analysis_artifact_has_no_summary() -> Result<()> {
    let root = tempfile::tempdir()?;

    let summary = summarize_rust_analysis_artifact(root.path(), &json!(null));

    assert!(summary.is_none());
    Ok(())
}

#[test]
fn rust_analysis_summary_reports_root_mismatch() -> Result<()> {
    let root = tempfile::tempdir()?;
    let other = tempfile::tempdir()?;
    let other_root = other.path().to_string_lossy().to_string();
    let artifact = json!({
        "schemaVersion": "lumin-rust-analyzer.v1",
        "policyVersion": "lumin-rust-analyzer-policy.v1",
        "meta": {
            "producer": "lumin-rust-analyzer",
            "mode": "rust-main",
            "input": { "root": other_root }
        },
        "summary": { "files": 1 }
    });

    let summary = summarize_rust_analysis_artifact(root.path(), &artifact)
        .ok_or_else(|| anyhow::anyhow!("object artifact should yield a summary"))?;

    assert_eq!(summary.status, RustAnalysisStatus::RootMismatch);
    assert!(!summary.available);
    assert_eq!(
        serde_json::to_value(summary)?,
        json!({
            "artifact": "rust-analyzer-health.latest.json",
            "status": "root-mismatch",
            "available": false,
            "root": other.path().to_string_lossy()
        })
    );
    Ok(())
}

#[test]
fn rust_analysis_summary_reports_invalid_shape() -> Result<()> {
    let root = tempfile::tempdir()?;
    let root_text = root.path().to_string_lossy().to_string();
    let artifact = json!({
        "schemaVersion": "lumin-rust-analyzer.v1",
        "meta": {
            "producer": "lumin-rust-analyzer",
            "mode": "rust-main",
            "input": { "root": root_text }
        },
        "summary": {}
    });

    let summary = summarize_rust_analysis_artifact(root.path(), &artifact)
        .ok_or_else(|| anyhow::anyhow!("object artifact should yield a summary"))?;

    assert_eq!(summary.status, RustAnalysisStatus::InvalidShape);
    assert!(!summary.available);
    assert_eq!(
        serde_json::to_value(summary)?,
        json!({
            "artifact": "rust-analyzer-health.latest.json",
            "status": "invalid-shape",
            "available": false,
            "root": root_text
        })
    );
    Ok(())
}

#[test]
fn rust_analysis_summary_preserves_complete_scope_and_counts() -> Result<()> {
    let root = tempfile::tempdir()?;
    let root_text = root.path().to_string_lossy().to_string();
    let artifact = json!({
        "schemaVersion": "lumin-rust-analyzer.v1",
        "policyVersion": "lumin-rust-analyzer-policy.v1",
        "meta": {
            "producer": "lumin-rust-analyzer",
            "mode": "rust-main",
            "input": {
                "root": root_text,
                "effectiveSourceHealthProfile": "compact",
                "sourceHealthProfile": "full",
                "semanticMode": { "kind": "metadata-only" },
                "includeTests": false,
                "exclude": ["generated", 42]
            }
        },
        "phases": {
            "syntax": {
                "meta": {
                    "input": {
                        "pathPolicy": {
                            "exclude": ["**/target/**", "generated"]
                        }
                    }
                }
            }
        },
        "summary": {
            "files": 2,
            "syntaxReviewSignals": 3,
            "syntaxReviewOpaqueSurfaces": 4,
            "syntaxFunctionCloneExactBodyGroups": 5,
            "syntaxFunctionCloneStructureGroups": 6,
            "syntaxFunctionCloneSignatureGroups": 7,
            "syntaxFunctionCloneNearCandidates": 8,
            "actionTierSummary": { "safeFix": 1 },
            "oracleBridgeStatus": "metadata-only"
        }
    });

    let summary = summarize_rust_analysis_artifact(root.path(), &artifact)
        .ok_or_else(|| anyhow::anyhow!("object artifact should yield a summary"))?;

    assert_eq!(summary.status, RustAnalysisStatus::Complete);
    assert!(summary.available);
    assert_eq!(summary.files, 2);
    assert_eq!(summary.syntax_review_signals, 3);
    assert_eq!(
        summary
            .scan_scope
            .as_ref()
            .and_then(|scope| scope.include_tests),
        Some(false)
    );
    assert_eq!(
        summary
            .scan_scope
            .as_ref()
            .and_then(|scope| scope.exclude.clone()),
        Some(names(&["generated"]))
    );
    assert_eq!(
        serde_json::to_value(summary)?,
        json!({
            "artifact": "rust-analyzer-health.latest.json",
            "status": "complete",
            "available": true,
            "schemaVersion": "lumin-rust-analyzer.v1",
            "policyVersion": "lumin-rust-analyzer-policy.v1",
            "producer": "lumin-rust-analyzer",
            "mode": "rust-main",
            "sourceHealthProfile": "compact",
            "semanticMode": { "kind": "metadata-only" },
            "scanScope": {
                "includeTests": false,
                "exclude": ["generated"],
                "pathPolicy": {
                    "exclude": ["**/target/**", "generated"]
                }
            },
            "files": 2,
            "syntaxReviewSignals": 3,
            "syntaxReviewOpaqueSurfaces": 4,
            "syntaxFunctionCloneExactBodyGroups": 5,
            "syntaxFunctionCloneStructureGroups": 6,
            "syntaxFunctionCloneSignatureGroups": 7,
            "syntaxFunctionCloneNearCandidates": 8,
            "actionTierSummary": { "safeFix": 1 },
            "oracleBridgeStatus": "metadata-only"
        })
    );
    Ok(())
}

#[test]
fn rust_analysis_summary_uses_syntax_scan_scope_fallback() -> Result<()> {
    let root = tempfile::tempdir()?;
    let root_text = root.path().to_string_lossy().to_string();
    let artifact = json!({
        "schemaVersion": "lumin-rust-analyzer.v1",
        "policyVersion": "lumin-rust-analyzer-policy.v1",
        "meta": {
            "producer": "lumin-rust-analyzer",
            "mode": "rust-main",
            "input": { "root": root_text }
        },
        "phases": {
            "syntax": {
                "meta": {
                    "input": {
                        "includeTests": true,
                        "exclude": ["fixtures"],
                        "pathPolicy": { "mode": "syntax" }
                    }
                }
            }
        },
        "summary": { "files": 1 }
    });

    let summary = summarize_rust_analysis_artifact(root.path(), &artifact)
        .ok_or_else(|| anyhow::anyhow!("object artifact should yield a summary"))?;

    assert_eq!(summary.status, RustAnalysisStatus::Complete);
    assert_eq!(
        serde_json::to_value(summary)?.get("scanScope").cloned(),
        Some(json!({
            "includeTests": true,
            "exclude": ["fixtures"],
            "pathPolicy": { "mode": "syntax" }
        }))
    );
    Ok(())
}

fn names(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}
