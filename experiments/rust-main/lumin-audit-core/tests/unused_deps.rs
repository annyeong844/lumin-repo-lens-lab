use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use lumin_audit_core::unused_deps::{
    build_unused_deps_artifact, package_name_from_specifier, script_tool_evidence,
    UnusedDepsProducerRequest,
};

fn request(package_records: Value, symbols: Value) -> Result<UnusedDepsProducerRequest> {
    Ok(serde_json::from_value(json!({
        "schemaVersion": "lumin-unused-deps-producer-request.v1",
        "root": "C:/repo",
        "includeTests": true,
        "exclude": [],
        "packageRecords": package_records,
        "symbols": symbols
    }))?)
}

#[test]
fn normalizes_package_specifiers_like_js_producer() {
    assert_eq!(
        package_name_from_specifier("react"),
        Some("react".to_string())
    );
    assert_eq!(
        package_name_from_specifier("react/jsx-runtime"),
        Some("react".to_string())
    );
    assert_eq!(
        package_name_from_specifier("@scope/pkg/sub/path"),
        Some("@scope/pkg".to_string())
    );
    assert_eq!(package_name_from_specifier("node:fs"), None);
    assert_eq!(package_name_from_specifier("./local"), None);
    assert_eq!(package_name_from_specifier("../local"), None);
    assert_eq!(package_name_from_specifier("/abs/local"), None);
    assert_eq!(package_name_from_specifier("C:/abs/local"), None);
    assert_eq!(
        package_name_from_specifier("https://cdn.example/pkg.js"),
        None
    );
    assert_eq!(
        package_name_from_specifier("data:text/javascript,export{}"),
        None
    );
    assert_eq!(package_name_from_specifier("#internal"), None);
    assert_eq!(package_name_from_specifier("virtual:foo"), None);
    assert_eq!(package_name_from_specifier("@broken"), None);
    assert_eq!(package_name_from_specifier(""), None);
}

#[test]
fn extracts_direct_package_script_tool_evidence_without_following_wrappers() -> Result<()> {
    let record = serde_json::from_value(json!({
        "root": "C:/repo",
        "relRoot": ".",
        "packageJson": {
            "scripts": {
                "start": "tsx src/server.ts",
                "dev": "vite --host 0.0.0.0",
                "lint": "pnpm eslint .",
                "bunvite": "bunx vite build",
                "npxlint": "npx eslint .",
                "npmexec": "npm exec eslint .",
                "npmstart": "npm start",
                "npmtest": "npm test",
                "wrapped": "npm run start"
            }
        }
    }))?;
    let evidence = script_tool_evidence(&record);
    let keys = evidence
        .iter()
        .map(|entry| format!("{}:{}", entry.tool, entry.script_name))
        .collect::<Vec<_>>();

    assert_eq!(
        keys,
        vec![
            "eslint:lint",
            "eslint:npmexec",
            "eslint:npxlint",
            "tsx:start",
            "vite:bunvite",
            "vite:dev"
        ]
    );
    Ok(())
}

#[test]
fn classifies_used_muted_and_review_unused_dependencies() -> Result<()> {
    let artifact = build_unused_deps_artifact(request(
        json!([{
            "root": "C:/repo",
            "relRoot": ".",
            "packageJson": {
                "name": "app",
                "scripts": { "start": "tsx src/server.ts" },
                "dependencies": { "react": "^19.0.0", "left-pad": "^1.3.0" },
                "devDependencies": { "tsx": "^4.0.0", "@types/node": "^22.0.0" },
                "peerDependencies": { "@storybook/react": "^8.0.0" },
                "optionalDependencies": { "fsevents": "^2.3.0" }
            }
        }]),
        json!({
            "meta": { "supports": { "dependencyImportConsumers": true } },
            "dependencyImportConsumers": [
                {
                    "file": "src/app.tsx",
                    "fromSpec": "react/jsx-runtime",
                    "depRoot": "react",
                    "kind": "import",
                    "source": "source-import"
                }
            ]
        }),
    )?)?;
    let value = serde_json::to_value(artifact)?;

    assert_eq!(value["schemaVersion"], "unused-deps.v1");
    assert_eq!(value["policyVersion"], "unused-deps-review-policy-v1");
    assert_eq!(value["status"], "complete");
    assert_eq!(value["summary"]["declaredDependencyCount"], 6);
    assert_eq!(value["summary"]["usedCount"], 1);
    assert_eq!(value["summary"]["mutedCount"], 4);
    assert_eq!(value["summary"]["reviewUnusedCount"], 1);
    assert_eq!(dep(&value, "react")?["status"], "used");
    assert_eq!(dep(&value, "left-pad")?["status"], "review-unused");
    assert_eq!(dep(&value, "tsx")?["reason"], "package-script-tool");
    assert_eq!(dep(&value, "@types/node")?["reason"], "ambient-types");
    assert_eq!(dep(&value, "@storybook/react")?["reason"], "peer-contract");
    assert_eq!(dep(&value, "fsevents")?["reason"], "optional-runtime");
    Ok(())
}

#[test]
fn keeps_workspace_package_scopes_separate() -> Result<()> {
    let artifact = build_unused_deps_artifact(request(
        json!([
            {
                "root": "C:/repo",
                "relRoot": ".",
                "packageJson": {
                    "name": "root-app",
                    "dependencies": {
                        "react": "^19.0.0",
                        "@repo/shared": "workspace:*"
                    }
                }
            },
            {
                "root": "C:/repo/packages/app",
                "relRoot": "packages/app",
                "packageJson": {
                    "name": "@repo/app",
                    "dependencies": {
                        "react": "^19.0.0",
                        "@repo/shared": "workspace:*"
                    }
                }
            },
            {
                "root": "C:/repo/packages/shared",
                "relRoot": "packages/shared",
                "packageJson": { "name": "@repo/shared" }
            }
        ]),
        json!({
            "meta": { "supports": { "dependencyImportConsumers": true } },
            "dependencyImportConsumers": [
                {
                    "file": "packages/app/src/App.tsx",
                    "fromSpec": "react",
                    "depRoot": "react",
                    "kind": "import",
                    "source": "source-import"
                }
            ]
        }),
    )?)?;
    let value = serde_json::to_value(artifact)?;

    assert_eq!(pkg_dep(&value, ".", "react")?["status"], "review-unused");
    assert_eq!(pkg_dep(&value, "packages/app", "react")?["status"], "used");
    assert_eq!(
        pkg_dep(&value, "packages/app", "@repo/shared")?["reason"],
        "workspace-internal"
    );
    Ok(())
}

#[test]
fn unavailable_when_dependency_import_consumer_support_is_missing() -> Result<()> {
    let artifact = build_unused_deps_artifact(request(
        json!([{
            "root": "C:/repo",
            "relRoot": ".",
            "packageJson": {
                "name": "app",
                "dependencies": { "react": "^19.0.0" }
            }
        }]),
        json!({
            "meta": { "supports": {} },
            "dependencyImportConsumers": []
        }),
    )?)?;
    let value = serde_json::to_value(artifact)?;

    assert_eq!(value["status"], "unavailable");
    assert_eq!(value["reason"], "input-artifact-missing");
    assert_eq!(value["summary"]["declaredDependencyCount"], 0);
    assert_eq!(value["packages"], json!([]));
    Ok(())
}

#[test]
fn cli_unused_deps_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-unused-deps-producer-request.v1",
            "root": "C:/repo",
            "includeTests": true,
            "exclude": [],
            "packageRecords": [{
                "root": "C:/repo",
                "relRoot": ".",
                "packageJson": {
                    "name": "app",
                    "dependencies": { "left-pad": "^1.3.0" }
                }
            }],
            "symbols": {
                "meta": { "supports": { "dependencyImportConsumers": true } },
                "dependencyImportConsumers": []
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("unused-deps-artifact")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&result)
        .output()?;

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let artifact: Value = serde_json::from_slice(&fs::read(&result)?)?;
    assert_eq!(artifact["schemaVersion"], "unused-deps.v1");
    assert_eq!(artifact["summary"]["reviewUnusedCount"], 1);
    Ok(())
}

fn dep<'a>(artifact: &'a Value, name: &str) -> Result<&'a Value> {
    pkg_dep(artifact, ".", name)
}

fn pkg_dep<'a>(artifact: &'a Value, package_dir: &str, name: &str) -> Result<&'a Value> {
    let packages = artifact["packages"]
        .as_array()
        .context("packages should be an array")?;
    let package = packages
        .iter()
        .find(|package| package["packageDir"] == package_dir)
        .with_context(|| format!("package '{package_dir}' should exist"))?;
    let dependencies = package["dependencies"]
        .as_array()
        .with_context(|| format!("package '{package_dir}' dependencies should be an array"))?;
    dependencies
        .iter()
        .find(|dependency| dependency["name"] == name)
        .with_context(|| format!("dependency '{name}' should exist in package '{package_dir}'"))
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
