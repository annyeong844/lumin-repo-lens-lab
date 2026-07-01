use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

use lumin_audit_core::living_audit::summarize_living_audit;

#[test]
fn living_audit_summary_reports_create_only_when_no_docs_exist() -> Result<()> {
    let tempdir = tempfile::tempdir()?;

    let summary = serde_json::to_value(summarize_living_audit(tempdir.path()))?;

    assert_eq!(
        summary,
        json!({
            "preferredPath": "docs/current/audit/lumin-structural-audit.md",
            "existingDocs": [],
            "action": "create-only-on-explicit-tracking-request"
        })
    );
    Ok(())
}

#[test]
fn living_audit_summary_preserves_candidate_order_and_absolute_paths() -> Result<()> {
    let tempdir = tempfile::tempdir()?;
    fs::write(tempdir.path().join("LUMIN_AUDIT.md"), "audit")?;
    let nested = tempdir.path().join("docs/current/audit");
    fs::create_dir_all(&nested)?;
    fs::write(nested.join("lumin-structural-audit.md"), "structural")?;

    let summary = serde_json::to_value(summarize_living_audit(tempdir.path()))?;

    assert_eq!(summary["action"], "read-and-update-before-final-answer");
    assert_eq!(
        summary["existingDocs"][0],
        json!({
            "path": "docs/current/audit/lumin-structural-audit.md",
            "absolutePath": tempdir.path()
                .join("docs/current/audit/lumin-structural-audit.md")
                .to_string_lossy()
        })
    );
    assert_eq!(
        summary["existingDocs"][1],
        json!({
            "path": "LUMIN_AUDIT.md",
            "absolutePath": tempdir.path().join("LUMIN_AUDIT.md").to_string_lossy()
        })
    );
    Ok(())
}

#[test]
fn cli_living_audit_summary_emits_json() -> Result<()> {
    let tempdir = tempfile::tempdir()?;
    fs::write(tempdir.path().join("TECH_DEBT_AUDIT.md"), "debt")?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("living-audit-summary")
        .arg("--root")
        .arg(tempdir.path())
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(
        stdout["preferredPath"],
        "docs/current/audit/lumin-structural-audit.md"
    );
    assert_eq!(stdout["existingDocs"][0]["path"], "TECH_DEBT_AUDIT.md");
    assert_eq!(stdout["action"], "read-and-update-before-final-answer");
    Ok(())
}
