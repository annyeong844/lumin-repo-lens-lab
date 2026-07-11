#![recursion_limit = "256"]

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_symbol_graph_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-symbol-graph-producer-request.v2",
            "context": {
                "generated": "2026-07-05T00:00:00.000Z",
                "root": "C:/repo",
                "includeTests": true,
                "exclude": [],
                "generatedArtifactsMode": "default",
                "languageSupport": { "ts": { "enabled": true, "reason": null } },
                "warnings": [],
                "incremental": null
            },
            "extraction": {
                "pathTable": ["src/a.ts", "src/b.ts"],
                "fileIds": [0, 1],
                "defIndex": [{
                    "filePathId": 0,
                    "definitions": {
                        "alpha": { "name": "alpha", "kind": "FunctionDeclaration", "line": 1 },
                        "beta": { "name": "beta", "kind": "FunctionDeclaration", "line": 2 }
                    }
                }],
                "fileData": [{
                    "filePathId": 0,
                    "pyDunderAll": null,
                    "reExports": [{ "source": "./b", "line": 2 }],
                    "classMethods": [],
                    "localOperations": [],
                    "typeEscapes": [],
                    "dynamicImportOpacity": [],
                    "cjsExportSurface": null,
                    "cjsRequireOpacity": []
                }],
                "parseErrorFileIds": []
            },
            "sourceUseAssembly": {
                "schemaVersion": "lumin-source-use-assembly-request.v1",
                "root": "C:/repo",
                "records": [{
                    "recordId": "resolved",
                    "consumerFileId": 1,
                    "resolvedFileId": 0,
                    "fromSpec": "./a",
                    "name": "alpha",
                    "kind": "import",
                    "typeOnly": false,
                    "typeOnlyPresent": true,
                    "resolverStage": "resolved-internal"
                }, {
                    "recordId": "missing",
                    "consumerFileId": 1,
                    "fromSpec": "@/missing/foo",
                    "name": "missing",
                    "kind": "import",
                    "resolverStage": "unresolved-internal",
                    "unresolvedEvidence": { "reason": "alias-miss" }
                }]
            },
            "graph": {
                "fanIn": { "consumerEntries": [], "namespaceUserEntries": [] },
                "deadCandidates": { "barrelFiles": [], "testLikeFiles": [] },
                "sfc": {
                    "styleAssetReferences": [],
                    "templateComponentRefs": [],
                    "globalComponentRegistrations": [],
                    "generatedComponentManifests": [],
                    "generatedManifestExternalUses": 0,
                    "frameworkConventionComponents": []
                }
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("symbol-graph-artifact")
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
    assert_eq!(artifact["meta"]["tool"], "build-symbol-graph.mjs");
    assert_eq!(artifact["meta"]["schemaVersion"], 3);
    assert_eq!(artifact["files"], 2);
    assert_eq!(artifact["uses"]["unresolvedInternalRatio"], 0.5);
    assert_eq!(artifact["fanInByIdentity"]["src/a.ts::alpha"], 1);
    assert_eq!(artifact["deadProdList"][0]["symbol"], "beta");
    Ok(())
}

#[test]
fn cli_symbol_graph_resolves_sfc_style_asset_inputs() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path();
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir)?;
    let app = src_dir.join("App.vue");
    let css = src_dir.join("App.css");
    fs::write(&app, "<style>@import \"./App.css?inline\";</style>")?;
    fs::write(&css, ".root {}")?;

    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-symbol-graph-producer-request.v2",
            "context": {
                "generated": "2026-07-05T00:00:00.000Z",
                "root": root.to_string_lossy(),
                "includeTests": true,
                "exclude": [],
                "generatedArtifactsMode": "default",
                "languageSupport": { "ts": { "enabled": true, "reason": null } },
                "warnings": [],
                "incremental": null
            },
            "extraction": {
                "pathTable": [app.to_string_lossy()],
                "fileIds": [0],
                "defIndex": [],
                "fileData": [],
                "parseErrorFileIds": []
            },
            "sourceUseAssembly": {
                "schemaVersion": "lumin-source-use-assembly-request.v1",
                "root": root.to_string_lossy(),
                "records": []
            },
            "graph": {
                "fanIn": { "consumerEntries": [], "namespaceUserEntries": [] },
                "deadCandidates": { "barrelFiles": [], "testLikeFiles": [] },
                "sfc": {
                    "styleAssetReferences": [{
                        "consumerFile": app.to_string_lossy(),
                        "fromSpec": "./App.css?inline",
                        "source": "sfc-style-import",
                        "kind": "sfc-style-import",
                        "styleKind": "import",
                        "confidence": "grounded-asset-reference",
                        "importSyntax": "src",
                        "line": 1,
                        "sfcBlockKind": "vue-style",
                        "sfcLanguage": "vue"
                    }, {
                        "consumerFile": app.to_string_lossy(),
                        "fromSpec": "./missing.css",
                        "source": "sfc-style-url",
                        "kind": "sfc-style-url",
                        "styleKind": "url",
                        "confidence": "grounded-asset-reference",
                        "line": 2,
                        "sfcBlockKind": "vue-style",
                        "sfcLanguage": "vue"
                    }],
                    "templateComponentRefs": [],
                    "globalComponentRegistrations": [],
                    "generatedComponentManifests": [],
                    "generatedManifestExternalUses": 0,
                    "frameworkConventionComponents": []
                }
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("symbol-graph-artifact")
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
    assert_eq!(artifact["uses"]["sfcStyleAssetReferences"], 1);
    let references = artifact["sfcStyleAssetReferences"]
        .as_array()
        .context("sfcStyleAssetReferences array")?;
    assert!(references
        .iter()
        .any(|reference| reference["fromSpec"] == "./App.css?inline"
            && reference["resolvedFile"] == "src/App.css"
            && reference["status"] == "resolved"));
    assert!(references
        .iter()
        .any(|reference| reference["fromSpec"] == "./missing.css"
            && reference["status"] == "unresolved"
            && reference["reason"] == "sfc-style-asset-unresolved"));
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
