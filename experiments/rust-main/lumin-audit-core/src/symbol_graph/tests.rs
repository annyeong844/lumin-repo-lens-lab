use super::*;
use anyhow::Context;
use serde_json::{json, Value};
use std::fs;

fn empty_extraction() -> Value {
    json!({
        "pathTable": [],
        "fileIds": [],
        "defIndex": [],
        "fileData": [],
        "parseErrorFileIds": [],
    })
}

fn empty_source_use(root: &str) -> Value {
    json!({
        "schemaVersion": "lumin-source-use-assembly-request.v1",
        "root": root,
        "records": [],
    })
}

fn empty_graph() -> Value {
    json!({
        "fanIn": {
            "consumerEntries": [],
            "namespaceUserEntries": [],
        },
        "deadCandidates": {
            "barrelFiles": [],
            "testLikeFiles": [],
        },
        "sfc": {
            "styleAssetReferences": [],
            "templateComponentRefs": [],
            "globalComponentRegistrations": [],
            "generatedComponentManifests": [],
            "generatedManifestExternalUses": 0,
            "frameworkConventionComponents": [],
        },
    })
}

fn request_value(root: &str, extraction: Value, source_use: Value, graph: Value) -> Value {
    json!({
        "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
        "context": {
            "generated": "2026-07-11T00:00:00.000Z",
            "root": root,
            "includeTests": true,
            "exclude": [],
            "generatedArtifactsMode": "default",
            "languageSupport": {"ts": {"enabled": true}},
            "warnings": [],
            "incremental": null,
        },
        "extraction": extraction,
        "sourceUseAssembly": source_use,
        "graph": graph,
    })
}

fn build(root: &str, extraction: Value, source_use: Value, graph: Value) -> Result<Value> {
    let request = serde_json::from_value::<SymbolGraphRequest>(request_value(
        root, extraction, source_use, graph,
    ))?;
    build_symbol_graph_artifact(request)
}

#[test]
fn builds_strict_graph_from_raw_facts_once() -> Result<()> {
    let root = "C:/repo";
    let extraction = json!({
        "pathTable": ["src/a.ts", "src/b.ts"],
        "fileIds": [0, 1],
        "defIndex": [{
            "filePathId": 0,
            "definitions": {
                "alpha": {"name": "alpha", "kind": "FunctionDeclaration", "line": 1},
                "beta": {"name": "beta", "kind": "FunctionDeclaration", "line": 2},
            },
        }],
        "fileData": [{
            "filePathId": 0,
            "pyDunderAll": null,
            "reExports": [{"source": "./b", "line": 3}],
            "classMethods": [
                {"className": "Example", "name": "constructor", "line": 5},
                {"className": "Example", "name": "toString", "line": 6},
                {"className": "Example", "name": "hasOwnProperty", "line": 7},
                {"className": "Example", "name": "valueOf", "line": 8},
                {"className": "Example", "name": "__proto__", "line": 9}
            ],
            "localOperations": [{
                "identity": "src/a.ts::outer#formatName",
                "name": "formatName",
                "containerName": "outer",
                "line": 4,
                "operationFamily": "format",
                "domainTokens": ["name"],
            }],
            "typeEscapes": [{
                "file": "src/a.ts",
                "line": 1,
                "escapeKind": "explicit-any",
                "insideExportedIdentity": "src/a.ts::alpha",
            }],
            "dynamicImportOpacity": [],
            "cjsExportSurface": null,
            "cjsRequireOpacity": [],
        }],
        "parseErrorFileIds": [],
    });
    let source_use = json!({
        "schemaVersion": "lumin-source-use-assembly-request.v1",
        "root": root,
        "records": [
            {
                "recordId": "resolved",
                "consumerFileId": 1,
                "resolvedFileId": 0,
                "fromSpec": "@/a",
                "name": "alpha",
                "kind": "import",
                "typeOnly": false,
                "typeOnlyPresent": true,
                "resolverStage": "resolved-internal",
            },
            {
                "recordId": "external",
                "consumerFileId": 1,
                "fromSpec": "react/jsx-runtime",
                "kind": "import-side-effect",
                "resolverStage": "external",
            },
            {
                "recordId": "missing",
                "consumerFileId": 1,
                "fromSpec": "@/missing",
                "name": "missing",
                "kind": "import",
                "resolverStage": "unresolved-internal",
                "unresolvedEvidence": {
                    "reason": "tsconfig-path-target-missing",
                    "resolverStage": "tsconfig-paths",
                    "hint": "check-tsconfig-paths",
                },
            },
            {
                "recordId": "asset",
                "consumerFileId": 1,
                "fromSpec": "./style.css",
                "kind": "import-side-effect",
                "resolverStage": "non-source-asset",
            },
        ],
    });

    let artifact = build(root, extraction, source_use, empty_graph())?;

    assert_eq!(artifact["meta"]["tool"], TOOL_NAME);
    assert_eq!(artifact["files"], 2);
    assert_eq!(artifact["totalUsesResolved"], 1);
    assert_eq!(artifact["unresolvedUses"], 2);
    assert_eq!(artifact["uses"]["resolvedInternal"], 1);
    assert_eq!(artifact["uses"]["external"], 1);
    assert_eq!(artifact["uses"]["nonSourceAsset"], 1);
    assert_eq!(artifact["fanInByIdentity"]["src/a.ts::alpha"], 1);
    assert_eq!(artifact["fanInByIdentity"]["src/a.ts::beta"], 0);
    assert_eq!(artifact["deadProdList"][0]["symbol"], "beta");
    for method_name in [
        "constructor",
        "toString",
        "hasOwnProperty",
        "valueOf",
        "__proto__",
    ] {
        assert!(artifact["classMethodIndex"]["src/a.ts"][method_name].is_array());
    }
    assert_eq!(
        artifact["helperOwnersByIdentity"]["src/a.ts::alpha"]["anyContamination"]["measurements"]
            ["explicitAnyCount"],
        1
    );
    assert_eq!(
        artifact["unresolvedInternalSummaryByReason"]["tsconfig-path-target-missing"]["count"],
        1
    );
    Ok(())
}

#[test]
fn preserves_export_level_dead_candidates_from_reachable_test_files() -> Result<()> {
    let root = "C:/repo";
    let extraction = json!({
        "pathTable": ["tests/setup/server.js", "tests/server.test.js"],
        "fileIds": [0, 1],
        "defIndex": [{
            "filePathId": 0,
            "definitions": {
                "usedServer": {
                    "name": "usedServer",
                    "kind": "FunctionDeclaration",
                    "line": 1,
                },
                "unusedServer": {
                    "name": "unusedServer",
                    "kind": "FunctionDeclaration",
                    "line": 2,
                },
            },
        }],
        "fileData": [],
        "parseErrorFileIds": [],
    });
    let source_use = json!({
        "schemaVersion": "lumin-source-use-assembly-request.v1",
        "root": root,
        "records": [{
            "recordId": "test-server-consumer",
            "consumerFileId": 1,
            "resolvedFileId": 0,
            "fromSpec": "./setup/server.js",
            "name": "usedServer",
            "kind": "import",
            "resolverStage": "resolved-internal",
        }],
    });
    let mut graph = empty_graph();
    graph["deadCandidates"]["testLikeFiles"] = json!(["tests/setup/server.js"]);

    let artifact = build(root, extraction, source_use, graph)?;

    assert_eq!(
        artifact["fanInByIdentity"]["tests/setup/server.js::usedServer"],
        1
    );
    assert_eq!(artifact["deadInProd"], 0);
    assert_eq!(artifact["deadInTest"], 1);
    assert_eq!(artifact["deadProdList"], json!([]));
    assert_eq!(artifact["deadTestList"][0]["file"], "tests/setup/server.js");
    assert_eq!(artifact["deadTestList"][0]["symbol"], "unusedServer");
    Ok(())
}

#[test]
fn shares_parent_path_table_with_embedded_source_use() -> Result<()> {
    let root = "C:/repo";
    let extraction = json!({
        "pathTable": ["src/consumer.ts", "src/dep.ts"],
        "fileIds": [0, 1],
        "defIndex": [{
            "filePathId": 1,
            "definitions": {
                "alpha": {"name": "alpha", "kind": "FunctionDeclaration", "line": 1},
            },
        }],
        "fileData": [],
        "parseErrorFileIds": [],
    });
    let source_use = json!({
        "schemaVersion": "lumin-source-use-assembly-request.v1",
        "root": root,
        "records": [{
            "recordId": "edge",
            "consumerFileId": 0,
            "resolvedFileId": 1,
            "fromSpec": "./dep",
            "name": "alpha",
            "kind": "import",
            "resolverStage": "resolved-internal",
        }],
    });

    let artifact = build(root, extraction, source_use, empty_graph())?;

    assert_eq!(
        artifact["resolvedInternalEdges"][0]["from"],
        "src/consumer.ts"
    );
    assert_eq!(artifact["resolvedInternalEdges"][0]["to"], "src/dep.ts");
    assert_eq!(artifact["fanInByIdentity"]["src/dep.ts::alpha"], 1);
    Ok(())
}

#[test]
fn projects_sfc_rows_from_source_use_targets() -> Result<()> {
    let root = "C:/repo";
    let extraction = json!({
        "pathTable": [
            "src/App.vue",
            "src/Button.ts",
            "components.d.ts",
            "src/Manifest.ts",
        ],
        "fileIds": [0, 1, 2, 3],
        "defIndex": [],
        "fileData": [],
        "parseErrorFileIds": [],
    });
    let source_use = json!({
        "schemaVersion": "lumin-source-use-assembly-request.v1",
        "root": root,
        "records": [
            {
                "recordId": "template",
                "consumerFileId": 0,
                "resolvedFileId": 1,
                "fromSpec": "./Button",
                "name": "*",
                "kind": "sfc-template-component-ref",
                "consumerSource": "sfc-template-component-ref",
                "resolverStage": "resolved-internal",
            },
            {
                "recordId": "template-asset",
                "consumerFileId": 0,
                "resolvedFile": "src/Button.css",
                "fromSpec": "./Button.css",
                "name": "*",
                "kind": "sfc-template-component-ref",
                "consumerSource": "sfc-template-component-ref",
                "resolverStage": "non-source-asset",
            },
            {
                "recordId": "manifest-local",
                "consumerFileId": 2,
                "resolvedFileId": 3,
                "fromSpec": "./src/Manifest",
                "name": "*",
                "kind": "sfc-generated-component-manifest",
                "consumerSource": "sfc-generated-component-manifest",
                "resolverStage": "resolved-internal",
            },
        ],
    });
    let graph = json!({
        "fanIn": {"consumerEntries": [], "namespaceUserEntries": []},
        "deadCandidates": {"barrelFiles": [], "testLikeFiles": []},
        "sfc": {
            "styleAssetReferences": [],
            "templateComponentRefs": [{
                "consumerFile": "src/App.vue",
                "tagName": "UiButton",
                "bindingName": "UiButton",
                "bindingSource": "./Button",
                "sourceUseRecordId": "template",
            }, {
                "consumerFile": "src/App.vue",
                "tagName": "ZCssButton",
                "bindingSource": "./Button.css",
                "sourceUseRecordId": "template-asset",
            }],
            "globalComponentRegistrations": [],
            "generatedComponentManifests": [{
                "manifestFile": "components.d.ts",
                "componentName": "ManifestComponent",
                "normalizedTagNames": ["manifest-component"],
                "bindingSource": "./src/Manifest",
                "fromSpec": "./src/Manifest",
                "sourceUseRecordId": "manifest-local",
            }],
            "generatedManifestExternalUses": 1,
            "frameworkConventionComponents": [{
                "framework": "nuxt",
                "conventionKind": "components-dir",
                "consumerFile": "src/App.vue",
                "componentName": "ConventionCard",
                "sourceFile": "components/ConventionCard.vue",
                "resolvedFile": "components/ConventionCard.vue",
                "pathPrefix": true,
                "global": true,
                "eligibleForFanIn": false,
                "eligibleForSafeFix": false,
            }],
        },
    });

    let artifact = build(root, extraction, source_use, graph)?;

    assert_eq!(
        artifact["sfcTemplateComponentRefs"][0]["resolvedFile"],
        "src/Button.ts"
    );
    assert_eq!(artifact["sfcTemplateComponentRefs"][1]["status"], "muted");
    assert_eq!(
        artifact["sfcTemplateComponentRefs"][1]["resolvedFile"],
        "src/Button.css"
    );
    assert_eq!(
        artifact["sfcTemplateComponentRefs"][1]["reason"],
        "sfc-template-component-non-source-binding"
    );
    assert_eq!(
        artifact["sfcGeneratedComponentManifests"][0]["resolvedFile"],
        "src/Manifest.ts"
    );
    assert_eq!(artifact["uses"]["sfcGeneratedComponentManifests"], 2);
    assert_eq!(artifact["uses"]["sfcFrameworkConventionComponents"], 1);
    assert_eq!(
        artifact["sfcFrameworkConventionComponents"][0]["eligibleForFanIn"],
        false
    );
    assert_eq!(
        artifact["sfcFrameworkConventionComponents"][0]["eligibleForSafeFix"],
        false
    );
    Ok(())
}

#[test]
fn builds_generated_blind_zone_from_unresolved_evidence() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().to_string_lossy().replace('\\', "/");
    let source_use = json!({
        "schemaVersion": "lumin-source-use-assembly-request.v1",
        "root": root,
        "records": [{
            "recordId": "missing-generated",
            "consumerFile": "src/consumer.ts",
            "fromSpec": "@scope/generated-client",
            "name": "client",
            "kind": "import",
            "resolverStage": "unresolved-internal",
            "unresolvedEvidence": {
                "reason": "workspace-generated-artifact-missing",
                "targetCandidates": ["packages/api/generated/client.ts"],
                "generatedArtifact": {
                    "matchedPackage": "@scope/api",
                    "packageRoot": "packages/api",
                    "generatorFamily": "path-segment",
                    "confidence": "supporting",
                },
            },
        }],
    });

    let artifact = build(&root, empty_extraction(), source_use, empty_graph())?;

    assert_eq!(
        artifact["generatedConsumerBlindZones"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        artifact["generatedConsumerBlindZones"][0]["candidatePath"],
        "packages/api/generated/client.ts"
    );
    Ok(())
}

#[test]
fn explicit_parse_error_ids_are_visible() -> Result<()> {
    let extraction = json!({
        "pathTable": ["src/broken.ts"],
        "fileIds": [0],
        "defIndex": [],
        "fileData": [],
        "parseErrorFileIds": [0],
    });

    let artifact = build(
        "C:/repo",
        extraction,
        empty_source_use("C:/repo"),
        empty_graph(),
    )?;

    assert_eq!(artifact["filesWithParseErrors"][0], "src/broken.ts");
    assert_eq!(artifact["meta"]["warnings"][0]["code"], "parse-errors");
    Ok(())
}

#[test]
fn namespace_reexport_escape_adds_broad_fan_in() -> Result<()> {
    let root = "C:/repo";
    let extraction = json!({
        "pathTable": ["src/source.ts", "src/barrel.ts", "src/consumer.ts"],
        "fileIds": [0, 1, 2],
        "defIndex": [{
            "filePathId": 0,
            "definitions": {
                "alpha": {"name": "alpha", "kind": "FunctionDeclaration", "line": 1},
            },
        }],
        "fileData": [],
        "parseErrorFileIds": [],
    });
    let source_use = json!({
        "schemaVersion": "lumin-source-use-assembly-request.v1",
        "root": root,
        "sourceFileIds": [0, 1, 2],
        "namespaceReExports": [{
            "barrelFile": "src/barrel.ts",
            "exportedName": "ns",
            "targetFile": "src/source.ts",
            "sourceSpec": "./source",
        }],
        "records": [{
            "recordId": "namespace-escape",
            "consumerFileId": 2,
            "fromSpec": "./barrel",
            "name": "ns",
            "kind": "imported-namespace-escape",
            "resolverStage": "relative",
        }],
    });

    let artifact = build(root, extraction, source_use, empty_graph())?;

    assert_eq!(
        artifact["fanInByIdentitySpace"]["src/source.ts::alpha"]["broad"],
        1
    );
    assert_eq!(
        artifact["namespaceReExportDiagnostics"][0]["targetFile"],
        "src/source.ts"
    );
    Ok(())
}

#[test]
fn sfc_style_asset_resolution_requires_a_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let src = temp.path().join("src");
    fs::create_dir_all(&src)?;
    let consumer = src.join("App.vue");
    fs::write(&consumer, "")?;
    fs::create_dir(src.join("App.css"))?;
    let consumer = consumer.to_string_lossy().replace('\\', "/");

    assert_eq!(
        super::sfc::resolve_sfc_style_asset_target(&consumer, "./App.css"),
        None
    );

    fs::remove_dir(src.join("App.css"))?;
    fs::write(src.join("App.css"), "")?;
    assert_eq!(
        super::sfc::resolve_sfc_style_asset_target(&consumer, "./App.css")
            .as_deref()
            .and_then(|path| path.rsplit('/').next()),
        Some("App.css")
    );
    Ok(())
}

#[test]
fn hard_stops_when_embedded_assembly_skips_a_record() -> Result<()> {
    let source_use = json!({
        "schemaVersion": "lumin-source-use-assembly-request.v1",
        "root": "C:/repo",
        "records": [{
            "recordId": "invalid-stage",
            "consumerFile": "src/a.ts",
            "fromSpec": "@/a",
            "name": "alpha",
            "kind": "import",
            "resolverStage": "relative",
        }],
    });

    let error = build("C:/repo", empty_extraction(), source_use, empty_graph())
        .err()
        .map(|error| error.to_string());

    assert!(error
        .as_deref()
        .is_some_and(|message| message.contains("sourceUseAssembly skipped 1 record")));
    Ok(())
}

#[test]
fn rejects_v1_unknown_fields_and_missing_sections() -> Result<()> {
    let v1 = serde_json::from_value::<SymbolGraphRequest>(json!({
        "schemaVersion": "lumin-symbol-graph-producer-request.v1",
        "generated": "2026-07-11T00:00:00.000Z",
        "root": "C:/repo",
    }));
    assert!(v1.is_err());

    let mut unknown = request_value(
        "C:/repo",
        empty_extraction(),
        empty_source_use("C:/repo"),
        empty_graph(),
    );
    unknown
        .as_object_mut()
        .context("strict request object")?
        .insert("dead".to_string(), json!([]));
    assert!(serde_json::from_value::<SymbolGraphRequest>(unknown).is_err());

    let missing_graph = json!({
        "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
        "context": request_value(
            "C:/repo",
            empty_extraction(),
            empty_source_use("C:/repo"),
            empty_graph(),
        )["context"],
        "extraction": empty_extraction(),
        "sourceUseAssembly": empty_source_use("C:/repo"),
    });
    assert!(serde_json::from_value::<SymbolGraphRequest>(missing_graph).is_err());
    Ok(())
}

#[test]
fn rejects_future_schema_after_strict_deserialization() -> Result<()> {
    let mut value = request_value(
        "C:/repo",
        empty_extraction(),
        empty_source_use("C:/repo"),
        empty_graph(),
    );
    value["schemaVersion"] = json!("future");
    let request = serde_json::from_value::<SymbolGraphRequest>(value)?;

    let error = build_symbol_graph_artifact(request)
        .err()
        .map(|error| error.to_string());

    assert!(error
        .as_deref()
        .is_some_and(|message| message.contains("unsupported schemaVersion")));
    Ok(())
}

#[test]
fn rejects_malformed_raw_fact_shapes_instead_of_defaulting_them_empty() -> Result<()> {
    let extraction = json!({
        "pathTable": ["src/a.ts"],
        "fileIds": [0],
        "defIndex": [{
            "filePathId": 0,
            "definitions": {"alpha": 7},
        }],
        "fileData": [],
        "parseErrorFileIds": [],
    });
    let malformed_definition = build(
        "C:/repo",
        extraction,
        empty_source_use("C:/repo"),
        empty_graph(),
    )
    .err()
    .map(|error| error.to_string());
    assert!(malformed_definition
        .as_deref()
        .is_some_and(|message| message.contains("definitions.alpha must be an object")));

    let mut source_use = empty_source_use("C:/repo");
    source_use["records"] = json!([{
        "generatedVirtualSurface": {
            "id": "virtual:generated",
            "exports": "not-an-array",
        },
    }]);
    let malformed_surface = build("C:/repo", empty_extraction(), source_use, empty_graph())
        .err()
        .map(|error| error.to_string());
    assert!(malformed_surface.as_deref().is_some_and(
        |message| message.contains("generatedVirtualSurface.exports must be an array")
    ));

    let mut graph = empty_graph();
    graph["sfc"]["frameworkConventionComponents"] = json!([{"pathPrefix": []}]);
    let malformed_sfc = build(
        "C:/repo",
        empty_extraction(),
        empty_source_use("C:/repo"),
        graph,
    )
    .err()
    .map(|error| error.to_string());
    assert!(malformed_sfc
        .as_deref()
        .is_some_and(|message| message.contains("pathPrefix must be a boolean or string")));
    Ok(())
}
