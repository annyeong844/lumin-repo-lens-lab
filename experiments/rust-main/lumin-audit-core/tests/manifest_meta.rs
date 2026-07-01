use anyhow::{bail, Result};
use serde_json::Value;
use std::process::Command;

use lumin_audit_core::manifest_meta::{build_manifest_meta, ManifestMetaInput};

#[test]
fn manifest_meta_preserves_existing_manifest_shape() -> Result<()> {
    let meta = serde_json::to_value(build_manifest_meta(ManifestMetaInput {
        generated: "2026-07-01T00:00:00.000Z".to_string(),
        profile: "full".to_string(),
        root: "C:/repo".to_string(),
        output: "C:/repo/.audit".to_string(),
    })?)?;

    assert_eq!(meta["generated"], "2026-07-01T00:00:00.000Z");
    assert_eq!(meta["tool"], "audit-repo.mjs");
    assert_eq!(meta["profile"], "full");
    assert_eq!(meta["root"], "C:/repo");
    assert_eq!(meta["output"], "C:/repo/.audit");
    Ok(())
}

#[test]
fn manifest_meta_rejects_unknown_profile() -> Result<()> {
    let result = build_manifest_meta(ManifestMetaInput {
        generated: "2026-07-01T00:00:00.000Z".to_string(),
        profile: "debug".to_string(),
        root: "C:/repo".to_string(),
        output: "C:/repo/.audit".to_string(),
    });
    let error = match result {
        Ok(meta) => bail!("unknown profile must be rejected, got {meta:?}"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("invalid --profile"));
    Ok(())
}

#[test]
fn cli_manifest_meta_emits_json() -> Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-meta")
        .arg("--generated")
        .arg("2026-07-01T00:00:00.000Z")
        .arg("--profile")
        .arg("quick")
        .arg("--root")
        .arg("C:/repo")
        .arg("--output")
        .arg("C:/repo/.audit")
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<Value>(&output.stdout)?;
    assert_eq!(stdout["tool"], "audit-repo.mjs");
    assert_eq!(stdout["profile"], "quick");
    Ok(())
}

#[test]
fn cli_manifest_meta_hard_stops_on_missing_output() -> Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-meta")
        .arg("--generated")
        .arg("2026-07-01T00:00:00.000Z")
        .arg("--profile")
        .arg("quick")
        .arg("--root")
        .arg("C:/repo")
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("manifest-meta: missing --output <dir>"));
    Ok(())
}
