use anyhow::Result;
use std::fs;

use lumin_audit_core::artifact_registry::collect_produced_artifacts;

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

fn names(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}
