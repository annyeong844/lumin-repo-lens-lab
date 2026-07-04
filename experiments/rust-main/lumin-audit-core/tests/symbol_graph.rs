#![recursion_limit = "256"]

use anyhow::Result;
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
            "schemaVersion": "lumin-symbol-graph-producer-request.v1",
            "generated": "2026-07-05T00:00:00.000Z",
            "root": "C:/repo",
            "files": ["C:/repo/src/a.ts", "C:/repo/src/b.ts"],
            "defIndex": [
                {
                    "filePath": "C:/repo/src/a.ts",
                    "definitions": {
                        "alpha": { "name": "alpha", "kind": "FunctionDeclaration", "line": 1 }
                    }
                }
            ],
            "fileData": [
                {
                    "filePath": "C:/repo/src/a.ts",
                    "reExports": [{ "source": "./b", "line": 2 }],
                    "classMethods": [],
                    "localOperations": [],
                    "dynamicImportOpacity": [],
                    "cjsExportSurface": null,
                    "cjsRequireOpacity": []
                }
            ],
            "parseErrors": 0,
            "warnings": [],
            "nextCacheEntries": {},
            "unresolvedInternalByPrefix": [{ "key": "@/missing", "count": 1 }],
            "prefixExamples": { "@/missing": "@/missing/foo" },
            "unresolvedInternalSpecifiers": ["@/missing/foo"],
            "unresolvedInternalSpecifierRecords": [
                {
                    "specifier": "@/missing/foo",
                    "consumerFile": "src/b.ts",
                    "kind": "import",
                    "typeOnly": false,
                    "reason": "alias-miss"
                }
            ],
            "languageSupport": { "ts": { "enabled": true, "reason": null } },
            "totalUses": 1,
            "unresolvedUses": 1,
            "resolvedInternalUses": 1,
            "resolvedGeneratedVirtualUses": 0,
            "nonSourceAssetUses": 0,
            "externalUses": 0,
            "dependencyImportConsumers": [],
            "resolvedInternalEdges": [
                { "from": "src/b.ts", "to": "src/a.ts", "kind": "import", "source": "./a", "typeOnly": false }
            ],
            "generatedConsumerBlindZones": [],
            "generatedVirtualSurfaces": [],
            "generatedVirtualImportConsumers": [],
            "unresolvedInternalUses": 1,
            "mdxConsumerUses": 0,
            "sfcScriptConsumerUses": 0,
            "sfcScriptSrcReachabilityUses": 0,
            "sfcStyleAssetReferenceUses": 0,
            "sfcTemplateComponentRefUses": 0,
            "sfcGlobalComponentRegistrationUses": 0,
            "sfcGeneratedComponentManifestUses": 0,
            "sfcFrameworkConventionComponentUses": 0,
            "sfcStyleAssetReferences": [],
            "sfcTemplateComponentRefs": [],
            "sfcGlobalComponentRegistrations": [],
            "sfcGeneratedComponentManifests": [],
            "sfcFrameworkConventionComponents": [],
            "dead": [{ "file": "src/a.ts", "symbol": "alpha", "line": 1 }],
            "trulyDead": [{ "file": "src/a.ts", "symbol": "alpha", "line": 1 }],
            "deadInProd": [{ "file": "src/a.ts", "symbol": "alpha", "line": 1 }],
            "deadInTest": [],
            "symbolFanIn": [
                { "defFile": "src/a.ts", "symbol": "alpha", "count": 0, "kind": "FunctionDeclaration" }
            ],
            "fanInByIdentity": { "src/a.ts::alpha": 0 },
            "fanInByIdentitySpace": { "src/a.ts::alpha": { "value": 0, "type": 0, "broad": 0 } },
            "namespaceReExportDiagnostics": [],
            "anyContaminationFacts": {
                "helperOwnersByIdentity": {},
                "typeOwnersByIdentity": {}
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
    assert_eq!(artifact["fanInByIdentity"]["src/a.ts::alpha"], 0);
    assert_eq!(artifact["deadProdList"][0]["symbol"], "alpha");
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
