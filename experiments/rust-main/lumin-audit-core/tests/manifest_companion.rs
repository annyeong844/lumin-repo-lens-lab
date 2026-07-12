use anyhow::Result;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use lumin_audit_core::manifest_companion::{
    build_manifest_companion_update, ManifestCompanionUpdateInput,
};

#[test]
fn manifest_companion_update_projects_human_companion_blocks() -> Result<()> {
    let input = serde_json::from_value::<ManifestCompanionUpdateInput>(json!({
        "topologyMermaidPath": "C:/repo/.audit/topology.mermaid.md",
        "auditSummaryPath": "C:/repo/.audit/audit-summary.latest.md",
        "reviewPackPath": "C:/repo/.audit/audit-review-pack.latest.md"
    }))?;

    let update = serde_json::to_value(build_manifest_companion_update(input)?)?;

    assert_eq!(
        update["topologyMermaid"]["path"],
        "C:/repo/.audit/topology.mermaid.md"
    );
    assert_eq!(update["topologyMermaid"]["format"], "markdown");
    assert_eq!(update["topologyMermaid"]["source"], "topology.json");
    assert_eq!(
        update["topologyMermaid"]["use"],
        "human visual companion; topology.json remains authoritative for exact citations"
    );
    assert_eq!(
        update["auditSummary"],
        json!({
            "path": "C:/repo/.audit/audit-summary.latest.md",
            "format": "markdown"
        })
    );
    assert_eq!(update["reviewPack"]["format"], "markdown");
    assert_eq!(
        update["reviewPack"]["use"],
        "main assistant reads lanes as artifact briefs; if using built-in reviewer subagents, translate lanes into focused codebase-reading tasks with file:line evidence; the engine never calls external APIs"
    );
    Ok(())
}

#[test]
fn manifest_companion_update_omits_absent_blocks() -> Result<()> {
    let input = serde_json::from_value::<ManifestCompanionUpdateInput>(json!({
        "auditSummaryPath": "C:/repo/.audit/audit-summary.latest.md"
    }))?;

    let update = serde_json::to_value(build_manifest_companion_update(input)?)?;

    assert!(update.get("topologyMermaid").is_none());
    assert!(update.get("reviewPack").is_none());
    assert_eq!(
        update["auditSummary"]["path"],
        "C:/repo/.audit/audit-summary.latest.md"
    );
    Ok(())
}

#[test]
fn manifest_companion_update_rejects_empty_paths() -> Result<()> {
    let input = serde_json::from_value::<ManifestCompanionUpdateInput>(json!({
        "reviewPackPath": "   "
    }))?;

    let Err(error) = build_manifest_companion_update(input) else {
        panic!("empty companion path should hard-stop");
    };
    assert!(error
        .to_string()
        .contains("manifest-companion-update: reviewPackPath must be a non-empty string"));
    Ok(())
}

#[test]
fn cli_manifest_companion_update_reads_stdin_json() -> Result<()> {
    let input = json!({
        "topologyMermaidPath": "C:/repo/.audit/topology.mermaid.md",
        "auditSummaryPath": "C:/repo/.audit/audit-summary.latest.md"
    });

    let mut child = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-companion-update")
        .arg("--input")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "stdin missing"))?;
        stdin.write_all(input.to_string().as_bytes())?;
    }
    let output = child.wait_with_output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["topologyMermaid"]["source"], "topology.json");
    assert_eq!(stdout["auditSummary"]["format"], "markdown");
    Ok(())
}

#[test]
fn cli_manifest_companion_update_hard_stops_on_incomplete_shape() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("companion.json");
    fs::write(&input_path, serde_json::to_vec(&json!([]))?)?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-companion-update")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("manifest-companion-update: invalid request shape"));
    Ok(())
}
