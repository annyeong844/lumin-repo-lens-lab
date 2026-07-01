use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use lumin_audit_core::blind_zones::{summarize_blind_zones, BlindZoneInput};

fn empty_input<'a>() -> BlindZoneInput<'a> {
    BlindZoneInput {
        triage: None,
        symbols: None,
        dead_classify: None,
        entry_surface: None,
        resolver_diagnostics: None,
        rust_analysis: None,
    }
}

fn serialized_zones(input: BlindZoneInput<'_>) -> Result<Value> {
    Ok(serde_json::to_value(summarize_blind_zones(input))?)
}

fn zone_by_area<'a>(zones: &'a Value, area: &str) -> Result<&'a Value> {
    zones
        .as_array()
        .and_then(|zones| {
            zones
                .iter()
                .find(|zone| zone.get("area").and_then(Value::as_str) == Some(area))
        })
        .ok_or_else(|| anyhow::anyhow!("missing zone area {area}: {zones}"))
}

#[test]
fn rust_scan_gap_depends_on_current_run_rust_analysis() -> Result<()> {
    let triage = json!({ "byLanguage": { "ts": 3, "rs": 2 } });
    let zones = serialized_zones(BlindZoneInput {
        triage: Some(&triage),
        ..empty_input()
    })?;
    let rust = zone_by_area(&zones, "rust")?;
    assert_eq!(rust["severity"], "scan-gap");
    assert!(rust["effect"]
        .as_str()
        .unwrap_or_default()
        .contains("absence claims"));

    let rust_analysis = json!({ "status": "complete", "available": true });
    let zones = serialized_zones(BlindZoneInput {
        triage: Some(&triage),
        rust_analysis: Some(&rust_analysis),
        ..empty_input()
    })?;
    assert!(
        zone_by_area(&zones, "rust").is_err(),
        "complete Rust analysis should clear rust scan-gap: {zones}"
    );
    Ok(())
}

#[test]
fn sfc_files_are_grouped_without_per_extension_noise() -> Result<()> {
    let triage = json!({
        "shape": {
            "totalFiles": 4,
            "tsFiles": 1,
            "jsFiles": 0,
            "pyFiles": 0,
            "goFiles": 0,
            "sfcFiles": 3
        },
        "byLanguage": { "ts": 1, "vue": 1, "svelte": 1, "astro": 1 }
    });
    let zones = serialized_zones(BlindZoneInput {
        triage: Some(&triage),
        ..empty_input()
    })?;
    let sfc = zone_by_area(&zones, "sfc-scan-gap")?;
    assert_eq!(sfc["severity"], "scan-gap");
    assert_eq!(sfc["details"]["files"], 3);
    assert_eq!(sfc["details"]["languages"]["vue"], 1);
    assert!(zone_by_area(&zones, "vue").is_err());
    assert!(zone_by_area(&zones, "svelte").is_err());
    assert!(zone_by_area(&zones, "astro").is_err());
    Ok(())
}

#[test]
fn python_and_go_unavailable_extractors_are_scan_gaps() -> Result<()> {
    let triage = json!({
        "shape": {
            "totalFiles": 4,
            "tsFiles": 2,
            "jsFiles": 0,
            "pyFiles": 1,
            "goFiles": 1
        }
    });
    let symbols = json!({
        "meta": {
            "languageSupport": {
                "python": { "enabled": false, "reason": "python executable unavailable" },
                "go": { "enabled": false, "reason": "tree-sitter unavailable" }
            }
        }
    });
    let zones = serialized_zones(BlindZoneInput {
        triage: Some(&triage),
        symbols: Some(&symbols),
        ..empty_input()
    })?;
    let py = zone_by_area(&zones, "python-scan-gap")?;
    let go = zone_by_area(&zones, "go-scan-gap")?;
    assert_eq!(py["severity"], "scan-gap");
    assert_eq!(py["details"]["reason"], "python executable unavailable");
    assert_eq!(go["severity"], "scan-gap");
    assert_eq!(go["details"]["reason"], "tree-sitter unavailable");
    Ok(())
}

#[test]
fn resolver_confidence_gap_uses_threshold_policy_and_grouped_reasons() -> Result<()> {
    let symbols = json!({
        "uses": { "unresolvedInternalRatio": 0.06, "unresolvedInternal": 1300 },
        "topUnresolvedSpecifiers": [{ "specifierPrefix": "@workspace/", "count": 800 }],
        "unresolvedInternalSummaryByReason": {
            "workspace-package-subpath-target-missing": {
                "count": 12,
                "spaces": { "type": 12, "value": 0, "unknown": 0 },
                "resolverStages": { "workspacePackageSubpath": 12 }
            },
            "tsconfig-path-target-missing": {
                "count": 4,
                "spaces": { "type": 1, "value": 3, "unknown": 0 },
                "hints": { "generated-artifact-missing": 4 }
            }
        }
    });
    let zones = serialized_zones(BlindZoneInput {
        symbols: Some(&symbols),
        ..empty_input()
    })?;
    let resolver = zone_by_area(&zones, "resolver")?;
    assert_eq!(resolver["severity"], "confidence-gap");
    assert_eq!(resolver["details"]["trigger"], "absolute-count");
    assert_eq!(
        resolver["details"]["thresholdPolicy"]["policyId"],
        "resolver-blind-zone-policy"
    );
    assert_eq!(
        resolver["details"]["thresholdPolicy"]["policyVersion"],
        "resolver-blind-zone-policy-v1"
    );
    assert_eq!(
        resolver["details"]["topUnresolvedReasons"][0]["reason"],
        "workspace-package-subpath-target-missing"
    );
    assert_eq!(
        resolver["details"]["topUnresolvedReasons"][0]["spaces"]["type"],
        12
    );
    Ok(())
}

#[test]
fn precision_gap_branches_remain_structured() -> Result<()> {
    let symbols = json!({
        "meta": { "warnings": [{ "kind": "parse-errors", "count": 2, "message": "parse failed" }] },
        "cjsExportSurfaceByFile": {
            "src/exact.cjs": { "exact": [{ "name": "foo" }], "opaque": [] },
            "src/opaque.cjs": { "exact": [], "opaque": [{ "kind": "module-exports-assignment" }] }
        },
        "cjsRequireOpacity": [
            { "consumerFile": "src/consumer.js", "line": 2, "kind": "dynamic-require" }
        ]
    });
    let entry_surface = json!({
        "unresolvedHtmlEntrypoints": [
            { "htmlFile": "index.html", "src": "./missing.js", "resolvedFile": "missing.js" }
        ]
    });
    let zones = serialized_zones(BlindZoneInput {
        symbols: Some(&symbols),
        entry_surface: Some(&entry_surface),
        ..empty_input()
    })?;
    assert_eq!(zone_by_area(&zones, "parser")?["severity"], "precision-gap");
    assert_eq!(
        zone_by_area(&zones, "commonjs-export-surface")?["details"]["files"],
        1
    );
    assert_eq!(
        zone_by_area(&zones, "commonjs-dynamic-require")?["details"]["calls"],
        1
    );
    assert_eq!(
        zone_by_area(&zones, "html-entry-surface")?["severity"],
        "confidence-gap"
    );
    Ok(())
}

#[test]
fn missing_inputs_do_not_invent_blind_zones() -> Result<()> {
    let zones = summarize_blind_zones(empty_input());
    assert!(
        zones.is_empty(),
        "missing artifacts should skip branches, not invent blind zones"
    );
    Ok(())
}

#[test]
fn cli_blind_zones_summary_emits_fixture_json() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let fixture_path = temp.path().join("blind-zones-fixture.json");
    fs::write(
        &fixture_path,
        serde_json::to_vec(&json!({
            "triage": { "byLanguage": { "py": 1 } },
            "symbols": {
                "meta": {
                    "languageSupport": {
                        "python": { "enabled": false, "reason": "python unavailable" }
                    }
                }
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("blind-zones-summary")
        .arg("--input")
        .arg(&fixture_path)
        .output()?;

    if !output.status.success() {
        bail!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = serde_json::from_slice::<Value>(&output.stdout)?;
    assert_eq!(
        zone_by_area(&stdout, "python-scan-gap")?["severity"],
        "scan-gap"
    );
    Ok(())
}

#[test]
fn cli_blind_zones_summary_hard_stops_on_malformed_fixture() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let fixture_path = temp.path().join("blind-zones-fixture.json");
    fs::write(&fixture_path, "{not-json")?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("blind-zones-summary")
        .arg("--input")
        .arg(&fixture_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("blind-zones-summary: invalid JSON"));
    assert!(stderr.contains("blind-zones-fixture.json"));
    Ok(())
}
