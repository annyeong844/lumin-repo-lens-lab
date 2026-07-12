use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

use lumin_audit_core::artifact_registry::{
    collect_produced_artifacts, collect_produced_artifacts_for_manifest,
    rust_analysis_artifact_usable,
};
use lumin_audit_core::rust_analysis::{
    merge_rust_analysis_run, summarize_rust_analysis_artifact, RustAnalysisRunMergeInput,
    RustAnalysisStatus,
};

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
        "rust-pre-write-artifact.INV-1.json",
        "rust-pre-write-artifact.latest.json",
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
            "rust-pre-write-artifact.INV-1.json",
            "rust-pre-write-artifact.latest.json",
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
        "rust-pre-write-artifact.json",
        "rust-pre-write-artifact.txt",
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
fn rust_analyzer_artifact_is_produced_only_when_manifest_block_is_usable() -> Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(temp.path().join("rust-analyzer-health.latest.json"), "{}\n")?;
    let complete = json!({ "status": "complete", "available": true });
    let unavailable = json!({ "status": "artifact-unavailable", "available": false });

    assert!(rust_analysis_artifact_usable(Some(&complete)));
    assert!(!rust_analysis_artifact_usable(Some(&unavailable)));
    assert_eq!(
        collect_produced_artifacts_for_manifest(temp.path(), Some(&complete))?,
        names(&["rust-analyzer-health.latest.json"])
    );
    assert!(collect_produced_artifacts_for_manifest(temp.path(), Some(&unavailable))?.is_empty());
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
fn malformed_rust_analysis_artifact_preserves_unavailable_reason() -> Result<()> {
    let root = tempfile::tempdir()?;
    let artifact = json!({
        "artifact": "rust-analyzer-health.latest.json",
        "status": "unavailable",
        "reason": {
            "kind": "malformed-json",
            "message": "expected value"
        }
    });

    let summary = summarize_rust_analysis_artifact(root.path(), &artifact)
        .ok_or_else(|| anyhow::anyhow!("synthetic unavailable artifact should yield summary"))?;

    assert_eq!(summary.status, RustAnalysisStatus::Unavailable);
    assert!(!summary.available);
    assert_eq!(
        serde_json::to_value(summary)?,
        json!({
            "artifact": "rust-analyzer-health.latest.json",
            "status": "unavailable",
            "available": false,
            "root": null,
            "reason": {
                "kind": "malformed-json",
                "message": "expected value"
            }
        })
    );
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

#[test]
fn rust_analysis_run_merge_preserves_js_contract_branches() -> Result<()> {
    let complete = merged_rust_analysis(json!({
        "evidence": {
            "artifact": "rust-analyzer-health.latest.json",
            "status": "complete",
            "available": true,
            "files": 3
        },
        "run": {
            "requested": true,
            "ran": true,
            "status": "complete",
            "rustFiles": 3,
            "artifact": "rust-analyzer-health.latest.json",
            "path": "C:/repo/.audit/rust-analyzer-health.latest.json",
            "sourceCommit": "abc123",
            "producer": "lumin-rust-analyzer",
            "analyzerInvocation": { "source": "cargo-run" },
            "futureRunField": { "kept": true }
        }
    }))?;
    assert_eq!(complete["status"], "complete");
    assert_eq!(complete["available"], true);
    assert_eq!(complete["files"], 3);
    assert_eq!(complete["sourceCommit"], "abc123");
    assert_eq!(complete["analyzerInvocation"]["source"], "cargo-run");
    assert_eq!(complete["futureRunField"]["kept"], true);

    let artifact_unavailable = merged_rust_analysis(json!({
        "evidence": {
            "artifact": "rust-analyzer-health.latest.json",
            "status": "invalid-shape"
        },
        "run": {
            "requested": true,
            "ran": true,
            "status": "complete",
            "rustFiles": 3,
            "artifact": "rust-analyzer-health.latest.json"
        }
    }))?;
    assert_eq!(artifact_unavailable["status"], "artifact-unavailable");
    assert_eq!(artifact_unavailable["available"], false);
    assert_eq!(artifact_unavailable["artifactStatus"], "invalid-shape");
    assert_eq!(
        artifact_unavailable["artifact"],
        "rust-analyzer-health.latest.json"
    );

    let skipped = merged_rust_analysis(json!({
        "evidence": null,
        "run": {
            "requested": true,
            "ran": false,
            "status": "skipped",
            "rustFiles": 0,
            "reason": "no Rust files counted by triage"
        }
    }))?;
    assert_eq!(skipped["status"], "skipped");
    assert_eq!(skipped["reason"], "no Rust files counted by triage");

    let not_requested = merged_rust_analysis(json!({
        "evidence": {
            "artifact": "rust-analyzer-health.latest.json",
            "status": "complete"
        },
        "run": {
            "requested": false,
            "ran": false,
            "status": "not-requested",
            "rustFiles": 7
        }
    }))?;
    assert_eq!(not_requested["requested"], false);
    assert_eq!(not_requested["ran"], false);
    assert_eq!(not_requested["status"], "not-requested");
    assert_eq!(not_requested["rustFiles"], 7);
    assert_eq!(not_requested["artifactStatus"], "complete");

    let missing_run = merged_rust_analysis(json!({ "evidence": null }))?;
    assert_eq!(missing_run["status"], "not-requested");
    assert_eq!(missing_run["rustFiles"], 0);
    Ok(())
}

fn merged_rust_analysis(input: serde_json::Value) -> Result<serde_json::Value> {
    let input = serde_json::from_value::<RustAnalysisRunMergeInput>(input)?;
    merge_rust_analysis_run(input)
}

#[test]
fn rust_analysis_run_merge_rejects_empty_run_status() -> Result<()> {
    let input = serde_json::from_value::<RustAnalysisRunMergeInput>(json!({
        "evidence": null,
        "run": {
            "requested": true,
            "ran": false,
            "status": " "
        }
    }))?;
    let Err(error) = merge_rust_analysis_run(input) else {
        panic!("empty rust analysis run status should hard-stop");
    };
    assert!(error
        .to_string()
        .contains("rust-analysis-run-merge: run.status must be a non-empty string"));
    Ok(())
}

#[test]
fn cli_rust_analysis_run_merge_reads_stdin_json() -> Result<()> {
    let input = json!({
        "evidence": {
            "artifact": "rust-analyzer-health.latest.json",
            "status": "complete",
            "available": true,
            "files": 2
        },
        "run": {
            "requested": true,
            "ran": true,
            "status": "complete",
            "rustFiles": 2,
            "artifact": "rust-analyzer-health.latest.json"
        }
    });

    let child = Command::new(audit_core_bin())
        .arg("rust-analysis-run-merge")
        .arg("--input")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let output = write_child_stdin_and_wait(child, &input.to_string())?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["status"], "complete");
    assert_eq!(stdout["available"], true);
    assert_eq!(stdout["rustFiles"], 2);
    Ok(())
}

#[test]
fn cli_js_ts_pre_write_writes_shared_evidence_to_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let source = temp.path().join("app.ts");
    let result_path = temp.path().join("result.json");
    fs::write(&source, "export const value = input as any;\n")?;
    let request = json!({
        "schemaVersion": "lumin-js-ts-pre-write-evidence-request.v1",
        "root": temp.path(),
        "evidenceArtifact": "pre-write-evidence.PROBE.json",
        "anyInventoryArtifact": "any-inventory.pre.PROBE.json",
        "generated": "2026-07-11T00:00:00.000Z",
        "includeTests": true,
        "excludes": [],
        "dependencyRoots": [],
        "files": [{
            "filePath": source,
            "artifactFilePath": "app.ts"
        }]
    });
    let child = Command::new(audit_core_bin())
        .arg("js-ts-pre-write-evidence")
        .arg("--input")
        .arg("-")
        .arg("--result-output")
        .arg(&result_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let output = write_child_stdin_and_wait(child, &request.to_string())?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    let result = serde_json::from_slice::<serde_json::Value>(&fs::read(result_path)?)?;
    assert_eq!(
        result["anyInventory"]["typeEscapes"][0]["escapeKind"],
        "as-any"
    );
    assert_eq!(
        result["summary"]["runtime"]["singleFlight"]["status"],
        "acquired"
    );
    assert!(result["summary"]["runtime"]["timing"]["scanHeldMs"].is_u64());
    Ok(())
}

#[test]
fn cli_artifact_registry_emits_stdout_json() -> Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(temp.path().join("symbols.json"), "{}\n")?;
    fs::write(temp.path().join("pre-write-evidence.PROBE.json"), "{}\n")?;
    fs::write(temp.path().join("pre-write-evidence.latest.json"), "{}\n")?;
    fs::write(temp.path().join("rust-analyzer-health.latest.json"), "{}\n")?;

    let output = Command::new(audit_core_bin())
        .arg("artifact-registry")
        .arg("--output")
        .arg(temp.path())
        .output()?;

    assert!(output.status.success());
    let artifacts = serde_json::from_slice::<Vec<String>>(&output.stdout)?;
    assert_eq!(
        artifacts,
        names(&[
            "pre-write-evidence.PROBE.json",
            "pre-write-evidence.latest.json",
            "symbols.json",
        ])
    );
    Ok(())
}

#[test]
fn cli_artifact_registry_can_include_current_rust_analysis_artifact() -> Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(temp.path().join("rust-analyzer-health.latest.json"), "{}\n")?;

    let output = Command::new(audit_core_bin())
        .arg("artifact-registry")
        .arg("--output")
        .arg(temp.path())
        .arg("--rust-analysis-ran")
        .output()?;

    assert!(output.status.success());
    let artifacts = serde_json::from_slice::<Vec<String>>(&output.stdout)?;
    assert_eq!(artifacts, names(&["rust-analyzer-health.latest.json"]));
    Ok(())
}

#[test]
fn cli_artifact_registry_uses_rust_analysis_block_for_current_artifact() -> Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(temp.path().join("rust-analyzer-health.latest.json"), "{}\n")?;

    let unavailable = Command::new(audit_core_bin())
        .arg("artifact-registry")
        .arg("--output")
        .arg(temp.path())
        .arg("--rust-analysis-block")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let unavailable = write_child_stdin_and_wait(
        unavailable,
        &json!({ "status": "artifact-unavailable", "available": false }).to_string(),
    )?;

    assert!(
        unavailable.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&unavailable.stderr)
    );
    let artifacts = serde_json::from_slice::<Vec<String>>(&unavailable.stdout)?;
    assert!(artifacts.is_empty());

    let complete = Command::new(audit_core_bin())
        .arg("artifact-registry")
        .arg("--output")
        .arg(temp.path())
        .arg("--rust-analysis-block")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let complete = write_child_stdin_and_wait(
        complete,
        &json!({ "status": "complete", "available": true }).to_string(),
    )?;

    assert!(
        complete.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&complete.stderr)
    );
    let artifacts = serde_json::from_slice::<Vec<String>>(&complete.stdout)?;
    assert_eq!(artifacts, names(&["rust-analyzer-health.latest.json"]));
    Ok(())
}

#[test]
fn cli_rust_analysis_summary_emits_complete_json() -> Result<()> {
    let root = tempfile::tempdir()?;
    let root_text = root.path().to_string_lossy().to_string();
    let artifact = root.path().join("rust-analyzer-health.latest.json");
    fs::write(
        &artifact,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-rust-analyzer.v1",
            "policyVersion": "lumin-rust-analyzer-policy.v1",
            "meta": {
                "producer": "lumin-rust-analyzer",
                "mode": "rust-main",
                "input": { "root": root_text }
            },
            "summary": { "files": 3, "syntaxReviewSignals": 2 }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("rust-analysis-summary")
        .arg("--root")
        .arg(root.path())
        .arg("--artifact")
        .arg(&artifact)
        .output()?;

    assert!(output.status.success());
    let summary = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(summary["status"], "complete");
    assert_eq!(summary["available"], true);
    assert_eq!(summary["files"], 3);
    assert_eq!(summary["syntaxReviewSignals"], 2);
    Ok(())
}

#[test]
fn cli_unknown_argument_hard_stops() -> Result<()> {
    let output = Command::new(audit_core_bin())
        .arg("artifact-registry")
        .arg("--bogus")
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown argument"));
    Ok(())
}

fn names(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}

fn write_child_stdin_and_wait(
    mut child: std::process::Child,
    stdin_text: &str,
) -> Result<std::process::Output> {
    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "stdin pipe missing")
        })?;
        stdin.write_all(stdin_text.as_bytes())?;
    }
    Ok(child.wait_with_output()?)
}
