use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use lumin_audit_core::manifest_root::{
    build_manifest_evidence_update, build_manifest_root, ManifestEvidenceUpdateInput,
    ManifestRootInput,
};

#[test]
fn manifest_root_projects_typed_runtime_log_and_places_rust_owned_fields() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(output_dir.join("triage.json"), "{}")?;
    fs::write(output_dir.join("symbols.json"), "{}")?;
    fs::write(output_dir.join("rust-analyzer-health.latest.json"), "{}")?;

    let root = serde_json::from_value::<ManifestRootInput>(json!({
        "generated": "2026-07-01T00:00:00.000Z",
        "profile": "full",
        "root": "C:/repo",
        "output": output_dir,
        "commandsRun": [
            {
                "step": "measure-topology.mjs",
                "status": "ok",
                "ms": 12,
                "memory": {
                    "before": { "rssBytes": 1000 },
                    "after": { "rssBytes": 1400 },
                    "delta": { "rssBytes": 400 }
                }
            },
            {
                "step": "lumin-rust-analyzer",
                "status": "ok",
                "ms": 34,
                "artifact": "rust-analyzer-health.latest.json",
                "rustFiles": 8,
                "analyzerInvocation": { "source": "cargo-run", "manifestPath": "experiments/Cargo.toml" }
            },
            {
                "step": "resolve-method-calls.mjs",
                "status": "failed-optional",
                "ms": 4,
                "stderr": "resolver diagnostics unavailable"
            }
        ],
        "skipped": [
            { "step": "emit-sarif.mjs", "reason": "not in --sarif mode" }
        ],
        "evidence": {
            "scanRange": { "root": "C:/repo", "includeTests": true },
            "confidence": { "parseErrors": 0, "unresolvedInternal": 2 },
            "blindZones": [
                {
                    "area": "rs",
                    "severity": "scan-gap",
                    "effect": "read rust artifact first",
                    "details": { "files": 2 }
                }
            ],
            "rustAnalysis": {
                "requested": true,
                "ran": true,
                "status": "complete",
                "available": true
            },
            "generatedArtifacts": { "mode": "default", "status": "complete" },
            "resolverDiagnostics": { "resolverVersion": "resolver-v1" },
            "frameworkResourceSurfaces": { "artifact": "framework-resource-surfaces.json" },
            "unusedDependencies": { "artifact": "unused-deps.json" },
            "blockClones": { "artifact": "block-clones.json" },
            "sfcEvidence": { "status": "complete" },
            "livingAudit": { "action": "read-existing" }
        }
    }))?;
    let manifest = serde_json::to_value(build_manifest_root(root)?)?;

    assert_eq!(manifest["meta"]["tool"], "audit-repo.mjs");
    assert_eq!(manifest["meta"]["profile"], "full");
    assert_eq!(manifest["profile"], "full");
    assert_eq!(manifest["commandsRun"][0]["step"], "measure-topology.mjs");
    assert_eq!(
        manifest["commandsRun"][0]["memory"]["delta"]["rssBytes"],
        400
    );
    assert_eq!(
        manifest["commandsRun"][1]["artifact"],
        "rust-analyzer-health.latest.json"
    );
    assert_eq!(manifest["commandsRun"][1]["rustFiles"], 8);
    assert_eq!(
        manifest["commandsRun"][1]["analyzerInvocation"]["manifestPath"],
        "experiments/Cargo.toml"
    );
    assert_eq!(manifest["commandsRun"][2]["status"], "failed-optional");
    assert_eq!(
        manifest["commandsRun"][2]["stderr"],
        "resolver diagnostics unavailable"
    );
    assert_eq!(manifest["skipped"][0]["reason"], "not in --sarif mode");
    assert_eq!(manifest["blindZones"][0]["area"], "rs");
    assert_eq!(manifest["blindZones"][0]["details"]["files"], 2);
    assert_eq!(manifest["rustAnalysis"]["available"], true);
    assert_eq!(
        manifest["resolverDiagnostics"]["resolverVersion"],
        "resolver-v1"
    );
    assert_eq!(
        manifest["frameworkResourceSurfaces"]["artifact"],
        "framework-resource-surfaces.json"
    );
    assert_eq!(
        manifest["unusedDependencies"]["artifact"],
        "unused-deps.json"
    );
    assert_eq!(manifest["blockClones"]["artifact"], "block-clones.json");
    assert_eq!(manifest["sfcEvidence"]["status"], "complete");
    assert_eq!(
        manifest["artifactsProduced"],
        json!([
            "rust-analyzer-health.latest.json",
            "symbols.json",
            "triage.json"
        ])
    );
    Ok(())
}

#[test]
fn manifest_root_uses_rust_analysis_block_for_current_rust_artifact() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(output_dir.join("triage.json"), "{}")?;
    fs::write(output_dir.join("rust-analyzer-health.latest.json"), "{}")?;

    let base = json!({
        "generated": "2026-07-01T00:00:00.000Z",
        "profile": "quick",
        "root": "C:/repo",
        "output": output_dir,
        "commandsRun": [],
        "skipped": [],
        "evidence": {
            "scanRange": {},
            "confidence": {},
            "resolverDiagnostics": {},
            "blindZones": [],
            "rustAnalysis": {
                "requested": true,
                "ran": true,
                "status": "skipped",
                "available": false
            },
            "generatedArtifacts": {},
            "frameworkResourceSurfaces": null,
            "unusedDependencies": null,
            "blockClones": null,
            "sfcEvidence": {},
            "livingAudit": {}
        },
        "artifactsProduced": [
            "rust-analyzer-health.latest.json",
            "legacy-js-supplied-value.json"
        ]
    });
    let unavailable = serde_json::to_value(build_manifest_root(serde_json::from_value(base)?)?)?;
    assert_eq!(unavailable["artifactsProduced"], json!(["triage.json"]));

    let available = serde_json::to_value(build_manifest_root(serde_json::from_value(json!({
        "generated": "2026-07-01T00:00:00.000Z",
        "profile": "quick",
        "root": "C:/repo",
        "output": output_dir,
        "commandsRun": [],
        "skipped": [],
        "evidence": {
            "scanRange": {},
            "confidence": {},
            "resolverDiagnostics": {},
            "blindZones": [],
            "rustAnalysis": {
                "requested": true,
                "ran": true,
                "status": "complete",
                "available": true
            },
            "generatedArtifacts": {},
            "frameworkResourceSurfaces": null,
            "unusedDependencies": null,
            "blockClones": null,
            "sfcEvidence": {},
            "livingAudit": {}
        }
    }))?)?)?;
    assert_eq!(
        available["artifactsProduced"],
        json!(["rust-analyzer-health.latest.json", "triage.json"])
    );
    Ok(())
}

#[test]
fn cli_manifest_root_hard_stops_on_malformed_runtime_log() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("manifest-root.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "generated": "2026-07-01T00:00:00.000Z",
            "profile": "quick",
            "root": "C:/repo",
            "output": "C:/repo/.audit",
            "commandsRun": [
                { "step": "measure-topology.mjs", "ms": 12 }
            ],
            "skipped": [
                { "step": "emit-sarif.mjs", "reason": "not in --sarif mode" }
            ],
            "evidence": {
                "scanRange": {},
                "confidence": {},
                "blindZones": [],
                "rustAnalysis": null,
                "generatedArtifacts": {},
                "resolverDiagnostics": {},
                "frameworkResourceSurfaces": null,
                "unusedDependencies": null,
                "blockClones": null,
                "sfcEvidence": null,
                "livingAudit": {}
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-root")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("manifest-root: invalid request shape"));
    Ok(())
}

#[test]
fn manifest_root_rejects_empty_skipped_reason() -> Result<()> {
    let input = serde_json::from_value::<ManifestRootInput>(json!({
        "generated": "2026-07-01T00:00:00.000Z",
        "profile": "quick",
        "root": "C:/repo",
        "output": "C:/repo/.audit",
        "commandsRun": [],
        "skipped": [
            { "step": "emit-sarif.mjs", "reason": "  " }
        ],
        "evidence": {
            "scanRange": {},
            "confidence": {},
            "blindZones": [],
            "rustAnalysis": null,
            "generatedArtifacts": {},
            "resolverDiagnostics": {},
            "frameworkResourceSurfaces": null,
            "unusedDependencies": null,
            "blockClones": null,
            "sfcEvidence": null,
            "livingAudit": {}
        }
    }))?;

    let Err(error) = build_manifest_root(input) else {
        panic!("empty skipped reason should hard-stop");
    };
    assert!(error
        .to_string()
        .contains("manifest-root: skipped[].reason must be a non-empty string"));
    Ok(())
}

#[test]
fn cli_manifest_root_reads_stdin_json() -> Result<()> {
    let input = json!({
        "generated": "2026-07-01T00:00:00.000Z",
        "profile": "quick",
        "root": "C:/repo",
        "output": "C:/repo/.audit",
        "commandsRun": [],
        "skipped": [],
        "evidence": {
            "scanRange": { "root": "C:/repo" },
            "confidence": { "parseErrors": 0 },
            "blindZones": [],
            "rustAnalysis": { "requested": false, "ran": false, "status": "not-requested" },
            "generatedArtifacts": { "mode": "default" },
            "resolverDiagnostics": {},
            "frameworkResourceSurfaces": null,
            "unusedDependencies": null,
            "blockClones": null,
            "sfcEvidence": null,
            "livingAudit": { "action": "create-only-on-explicit-tracking-request" }
        },
        "artifactsProduced": []
    });

    let mut child = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-root")
        .arg("--input")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "stdin pipe missing")
        })?;
        stdin.write_all(input.to_string().as_bytes())?;
    }
    let output = child.wait_with_output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<Value>(&output.stdout)?;
    assert_eq!(stdout["profile"], "quick");
    assert_eq!(stdout["blindZones"], json!([]));
    Ok(())
}

#[test]
fn cli_manifest_root_hard_stops_on_invalid_profile() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("manifest-root.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "generated": "2026-07-01T00:00:00.000Z",
            "profile": "debug",
            "root": "C:/repo",
            "output": "C:/repo/.audit",
            "evidence": {
                "scanRange": {},
                "confidence": {},
                "blindZones": [],
                "rustAnalysis": null,
                "generatedArtifacts": {},
                "resolverDiagnostics": {},
                "frameworkResourceSurfaces": null,
                "unusedDependencies": null,
                "blockClones": null,
                "sfcEvidence": null,
                "livingAudit": {}
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-root")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("manifest-root: invalid --profile"));
    Ok(())
}

#[test]
fn manifest_evidence_update_projects_all_refresh_fields_without_reinterpreting_blind_zones(
) -> Result<()> {
    let input = serde_json::from_value::<ManifestEvidenceUpdateInput>(json!({
        "evidence": {
            "scanRange": { "includeTests": true, "production": false },
            "confidence": { "parseErrors": 0 },
            "resolverDiagnostics": { "status": "available" },
            "blindZones": [
                {
                    "area": "resolver",
                    "severity": "precision-gap",
                    "effect": "JS producer owns this meaning"
                }
            ],
            "rustAnalysis": { "status": "complete" },
            "generatedArtifacts": { "mode": "present" },
            "frameworkResourceSurfaces": { "status": "available" },
            "unusedDependencies": { "reviewUnused": 2 },
            "blockClones": { "groups": 1 },
            "sfcEvidence": { "files": 3 },
            "livingAudit": { "action": "read-existing" }
        }
    }))?;

    let update = serde_json::to_value(build_manifest_evidence_update(input))?;

    assert_eq!(update["scanRange"]["includeTests"], true);
    assert_eq!(update["resolverDiagnostics"]["status"], "available");
    assert_eq!(update["blindZones"][0]["area"], "resolver");
    assert_eq!(
        update["blindZones"][0]["effect"],
        "JS producer owns this meaning"
    );
    assert_eq!(update["unusedDependencies"]["reviewUnused"], 2);
    assert_eq!(update["livingAudit"]["action"], "read-existing");
    Ok(())
}

#[test]
fn cli_manifest_evidence_update_hard_stops_on_incomplete_evidence() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("manifest-evidence-update.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "evidence": {
                "scanRange": {},
                "confidence": {},
                "resolverDiagnostics": {},
                "blindZones": [],
                "rustAnalysis": {},
                "generatedArtifacts": {},
                "frameworkResourceSurfaces": {},
                "unusedDependencies": {},
                "blockClones": {},
                "sfcEvidence": {}
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-update")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("manifest-evidence-update: invalid request shape"));
    Ok(())
}
