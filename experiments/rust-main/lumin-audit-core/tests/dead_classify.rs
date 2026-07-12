use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_dead_classify_artifact_writes_result_file() -> Result<()> {
    let root = std::env::temp_dir().join(format!("lumin-dead-classify-cli-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;
    let input_path = root.join("request.json");
    let output_path = root.join("dead-classify.json");

    fs::write(
        &input_path,
        serde_json::to_string(&json!({
            "schemaVersion": "lumin-dead-classify-producer-request.v1",
            "classifiedCandidates": [
                {
                    "file": "src/dead.ts",
                    "line": 1,
                    "symbol": "Dead",
                    "kind": "FunctionDeclaration",
                    "fileInternalUses": 0,
                    "fileInternalUsesEvidence": "ast-ident-ref-count"
                },
                {
                    "file": "src/hub.ts",
                    "line": 7,
                    "symbol": "Hub",
                    "kind": "TSInterfaceDeclaration",
                    "fileInternalUses": 4
                }
            ],
            "excludedCandidates": [],
            "unprocessedCandidates": [],
            "excludedSummary": {
                "config_FP22": 0,
                "publicApi_FP23": 0,
                "scriptEntrypoint_FP45": 0,
                "htmlEntrypoint_FP47": 0,
                "frameworkSentinel_FP27": 0,
                "nuxtNitro_FP30": 0,
                "vitePress_FP46": 0,
                "declarationSidecar_FP48": 0,
                "dynamicImportOpacity_FP18": 0,
                "testConsumer_FP44": 0,
                "transitiveBarrelAdded_FP25": 0,
                "isNuxtNitroDetected": false,
                "testConsumerDiagnostics_FP44": 0
            },
            "frameworkPolicy": {},
            "performance": {"deadCandidatesProcessed": 2},
            "incomplete": false
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("dead-classify-artifact")
        .arg("--input")
        .arg(&input_path)
        .arg("--result-output")
        .arg(&output_path)
        .output()
        .context("failed to spawn lumin-audit-core")?;
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());

    let artifact: Value = serde_json::from_str(&fs::read_to_string(&output_path)?)?;
    assert_eq!(artifact["summary"]["category_C"], 1);
    assert_eq!(artifact["summary"]["category_B"], 1);
    assert_eq!(artifact["proposal_C_remove_symbol"][0]["symbol"], "Dead");
    assert_eq!(artifact["proposal_B_review"][0]["symbol"], "Hub");

    let _ = fs::remove_dir_all(&root);
    Ok(())
}
