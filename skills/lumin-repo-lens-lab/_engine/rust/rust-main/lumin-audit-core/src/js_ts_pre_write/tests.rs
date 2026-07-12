use anyhow::{bail, Result};
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use super::projection::TYPE_ESCAPE_KINDS;
use super::*;

#[test]
fn builds_compact_pre_write_evidence_without_repository_artifacts() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("src/dep.ts"),
        "export function live() { return 1; }\nexport const unused = 2;\n",
    )?;
    fs::write(
        root.join("src/app.ts"),
        "import { live } from './dep';\nimport react from 'react';\nimport aliasValue from 'lib/alias';\nexport const app = 1 as any;\n",
    )?;
    fs::write(root.join("src/side.ts"), "import './dep';\n")?;
    let request = JsTsPreWriteEvidenceRequest {
        schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
        root: root.to_path_buf(),
        evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
        any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
        generated: "2026-07-11T00:00:00.000Z".to_string(),
        include_tests: true,
        excludes: vec!["vendor".to_string()],
        dependency_roots: vec!["react".to_string()],
        discover_files: false,
        files: vec![
            source_file(root, "src/app.ts"),
            source_file(root, "src/dep.ts"),
            source_file(root, "src/side.ts"),
        ],
        incremental: Default::default(),
    };

    let evidence = build_js_ts_pre_write_evidence(request)?;
    assert_eq!(
        evidence["schemaVersion"],
        JS_TS_PRE_WRITE_EVIDENCE_RESPONSE_SCHEMA_VERSION
    );
    assert_eq!(
        evidence["symbols"]["defIndex"]["src/dep.ts"]["live"]["line"],
        1
    );
    assert_eq!(
        evidence["symbols"]["fanInByIdentity"]["src/dep.ts::live"],
        1
    );
    assert_eq!(
        evidence["symbols"]["fanInByIdentity"]["src/dep.ts::unused"],
        0
    );
    assert_eq!(
        evidence["symbols"]["fanInByIdentitySpace"]["src/dep.ts::unused"]["broad"],
        1
    );
    assert_eq!(
        evidence["symbols"]["defIndex"]["src/app.ts"]["app"]["anyContamination"]["label"],
        "any-contaminated"
    );
    assert_eq!(
        evidence["symbols"]["helperOwnersByIdentity"]["src/app.ts::app"]["anyContamination"]
            ["label"],
        "any-contaminated"
    );
    assert_eq!(
        evidence["symbols"]["dependencyImportConsumers"][0]["depRoot"],
        "react"
    );
    assert_eq!(
        evidence["symbols"]["dependencyImportConsumers"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(evidence["topology"]["edges"][0]["to"], "src/dep.ts");
    assert_eq!(evidence["anyInventory"]["meta"]["complete"], true);
    assert_eq!(
        evidence["anyInventory"]["meta"]["incremental"]["singleFlight"]["status"],
        "acquired"
    );
    assert_eq!(
        evidence["anyInventory"]["meta"]["incremental"]["singleFlight"]["scope"],
        "canonical-root"
    );
    for field in [
        "lockWaitMs",
        "discoveryMs",
        "parseMs",
        "projectionMs",
        "scanHeldMs",
        "totalRuntimeMs",
    ] {
        if !evidence["anyInventory"]["meta"]["incremental"]["timing"][field].is_u64() {
            bail!("missing numeric pre-write runtime timing field {field}");
        }
    }
    assert_eq!(
        evidence["summary"]["runtime"]["singleFlight"],
        evidence["anyInventory"]["meta"]["incremental"]["singleFlight"]
    );
    assert_eq!(
        evidence["anyInventory"]["meta"]["artifact"],
        "any-inventory.pre.PROBE.json"
    );
    assert_eq!(
        evidence["anyInventory"]["meta"]["supports"]["escapeKinds"],
        json!(TYPE_ESCAPE_KINDS)
    );
    assert_eq!(
        evidence["anyInventory"]["typeEscapes"][0]["escapeKind"],
        "as-any"
    );
    Ok(())
}

#[test]
fn parse_errors_degrade_every_shared_projection_without_claiming_absence() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join("src/broken.ts"), "export const = ;\n")?;
    let evidence = build_js_ts_pre_write_evidence(JsTsPreWriteEvidenceRequest {
        schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
        root: root.to_path_buf(),
        evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
        any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
        generated: "2026-07-11T00:00:00.000Z".to_string(),
        include_tests: false,
        excludes: Vec::new(),
        dependency_roots: Vec::new(),
        discover_files: false,
        files: vec![source_file(root, "src/broken.ts")],
        incremental: Default::default(),
    })?;

    assert_eq!(evidence["symbols"]["meta"]["complete"], false);
    assert_eq!(evidence["topology"]["meta"]["complete"], false);
    assert_eq!(evidence["anyInventory"]["meta"]["complete"], false);
    assert_eq!(
        evidence["anyInventory"]["meta"]["filesWithParseErrors"][0]["file"],
        "src/broken.ts"
    );
    Ok(())
}

#[test]
fn unreadable_required_sources_hard_stop() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    let result = build_js_ts_pre_write_evidence(JsTsPreWriteEvidenceRequest {
        schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
        root: root.to_path_buf(),
        evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
        any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
        generated: "2026-07-11T00:00:00.000Z".to_string(),
        include_tests: true,
        excludes: Vec::new(),
        dependency_roots: Vec::new(),
        discover_files: false,
        files: vec![source_file(root, "src/missing.ts")],
        incremental: Default::default(),
    });
    let Err(error) = result else {
        bail!("missing required source did not hard-stop");
    };

    assert!(error.to_string().contains("failed to read required source"));
    Ok(())
}

#[test]
fn rejects_paths_outside_the_declared_root() -> Result<()> {
    let root = tempdir()?;
    let outside = tempdir()?;
    let request = JsTsPreWriteEvidenceRequest {
        schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
        root: root.path().to_path_buf(),
        evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
        any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
        generated: "2026-07-11T00:00:00.000Z".to_string(),
        include_tests: true,
        excludes: Vec::new(),
        dependency_roots: Vec::new(),
        discover_files: false,
        files: vec![JsTsPreWriteSourceFile {
            file_path: outside.path().join("outside.ts"),
            artifact_file_path: "outside.ts".to_string(),
        }],
        incremental: Default::default(),
    };
    let result = build_js_ts_pre_write_evidence(request);
    let Err(error) = result else {
        bail!("outside path did not fail");
    };
    assert!(error.to_string().contains("inside root"));
    Ok(())
}

#[test]
fn rejects_lexically_nested_path_that_canonicalizes_outside_root() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path().join("repo");
    fs::create_dir_all(&root)?;
    fs::write(
        temp.path().join("outside.ts"),
        "export const outside = true;\n",
    )?;
    let request = JsTsPreWriteEvidenceRequest {
        schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
        root: root.clone(),
        evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
        any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
        generated: "2026-07-11T00:00:00.000Z".to_string(),
        include_tests: true,
        excludes: Vec::new(),
        dependency_roots: Vec::new(),
        discover_files: false,
        files: vec![JsTsPreWriteSourceFile {
            file_path: root.join("../outside.ts"),
            artifact_file_path: "outside.ts".to_string(),
        }],
        incremental: JsTsPreWriteIncrementalRequest {
            enabled: true,
            cache_root: Some(temp.path().join("cache")),
            clear: false,
        },
    };
    let result = build_js_ts_pre_write_evidence(request);
    let Err(error) = result else {
        bail!("canonical path escape did not fail");
    };
    assert!(error.to_string().contains("inside root"));
    Ok(())
}

#[test]
fn discovers_the_checked_production_scope_before_parsing() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join("src/app.ts"), "export const app = true;\n")?;
    fs::write(
        root.join("src/app.test.ts"),
        "export const testOnly = true;\n",
    )?;

    let evidence = build_js_ts_pre_write_evidence(JsTsPreWriteEvidenceRequest {
        schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
        root: root.to_path_buf(),
        evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
        any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
        generated: "2026-07-11T00:00:00.000Z".to_string(),
        include_tests: false,
        excludes: Vec::new(),
        dependency_roots: Vec::new(),
        discover_files: true,
        files: Vec::new(),
        incremental: Default::default(),
    })?;

    assert_eq!(evidence["files"], json!(["src/app.ts"]));
    assert_eq!(evidence["summary"]["fileCount"], 1);
    assert!(evidence["symbols"]["defIndex"]
        .get("src/app.test.ts")
        .is_none());
    Ok(())
}

#[test]
fn strict_cache_reuses_only_byte_identical_files_and_rebuilds_current_evidence() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    let cache_root = root.join(".cache");
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join("src/a.ts"), "export const a = 1;\n")?;
    fs::write(root.join("src/b.ts"), "export const b = 1;\n")?;

    let build = || {
        build_js_ts_pre_write_evidence(JsTsPreWriteEvidenceRequest {
            schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
            root: root.to_path_buf(),
            evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
            any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
            generated: "2026-07-11T00:00:00.000Z".to_string(),
            include_tests: true,
            excludes: Vec::new(),
            dependency_roots: Vec::new(),
            discover_files: true,
            files: Vec::new(),
            incremental: JsTsPreWriteIncrementalRequest {
                enabled: true,
                cache_root: Some(cache_root.clone()),
                clear: false,
            },
        })
    };

    let cold = build()?;
    assert_eq!(
        cold["anyInventory"]["meta"]["incremental"]["changedFiles"],
        2
    );
    assert_eq!(
        cold["anyInventory"]["meta"]["incremental"]["reusedFiles"],
        0
    );
    assert_eq!(
        cold["anyInventory"]["meta"]["incremental"]["identityMode"],
        "sha256"
    );
    assert_eq!(
        cold["anyInventory"]["meta"]["incremental"]["contentHashFiles"],
        2
    );
    assert_eq!(
        cold["anyInventory"]["meta"]["incremental"]["gitBlobFiles"],
        0
    );

    let warm = build()?;
    assert_eq!(
        warm["anyInventory"]["meta"]["incremental"]["changedFiles"],
        0
    );
    assert_eq!(
        warm["anyInventory"]["meta"]["incremental"]["reusedFiles"],
        2
    );
    assert_eq!(
        warm["anyInventory"]["meta"]["incremental"]["writeStatus"],
        "unchanged"
    );
    assert!(warm["anyInventory"]["meta"]["incremental"]["timing"]["sourceReadHashMs"].is_u64());
    assert!(warm["summary"]["runtime"]["timing"]["scanHeldMs"].is_u64());
    assert_eq!(warm["symbols"]["defIndex"]["src/a.ts"]["a"]["line"], 1);

    fs::write(root.join("src/a.ts"), "export const a = 2;\n")?;
    let changed = build()?;
    assert_eq!(
        changed["anyInventory"]["meta"]["incremental"]["changedFiles"],
        1
    );
    assert_eq!(
        changed["anyInventory"]["meta"]["incremental"]["reusedFiles"],
        1
    );
    assert_eq!(changed["summary"]["fileCount"], 2);

    fs::write(root.join("src/c.ts"), "export const c = 1;\n")?;
    let expanded = build()?;
    assert_eq!(
        expanded["anyInventory"]["meta"]["incremental"]["changedFiles"],
        3
    );
    assert_eq!(
        expanded["anyInventory"]["meta"]["incremental"]["reusedFiles"],
        0
    );
    assert_eq!(expanded["summary"]["fileCount"], 3);
    Ok(())
}

#[test]
fn strict_cache_keys_identity_by_source_bytes_not_artifact_alias() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    let cache_root = root.join(".cache");
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join("src/real.ts"), "export const before = 1;\n")?;
    fs::write(root.join("alias.ts"), "export const unrelated = 1;\n")?;
    let build = || {
        build_js_ts_pre_write_evidence(JsTsPreWriteEvidenceRequest {
            schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
            root: root.to_path_buf(),
            evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
            any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
            generated: "2026-07-11T00:00:00.000Z".to_string(),
            include_tests: true,
            excludes: Vec::new(),
            dependency_roots: Vec::new(),
            discover_files: false,
            files: vec![JsTsPreWriteSourceFile {
                file_path: root.join("src/real.ts"),
                artifact_file_path: "alias.ts".to_string(),
            }],
            incremental: JsTsPreWriteIncrementalRequest {
                enabled: true,
                cache_root: Some(cache_root.clone()),
                clear: false,
            },
        })
    };

    let cold = build()?;
    assert_eq!(cold["symbols"]["defIndex"]["alias.ts"]["before"]["line"], 1);

    fs::write(root.join("src/real.ts"), "export const after = 2;\n")?;
    let changed = build()?;
    assert_eq!(
        changed["anyInventory"]["meta"]["incremental"]["changedFiles"],
        1
    );
    assert_eq!(
        changed["anyInventory"]["meta"]["incremental"]["reusedFiles"],
        0
    );
    assert_eq!(
        changed["symbols"]["defIndex"]["alias.ts"]["after"]["line"],
        1
    );
    assert!(changed["symbols"]["defIndex"]["alias.ts"]
        .get("before")
        .is_none());
    Ok(())
}

fn source_file(root: &Path, relative: &str) -> JsTsPreWriteSourceFile {
    JsTsPreWriteSourceFile {
        file_path: root.join(relative),
        artifact_file_path: relative.to_string(),
    }
}
