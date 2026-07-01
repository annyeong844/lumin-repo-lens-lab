use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use lumin_audit_core::manifest_root::{build_manifest_root, ManifestRootInput};

#[test]
fn manifest_root_preserves_js_owned_blocks_and_places_rust_owned_fields() -> Result<()> {
    let root = serde_json::from_value::<ManifestRootInput>(json!({
        "generated": "2026-07-01T00:00:00.000Z",
        "profile": "full",
        "root": "C:/repo",
        "output": "C:/repo/.audit",
        "commandsRun": [
            { "step": "measure-topology.mjs", "status": "ok", "ms": 12 }
        ],
        "skipped": [
            { "step": "emit-sarif.mjs", "reason": "not in --sarif mode" }
        ],
        "evidence": {
            "scanRange": { "root": "C:/repo", "includeTests": true },
            "confidence": { "parseErrors": 0, "unresolvedInternal": 2 },
            "blindZones": [
                {
                    "area": "rust",
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
            "livingAudit": { "action": "read-existing" }
        },
        "artifactsProduced": ["triage.json", "symbols.json"]
    }))?;
    let manifest = serde_json::to_value(build_manifest_root(root)?)?;

    assert_eq!(manifest["meta"]["tool"], "audit-repo.mjs");
    assert_eq!(manifest["meta"]["profile"], "full");
    assert_eq!(manifest["profile"], "full");
    assert_eq!(manifest["commandsRun"][0]["step"], "measure-topology.mjs");
    assert_eq!(manifest["skipped"][0]["reason"], "not in --sarif mode");
    assert_eq!(manifest["blindZones"][0]["area"], "rust");
    assert_eq!(manifest["blindZones"][0]["details"]["files"], 2);
    assert_eq!(manifest["rustAnalysis"]["available"], true);
    assert_eq!(
        manifest["artifactsProduced"],
        json!(["triage.json", "symbols.json"])
    );
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
