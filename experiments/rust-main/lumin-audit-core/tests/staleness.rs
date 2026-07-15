use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_staleness_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join("src/lib.ts"), "export const deadThing = 1;\n")?;
    git(&root, &["init"])?;
    git(&root, &["config", "user.email", "lumin@example.invalid"])?;
    git(&root, &["config", "user.name", "Lumin Test"])?;
    git(&root, &["add", "."])?;
    git(&root, &["commit", "-m", "initial"])?;

    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-staleness-producer-request.v1",
            "root": root,
            "generated": "2026-07-04T00:00:00.000Z",
            "symbolsSource": "symbols.json",
            "symbols": {
                "deadProdList": [
                    {
                        "file": "src/lib.ts",
                        "line": 1,
                        "symbol": "deadThing",
                        "kind": "value"
                    }
                ]
            },
            "maxAgeDays": 365,
            "staleAgeDays": 90,
            "since": "5 years ago",
            "skipPickaxe": true,
            "incrementalEnabled": false
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("staleness-artifact")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&result)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    let artifact: Value = serde_json::from_slice(&fs::read(&result)?)?;
    assert_eq!(artifact["meta"]["tool"], "measure-staleness.mjs");
    assert_eq!(artifact["summary"]["total"], 1);
    assert_eq!(artifact["summary"]["byTier"]["recent"], 1);
    assert_eq!(artifact["enriched"][0]["symbolMentionStatus"], "skipped");
    Ok(())
}

#[test]
fn cli_staleness_batches_pickaxe_for_multiple_symbols() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("src/lib.ts"),
        "export const deadAlpha = 1;\nexport const deadBeta = 2;\n",
    )?;
    git(&root, &["init"])?;
    git(&root, &["config", "user.email", "lumin@example.invalid"])?;
    git(&root, &["config", "user.name", "Lumin Test"])?;
    git(&root, &["add", "."])?;
    git(&root, &["commit", "-m", "initial"])?;

    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-staleness-producer-request.v1",
            "root": root,
            "generated": "2026-07-15T00:00:00.000Z",
            "symbolsSource": "symbols.json",
            "symbols": {
                "deadProdList": [
                    { "file": "src/lib.ts", "line": 1, "symbol": "deadAlpha" },
                    { "file": "src/lib.ts", "line": 2, "symbol": "deadBeta" }
                ]
            },
            "since": "5 years ago",
            "skipPickaxe": false,
            "incrementalEnabled": false
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("staleness-artifact")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&result)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let artifact: Value = serde_json::from_slice(&fs::read(&result)?)?;
    assert_eq!(
        artifact["meta"]["pickaxeMode"],
        "batched-text-diff-count-v1"
    );
    assert_eq!(
        artifact["summary"]["performance"]["symbolPickaxeGitCalls"],
        1
    );
    assert_eq!(
        artifact["summary"]["performance"]["symbolPickaxeEligibleSymbols"],
        2
    );
    assert!(artifact["enriched"]
        .as_array()
        .is_some_and(|entries| entries
            .iter()
            .all(|entry| entry["symbolMentionStatus"] == "warm")));
    Ok(())
}

fn git(root: &std::path::Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git").args(args).current_dir(root).output()?;
    if output.status.success() {
        return Ok(());
    }
    anyhow::bail!(
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    )
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
