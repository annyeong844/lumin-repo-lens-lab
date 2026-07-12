use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_js_ts_extract_writes_symbol_extractor_shape() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    let source = r#"import mainThing, { foo as localFoo, type Bar } from "./dep";
import "./side.css";
export { localFoo as renamed };
export type { Bar as Baz } from "./types";
export * as ns from "./all";
export function run() {}
export const value = 1;
export interface Shape {}
export default class Widget {
  public render() {}
  protected helper = () => {};
  #secret() {}
}
"#;
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-js-ts-extract-request.v1",
            "files": [{
                "filePath": "C:/repo/src/widget.ts",
                "artifactFilePath": "src/widget.ts",
                "source": source
            }]
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("js-ts-extract-artifact")
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
    assert_eq!(artifact["schemaVersion"], "lumin-js-ts-extract-response.v1");
    let file = &artifact["files"][0];
    assert_eq!(file["filePath"], "C:/repo/src/widget.ts");
    assert_eq!(file["loc"], 14);
    assert!(file.get("error").is_none());

    assert_eq!(
        file["uses"],
        json!([
            {
                "fromSpec": "./dep",
                "name": "default",
                "kind": "default",
                "typeOnly": false,
                "line": 1
            },
            {
                "fromSpec": "./dep",
                "name": "foo",
                "kind": "import",
                "typeOnly": false,
                "line": 1,
                "localName": "localFoo"
            },
            {
                "fromSpec": "./dep",
                "name": "Bar",
                "kind": "import",
                "typeOnly": true,
                "line": 1
            },
            {
                "fromSpec": "./side.css",
                "name": "*",
                "kind": "import-side-effect",
                "typeOnly": false,
                "line": 2
            },
            {
                "fromSpec": "./types",
                "name": "Bar",
                "kind": "reExport",
                "typeOnly": true,
                "line": 4
            },
            {
                "fromSpec": "./all",
                "name": "ns",
                "kind": "reExportNamespace",
                "typeOnly": false,
                "line": 5
            },
            {
                "fromSpec": "./dep",
                "name": "foo",
                "kind": "imported-namespace-escape",
                "typeOnly": false,
                "line": 1,
                "localName": "localFoo",
                "degraded": true
            }
        ])
    );
    assert_eq!(
        file["reExports"],
        json!([
            { "source": "./types", "line": 4 },
            { "source": "./all", "line": 5, "namespace": "ns" }
        ])
    );
    assert_eq!(file["defs"][0]["name"], "renamed");
    assert_eq!(file["defs"][0]["kind"], "ExportSpecifier");
    assert_eq!(file["defs"][1]["name"], "run");
    assert_eq!(file["defs"][1]["kind"], "FunctionDeclaration");
    assert_eq!(file["defs"][2]["name"], "value");
    assert_eq!(file["defs"][2]["kind"], "const-var");
    assert_eq!(file["defs"][3]["name"], "Shape");
    assert_eq!(file["defs"][3]["kind"], "TSInterfaceDeclaration");
    assert_eq!(file["defs"][4]["name"], "default");
    assert_eq!(file["defs"][4]["kind"], "default");
    assert_eq!(
        file["classMethods"][0]["identity"],
        "src/widget.ts::Widget#render"
    );
    assert_eq!(file["classMethods"][0]["visibility"], "public");
    assert_eq!(
        file["classMethods"][1]["identity"],
        "src/widget.ts::Widget#helper"
    );
    assert_eq!(
        file["classMethods"][1]["memberKind"],
        "class-field-function"
    );
    assert_eq!(file["classMethods"][1]["visibility"], "protected");
    assert_eq!(
        file["classMethods"][2]["identity"],
        "src/widget.ts::Widget##secret"
    );
    assert_eq!(file["classMethods"][2]["visibility"], "private");
    Ok(())
}

#[test]
fn cli_js_ts_extract_preserves_namespace_import_consumers() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-js-ts-extract-request.v1",
            "files": [{
                "filePath": "C:/repo/src/consumer.ts",
                "artifactFilePath": "src/consumer.ts",
                "source": "import * as api from \"./dep\";\napi.foo();\n"
            }]
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("js-ts-extract-artifact")
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
        artifact["files"][0]["uses"],
        json!([{
            "fromSpec": "./dep",
            "name": "*",
            "kind": "namespace",
            "typeOnly": false,
            "line": 1,
            "localName": "api"
        }])
    );
    Ok(())
}

#[test]
fn cli_js_ts_extract_reads_source_from_file_path_when_source_is_omitted() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("repo");
    let source_dir = root.join("src");
    let source_file = source_dir.join("consumer.ts");
    fs::create_dir_all(&source_dir)?;
    fs::write(
        &source_file,
        "import { api } from \"./dep\";\napi.run();\nexport const value = 1;\n",
    )?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-js-ts-extract-request.v1",
            "files": [{
                "filePath": source_file.to_string_lossy(),
                "artifactFilePath": "src/consumer.ts"
            }]
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("js-ts-extract-artifact")
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
    let file = &artifact["files"][0];
    assert!(file.get("error").is_none());
    assert_eq!(file["loc"], 4);
    assert_eq!(file["defs"][0]["name"], "value");
    assert!(file["uses"].as_array().is_some_and(|uses| {
        uses.iter().any(|use_record| {
            use_record["fromSpec"] == "./dep"
                && use_record["kind"] == "imported-namespace-member"
                && use_record["memberName"] == "run"
        })
    }));
    Ok(())
}

#[test]
fn cli_js_ts_extract_preserves_named_import_member_precision() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-js-ts-extract-request.v1",
            "files": [{
                "filePath": "C:/repo/src/consumer.ts",
                "artifactFilePath": "src/consumer.ts",
                "source": concat!(
                    "import { api, api as escaped, safe, shadowed, fallback } from \"./barrel\";\n",
                    "api.foo();\n",
                    "consume(escaped);\n",
                    "if (safe) {}\n",
                    "const { value = fallback } = options;\n",
                    "function inner(shadowed) { shadowed.foo(); }\n"
                )
            }]
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("js-ts-extract-artifact")
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
        artifact["files"][0]["uses"],
        json!([
            {
                "fromSpec": "./barrel",
                "name": "api",
                "kind": "import",
                "typeOnly": false,
                "line": 1
            },
            {
                "fromSpec": "./barrel",
                "name": "api",
                "kind": "import",
                "typeOnly": false,
                "line": 1,
                "localName": "escaped"
            },
            {
                "fromSpec": "./barrel",
                "name": "safe",
                "kind": "import",
                "typeOnly": false,
                "line": 1
            },
            {
                "fromSpec": "./barrel",
                "name": "shadowed",
                "kind": "import",
                "typeOnly": false,
                "line": 1
            },
            {
                "fromSpec": "./barrel",
                "name": "fallback",
                "kind": "import",
                "typeOnly": false,
                "line": 1
            },
            {
                "fromSpec": "./barrel",
                "name": "api",
                "memberName": "foo",
                "kind": "imported-namespace-member",
                "typeOnly": false,
                "line": 2,
                "localName": "api"
            },
            {
                "fromSpec": "./barrel",
                "name": "api",
                "kind": "imported-namespace-escape",
                "typeOnly": false,
                "line": 1,
                "localName": "escaped",
                "degraded": true
            }
        ])
    );
    Ok(())
}

#[test]
fn cli_js_ts_extract_emits_pre_write_local_operations() -> Result<()> {
    let artifact = run_js_ts_extract_for_source(
        "src/repository.ts",
        concat!(
            "function buildAccountRepository() {\n",
            "  function getAccount() { return null; }\n",
            "  const findAccountById = () => null;\n",
            "  const updateAccount = () => null;\n",
            "}\n",
            "export { buildAccountRepository };\n",
            "export const createUserService = () => {\n",
            "  const searchUsers = function() { return []; };\n",
            "};\n",
            "export default function makeOrderService() {\n",
            "  const lookupOrder = () => null;\n",
            "}\n",
        ),
    )?;

    assert_eq!(
        artifact["localOperations"],
        json!([
            {
                "identity": "src/repository.ts::buildAccountRepository#findAccountById",
                "name": "findAccountById",
                "ownerFile": "src/repository.ts",
                "containerName": "buildAccountRepository",
                "containerKind": "function-declaration",
                "scopeKind": "nested-function",
                "matchedField": "preWriteLocalOperationIndex",
                "line": 3,
                "operationFamily": "read-query",
                "domainTokens": ["account", "by", "id"],
                "visibility": "local-only",
                "eligibleForDeadExportRanking": false,
                "eligibleForSafeFix": false
            },
            {
                "identity": "src/repository.ts::buildAccountRepository#getAccount",
                "name": "getAccount",
                "ownerFile": "src/repository.ts",
                "containerName": "buildAccountRepository",
                "containerKind": "function-declaration",
                "scopeKind": "nested-function",
                "matchedField": "preWriteLocalOperationIndex",
                "line": 2,
                "operationFamily": "read-query",
                "domainTokens": ["account"],
                "visibility": "local-only",
                "eligibleForDeadExportRanking": false,
                "eligibleForSafeFix": false
            },
            {
                "identity": "src/repository.ts::createUserService#searchUsers",
                "name": "searchUsers",
                "ownerFile": "src/repository.ts",
                "containerName": "createUserService",
                "containerKind": "const-arrow-function",
                "scopeKind": "nested-function",
                "matchedField": "preWriteLocalOperationIndex",
                "line": 8,
                "operationFamily": "read-query",
                "domainTokens": ["users"],
                "visibility": "local-only",
                "eligibleForDeadExportRanking": false,
                "eligibleForSafeFix": false
            },
            {
                "identity": "src/repository.ts::makeOrderService#lookupOrder",
                "name": "lookupOrder",
                "ownerFile": "src/repository.ts",
                "containerName": "makeOrderService",
                "containerKind": "function-declaration",
                "scopeKind": "nested-function",
                "matchedField": "preWriteLocalOperationIndex",
                "line": 11,
                "operationFamily": "read-query",
                "domainTokens": ["order"],
                "visibility": "local-only",
                "eligibleForDeadExportRanking": false,
                "eligibleForSafeFix": false
            }
        ])
    );
    Ok(())
}

#[test]
fn cli_js_ts_extract_retries_jsx_for_js_module_extensions() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-js-ts-extract-request.v1",
            "files": [
                {
                    "filePath": "C:/repo/src/view.mjs",
                    "artifactFilePath": "src/view.mjs",
                    "source": "import dep from \"./dep\";\nexport const view = <dep.Widget />;\n"
                },
                {
                    "filePath": "C:/repo/src/template.cjs",
                    "artifactFilePath": "src/template.cjs",
                    "source": "const view = <Widget />;\n"
                }
            ]
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("js-ts-extract-artifact")
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
    let mjs = &artifact["files"][0];
    assert!(mjs.get("error").is_none());
    assert_eq!(
        mjs["uses"][0],
        json!({
            "fromSpec": "./dep",
            "name": "default",
            "kind": "default",
            "typeOnly": false,
            "line": 1
        })
    );
    assert_eq!(mjs["defs"][0]["name"], "view");
    assert_eq!(mjs["defs"][0]["kind"], "const-var");
    assert!(artifact["files"][1].get("error").is_none());
    Ok(())
}

#[test]
fn cli_js_ts_extract_annotates_known_relative_source_targets() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-js-ts-extract-request.v1",
            "sourceFiles": [
                "C:/repo/src/dep.ts",
                "C:/repo/src/nested/index.ts",
                "C:/repo/src/compiled.ts"
            ],
            "files": [{
                "filePath": "C:/repo/src/consumer.ts",
                "artifactFilePath": "src/consumer.ts",
                "source": "import { foo } from \"./dep\";\nimport ns from \"./nested\";\nexport { thing } from \"./compiled.js\";\n"
            }]
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("js-ts-extract-artifact")
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
        artifact["files"][0]["uses"],
        json!([
            {
                "fromSpec": "./dep",
                "name": "foo",
                "kind": "import",
                "typeOnly": false,
                "line": 1,
                "resolvedFile": "C:/repo/src/dep.ts",
                "resolverStage": "relative"
            },
            {
                "fromSpec": "./nested",
                "name": "default",
                "kind": "default",
                "typeOnly": false,
                "line": 2,
                "resolvedFile": "C:/repo/src/nested/index.ts",
                "resolverStage": "relative"
            },
            {
                "fromSpec": "./compiled.js",
                "name": "thing",
                "kind": "reExport",
                "typeOnly": false,
                "line": 3,
                "resolvedFile": "C:/repo/src/compiled.ts",
                "resolverStage": "relative"
            }
        ])
    );
    Ok(())
}

#[test]
fn cli_js_ts_extract_emits_import_meta_glob_uses() -> Result<()> {
    let artifact = run_js_ts_extract_for_source(
        "src/routes.ts",
        concat!(
            "export const routes = import.meta.glob('./pages/*.ts');\n",
            "export const dynamic = import.meta.glob(pattern);\n",
        ),
    )?;

    assert_eq!(
        artifact["uses"],
        json!([
            {
                "fromSpec": "./pages/*.ts",
                "name": "*",
                "kind": "import-meta-glob",
                "typeOnly": false,
                "line": 1,
                "degraded": true,
                "resolverStage": "import-meta-glob"
            },
            {
                "fromSpec": "import.meta.glob(<nonliteral>)",
                "name": "*",
                "kind": "import-meta-glob",
                "typeOnly": false,
                "line": 2,
                "degraded": true,
                "resolverStage": "import-meta-glob"
            }
        ])
    );
    Ok(())
}

#[test]
fn cli_js_ts_extract_emits_cjs_require_evidence() -> Result<()> {
    let artifact = run_js_ts_extract_for_source(
        "src/cjs.cjs",
        concat!(
            "const api = require('./api');\n",
            "api.run();\n",
            "const value = api.value;\n",
            "const { exact } = require('./exact');\n",
            "require('./side-effect');\n",
            "const broad = require('./broad');\n",
            "use(broad);\n",
            "const config = require(path.join(__dirname, 'config.json'));\n",
            "const dyn = require(target);\n",
            "module.exports = require('./reexport');\n",
        ),
    )?;
    let uses = artifact["uses"]
        .as_array()
        .context("uses should be an array")?;

    assert!(uses.iter().any(|use_record| {
        use_record["fromSpec"] == "./api"
            && use_record["name"] == "run"
            && use_record["kind"] == "cjs-namespace-member"
            && use_record["localName"] == "api"
            && use_record["degraded"] != true
    }));
    assert!(uses.iter().any(|use_record| {
        use_record["fromSpec"] == "./api"
            && use_record["name"] == "value"
            && use_record["kind"] == "cjs-namespace-member"
            && use_record["localName"] == "api"
            && use_record["degraded"] != true
    }));
    assert!(uses.iter().any(|use_record| {
        use_record["fromSpec"] == "./exact"
            && use_record["name"] == "exact"
            && use_record["kind"] == "cjs-require-exact"
            && use_record["degraded"] != true
    }));
    assert!(uses.iter().any(|use_record| {
        use_record["fromSpec"] == "./side-effect"
            && use_record["name"] == "*"
            && use_record["kind"] == "cjs-side-effect-only"
    }));
    assert!(uses.iter().any(|use_record| {
        use_record["fromSpec"] == "./broad"
            && use_record["name"] == "*"
            && use_record["kind"] == "cjs-namespace-escape"
            && use_record["localName"] == "broad"
            && use_record["degraded"] == true
    }));
    assert!(uses.iter().any(|use_record| {
        use_record["fromSpec"] == "./reexport"
            && use_record["name"] == "*"
            && use_record["kind"] == "cjs-reexport-broad"
            && use_record["degraded"] == true
    }));

    assert_eq!(
        artifact["cjsRequireOpacity"],
        json!([{
            "line": 9,
            "kind": "dynamic-require"
        }])
    );
    Ok(())
}

#[test]
fn cli_js_ts_extract_emits_type_escape_facts() -> Result<()> {
    let source = concat!(
        "type A = any;\n",
        "const b = (x as any);\n",
        "const c = (<any>x);\n",
        "const d = (x as unknown as Foo);\n",
        "function e(...args: any[]) {}\n",
        "type F = { [k: string]: any };\n",
        "type G<T = any> = T;\n",
        "// @ts-ignore reason\nconst h = 1;\n",
        "// @ts-expect-error upstream type bug\nconst i = 1;\n",
        "// eslint-disable-next-line no-explicit-any\nconst j = 1;\n",
        "/** @type {any} */\nconst k = readValue();\n",
    );
    let artifact = run_js_ts_extract_for_source("src/escapes.ts", source)?;
    let escapes = type_escapes(&artifact);
    let kinds: Vec<&str> = escapes
        .iter()
        .filter_map(|entry| entry["escapeKind"].as_str())
        .collect();

    assert_eq!(
        kinds,
        vec![
            "explicit-any",
            "as-any",
            "angle-any",
            "as-unknown-as-T",
            "rest-any-args",
            "index-sig-any",
            "generic-default-any",
            "ts-ignore",
            "ts-expect-error",
            "no-explicit-any-disable",
            "jsdoc-any",
        ]
    );
    assert!(escapes.iter().all(|entry| {
        entry["file"] == "src/escapes.ts"
            && entry["occurrenceKey"]
                .as_str()
                .is_some_and(|key| key.starts_with("sha256:"))
            && entry["normalizedCodeShape"].as_str().is_some()
    }));
    Ok(())
}

#[test]
fn cli_js_ts_extract_keeps_nested_as_any_casts_specific() -> Result<()> {
    let artifact = run_js_ts_extract_for_source(
        "src/nested-casts.ts",
        "const value = (foo as any) as any;\n",
    )?;
    let escapes = type_escapes(&artifact);
    let as_any_count = escapes
        .iter()
        .filter(|entry| entry["escapeKind"] == "as-any")
        .count();
    let explicit_any_count = escapes
        .iter()
        .filter(|entry| entry["escapeKind"] == "explicit-any")
        .count();

    assert_eq!(as_any_count, 2);
    assert_eq!(explicit_any_count, 0);
    Ok(())
}

#[test]
fn cli_js_ts_extract_type_escapes_preserve_export_identity_and_normalization() -> Result<()> {
    let source = concat!(
        "export type X = any;\n",
        "function foo() { return value   as    any ; }\n",
        "export { foo as bar };\n",
        "export default () => (x as any);\n",
        "const literal = (\"a   b\" as any);\n",
    );
    let artifact = run_js_ts_extract_for_source("src/owners.ts", source)?;
    let escapes = type_escapes(&artifact);

    let explicit = escape_by_kind(&escapes, "explicit-any");
    assert_eq!(explicit["insideExportedIdentity"], "src/owners.ts::X");

    let bar = escapes.iter().find(|entry| {
        entry["insideExportedIdentity"] == "src/owners.ts::bar" && entry["escapeKind"] == "as-any"
    });
    assert!(bar.is_some());

    let default = escapes.iter().find(|entry| {
        entry["insideExportedIdentity"] == "src/owners.ts::default"
            && entry["escapeKind"] == "as-any"
    });
    assert!(default.is_some());

    let literal = escapes.iter().find(|entry| {
        entry["codeShape"]
            .as_str()
            .is_some_and(|shape| shape.contains("a   b"))
    });
    let Some(literal) = literal else {
        panic!("missing string-literal as-any escape");
    };
    assert_eq!(literal["normalizedCodeShape"], "\"a   b\" as any");
    Ok(())
}

#[test]
fn cli_js_ts_extract_records_parse_error_per_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-js-ts-extract-request.v1",
            "files": [{
                "filePath": "C:/repo/src/bad.ts",
                "artifactFilePath": "src/bad.ts",
                "source": "export const = ;"
            }]
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("js-ts-extract-artifact")
        .arg("--input")
        .arg(&input)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let artifact: Value = serde_json::from_slice(&output.stdout)?;
    let file = &artifact["files"][0];
    assert_eq!(file["filePath"], "C:/repo/src/bad.ts");
    assert!(file["error"]
        .as_str()
        .unwrap_or_default()
        .contains("oxc-parser"));
    assert_eq!(file["defs"], json!([]));
    assert_eq!(file["uses"], json!([]));
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}

fn run_js_ts_extract_for_source(artifact_file_path: &str, source: &str) -> Result<Value> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-js-ts-extract-request.v1",
            "files": [{
                "filePath": format!("C:/repo/{artifact_file_path}"),
                "artifactFilePath": artifact_file_path,
                "source": source
            }]
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("js-ts-extract-artifact")
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
    Ok(artifact["files"][0].clone())
}

fn type_escapes(file_artifact: &Value) -> Vec<Value> {
    let Some(items) = file_artifact["typeEscapes"].as_array() else {
        panic!("typeEscapes is not an array");
    };
    items.clone()
}

fn escape_by_kind(escapes: &[Value], kind: &str) -> Value {
    for entry in escapes {
        if entry["escapeKind"] == kind {
            return entry.clone();
        }
    }
    panic!("missing escape kind {kind}");
}
