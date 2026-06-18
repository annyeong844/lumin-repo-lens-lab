use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[test]
fn unified_cli_emits_syntax_and_semantic_phases_in_one_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )?;
    fs::write(
        root.join("src").join("lib.rs"),
        "pub fn demo() { let value = Some(1); let _ = value.unwrap(); }\n",
    )?;

    let metadata = metadata_json(&root);
    let metadata_path = temp.path().join("metadata.json");
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)?;

    let stdout = r#"{"reason":"compiler-message","package_id":"path+file:///fixture/app#0.1.0","target":{"kind":["lib"],"crate_types":["lib"],"name":"app","src_path":"src/lib.rs","edition":"2021"},"message":{"level":"error","message":"mismatched types","code":{"code":"E0308","explanation":null},"spans":[{"file_name":"src/lib.rs","is_primary":true,"line_start":1,"line_end":1,"column_start":1,"column_end":2,"expansion":null}],"children":[],"rendered":"error[E0308]: mismatched types\n"}}"#;
    let stdout_path = temp.path().join("cargo-check.stdout.jsonl");
    fs::write(&stdout_path, stdout)?;

    let fake_cargo = write_fake_cargo(temp.path(), &metadata_path, &stdout_path, 101)?;
    let output_path = temp.path().join("rust-analyzer-health.json");
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .context("repo root ancestor")?
        .to_path_buf();

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-rust-analyzer"))
        .arg("--root")
        .arg(&root)
        .arg("--output")
        .arg(&output_path)
        .arg("--source-commit")
        .arg("test-source-commit")
        .arg("--cargo-bin")
        .arg(&fake_cargo)
        .arg("--repo-root")
        .arg(repo_root)
        .output()
        .context("run unified rust analyzer")?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact: Value = serde_json::from_slice(&fs::read(output_path)?)?;
    assert_eq!(artifact["schemaVersion"], "rust-analyzer-health.v1");
    assert_eq!(artifact["meta"]["producer"], "lumin-rust-analyzer");
    assert_eq!(artifact["summary"]["files"], 1);
    assert_eq!(artifact["summary"]["syntaxReviewSignals"], 1);
    assert_eq!(artifact["summary"]["verifiedSemanticFindings"], 1);
    assert_eq!(
        artifact["summary"]["semanticClean"]["status"],
        "unavailable"
    );

    let syntax_signals = artifact["phases"]["syntax"]["files"]["src/lib.rs"]["signals"]
        .as_array()
        .context("syntax signals")?;
    assert_eq!(syntax_signals[0]["kind"], "unwrap-call");

    let semantic_findings = artifact["phases"]["semantic"]["findings"]
        .as_array()
        .context("semantic findings")?;
    assert_eq!(
        semantic_findings[0]["confidence"]["claimKind"],
        "verified.rust.rustc-error-diagnostic"
    );
    Ok(())
}

fn metadata_json(root: &Path) -> Value {
    let manifest_path = root.join("Cargo.toml").display().to_string();
    let src_path = root.join("src").join("lib.rs").display().to_string();
    let target_directory = root.join("target").display().to_string();
    json!({
        "packages": [{
            "id": "path+file:///fixture/app#0.1.0",
            "name": "app",
            "version": "0.1.0",
            "manifest_path": manifest_path,
            "targets": [{
                "kind": ["lib"],
                "crate_types": ["lib"],
                "name": "app",
                "src_path": src_path,
                "edition": "2021",
                "required_features": []
            }],
            "source": null
        }],
        "workspace_members": ["path+file:///fixture/app#0.1.0"],
        "workspace_default_members": ["path+file:///fixture/app#0.1.0"],
        "resolve": {
            "root": "path+file:///fixture/app#0.1.0"
        },
        "workspace_root": root.display().to_string(),
        "target_directory": target_directory
    })
}

#[cfg(unix)]
fn write_fake_cargo(
    dir: &Path,
    metadata_path: &Path,
    stdout_path: &Path,
    check_exit: i32,
) -> Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    let script = dir.join("fake-cargo.sh");
    fs::write(
        &script,
        format!(
            r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "${{1:-}}" == "metadata" ]]; then
  cat "{}"
  exit 0
fi
if [[ "${{1:-}}" == "check" ]]; then
  cat "{}"
  exit {}
fi
echo "unexpected cargo args: $*" >&2
exit 2
"#,
            metadata_path.display(),
            stdout_path.display(),
            check_exit
        ),
    )?;
    let mut permissions = fs::metadata(&script)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions)?;
    Ok(script)
}

#[cfg(windows)]
fn write_fake_cargo(
    dir: &Path,
    metadata_path: &Path,
    stdout_path: &Path,
    check_exit: i32,
) -> Result<PathBuf> {
    let script = dir.join("fake-cargo.cmd");
    fs::write(
        &script,
        format!(
            r#"@echo off
if "%1"=="metadata" (
  type "{}"
  exit /b 0
)
if "%1"=="check" (
  type "{}"
  exit /b {}
)
echo unexpected cargo args: %* 1>&2
exit /b 2
"#,
            metadata_path.display(),
            stdout_path.display(),
            check_exit
        ),
    )?;
    Ok(script)
}
