use anyhow::{Context, Result};
use serde_json::json;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn cli_discipline_artifact_writes_result_file() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path().join("repo");
    let out = temp.path().join("out");
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(&out)?;
    fs::write(
        root.join("src/app.ts"),
        "const value: any = input as any;\n// TODO\n",
    )?;
    fs::write(root.join("src/tool.py"), "# noqa\nvalue = exec('x')\n")?;

    let input_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-discipline-producer-request.v1",
            "generated": "2026-07-04T00:00:00.000Z",
            "root": root,
            "files": ["src/app.ts", "src/tool.py"]
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("discipline-artifact")
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
    assert!(output.stdout.is_empty());
    let artifact: serde_json::Value =
        serde_json::from_slice(&fs::read(&result_path).context("read result")?)?;
    assert_eq!(artifact["meta"]["tool"], "measure-discipline.mjs");
    assert_eq!(artifact["scannedFiles"], 2);
    assert_eq!(artifact["unreadableFiles"], 0);
    assert_eq!(artifact["totals"][":any"], 1);
    assert_eq!(artifact["totals"]["as any"], 1);
    assert_eq!(artifact["totals"]["TODO"], 1);
    assert_eq!(artifact["totals"]["# noqa"], 1);
    assert_eq!(artifact["totals"]["exec("], 1);
    assert_eq!(artifact["overallTopOffenders"][0]["file"], "src/app.ts");
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
