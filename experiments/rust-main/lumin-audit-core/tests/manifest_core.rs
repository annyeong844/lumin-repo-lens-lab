use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

use lumin_audit_core::manifest_core::{summarize_manifest_core, ManifestCoreOptions};

#[test]
fn manifest_core_summary_preserves_scan_range_confidence_and_sfc_evidence() -> Result<()> {
    let triage = json!({
        "summary": {
            "files": 42
        },
        "shape": {
            "tsFiles": 10,
            "jsFiles": 2,
            "pyFiles": 1,
            "goFiles": 0,
            "rustFiles": 3
        }
    });
    let symbols = json!({
        "meta": {
            "warnings": [
                { "kind": "other-warning", "count": 99 },
                { "code": "parse-errors", "count": 2 }
            ]
        },
        "filesWithParseErrors": ["fallback.ts"],
        "uses": {
            "external": 12,
            "resolvedInternal": 8,
            "unresolvedInternal": 3,
            "unresolvedInternalRatio": 0.2727,
            "sfcScriptConsumers": 2,
            "sfcScriptSrcReachability": 1,
            "sfcStyleAssetReferences": 3,
            "sfcTemplateComponentRefs": 4,
            "sfcGlobalComponentRegistrations": 5,
            "sfcGeneratedComponentManifests": 6,
            "sfcFrameworkConventionComponents": 7
        }
    });

    let summary = serde_json::to_value(summarize_manifest_core(
        ManifestCoreOptions {
            root: "C:/repo".to_string(),
            include_tests: false,
            production: true,
            excludes: vec!["dist".to_string(), "coverage".to_string()],
            auto_excludes: vec![".audit".to_string()],
        },
        Some(&triage),
        Some(&symbols),
    ))?;

    assert_eq!(
        summary["scanRange"],
        json!({
            "root": "C:/repo",
            "includeTests": false,
            "production": true,
            "excludes": ["dist", "coverage"],
            "autoExcludes": [".audit"],
            "languages": ["ts", "js", "py", "rs"],
            "files": 42
        })
    );
    assert_eq!(
        summary["confidence"],
        json!({
            "parseErrors": 2,
            "unresolvedInternalRatio": 0.2727,
            "externalImports": 12,
            "resolvedInternal": 8,
            "unresolvedInternal": 3
        })
    );
    assert_eq!(
        summary["sfcEvidence"],
        json!({
            "artifact": "symbols.json",
            "status": "complete",
            "scriptImportConsumerCount": 2,
            "reachabilityOnlyCount": 1,
            "reviewOnlyEvidenceCount": 25,
            "totalEvidenceCount": 28,
            "byLane": {
                "scriptImportConsumers": 2,
                "scriptSrcReachability": 1,
                "styleAssetReferences": 3,
                "templateComponentRefs": 4,
                "globalComponentRegistrations": 5,
                "generatedComponentManifests": 6,
                "frameworkConventionComponents": 7
            },
            "scanGapStillApplies": true
        })
    );
    Ok(())
}

#[test]
fn manifest_core_summary_falls_back_for_missing_triage_and_parse_warning_count() -> Result<()> {
    let triage = json!({
        "files": 12,
        "shape": {
            "totalFiles": 99,
            "tsFiles": 1
        }
    });
    let symbols = json!({
        "meta": {
            "warnings": [
                { "type": "parse-errors" }
            ]
        },
        "filesWithParseErrors": ["a.ts", "b.ts"],
        "uses": {
            "sfcScriptConsumers": "not-a-number"
        }
    });

    let summary = serde_json::to_value(summarize_manifest_core(
        ManifestCoreOptions {
            root: ".".to_string(),
            include_tests: true,
            production: false,
            excludes: Vec::new(),
            auto_excludes: Vec::new(),
        },
        Some(&triage),
        Some(&symbols),
    ))?;

    assert_eq!(summary["scanRange"]["files"], 12);
    assert_eq!(summary["scanRange"]["languages"], json!(["ts"]));
    assert_eq!(summary["confidence"]["parseErrors"], 2);
    assert_eq!(summary["confidence"]["externalImports"], json!(null));
    assert_eq!(summary["sfcEvidence"], json!(null));
    Ok(())
}

#[test]
fn manifest_core_summary_uses_by_language_keys_before_shape() -> Result<()> {
    let triage = json!({
        "summary": {
            "byLanguage": {
                "astro": 1,
                "svelte": 2,
                "vue": 3
            }
        },
        "shape": {
            "tsFiles": 10
        }
    });

    let summary = serde_json::to_value(summarize_manifest_core(
        ManifestCoreOptions {
            root: ".".to_string(),
            include_tests: true,
            production: false,
            excludes: Vec::new(),
            auto_excludes: Vec::new(),
        },
        Some(&triage),
        None,
    ))?;

    assert_eq!(
        summary["scanRange"]["languages"],
        json!(["astro", "svelte", "vue"])
    );
    assert_eq!(summary["scanRange"]["files"], json!(null));
    assert_eq!(summary["confidence"]["parseErrors"], 0);
    assert_eq!(summary["sfcEvidence"], json!(null));
    Ok(())
}

#[test]
fn cli_manifest_core_summary_reads_optional_artifacts() -> Result<()> {
    let tempdir = tempfile::tempdir()?;
    let triage_path = tempdir.path().join("triage.json");
    let symbols_path = tempdir.path().join("symbols.json");
    fs::write(
        &triage_path,
        serde_json::to_vec(&json!({
            "shape": {
                "totalFiles": 4,
                "jsFiles": 1
            }
        }))?,
    )?;
    fs::write(
        &symbols_path,
        serde_json::to_vec(&json!({
            "uses": {
                "external": 1,
                "resolvedInternal": 2,
                "unresolvedInternal": 0,
                "unresolvedInternalRatio": 0,
                "sfcTemplateComponentRefs": 2
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-core-summary")
        .arg("--root")
        .arg("C:/repo")
        .arg("--triage")
        .arg(&triage_path)
        .arg("--symbols")
        .arg(&symbols_path)
        .arg("--no-include-tests")
        .arg("--production")
        .arg("--exclude")
        .arg("dist")
        .arg("--auto-exclude")
        .arg(".audit")
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["scanRange"]["root"], "C:/repo");
    assert_eq!(stdout["scanRange"]["includeTests"], false);
    assert_eq!(stdout["scanRange"]["production"], true);
    assert_eq!(stdout["scanRange"]["excludes"], json!(["dist"]));
    assert_eq!(stdout["scanRange"]["autoExcludes"], json!([".audit"]));
    assert_eq!(stdout["scanRange"]["languages"], json!(["js"]));
    assert_eq!(stdout["confidence"]["externalImports"], 1);
    assert_eq!(stdout["sfcEvidence"]["totalEvidenceCount"], 2);
    Ok(())
}
