use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
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

fn value_at_path<'a>(value: &'a Value, path: &[&str]) -> Result<&'a Value> {
    let mut current = value;
    for part in path {
        current = match current {
            Value::Array(items) => {
                let index = part
                    .parse::<usize>()
                    .map_err(|_| anyhow::anyhow!("array path segment must be numeric: {part}"))?;
                items
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("missing array index {index}"))?
            }
            Value::Object(object) => object
                .get(*part)
                .ok_or_else(|| anyhow::anyhow!("missing object path segment {part}"))?,
            _ => bail!("cannot descend into scalar value at path segment {part}"),
        };
    }
    Ok(current)
}

fn case_by_name<'a>(cases: &'a Value, name: &str) -> Result<&'a Value> {
    cases
        .as_array()
        .and_then(|cases| {
            cases
                .iter()
                .find(|case| case.get("name").and_then(Value::as_str) == Some(name))
        })
        .ok_or_else(|| anyhow::anyhow!("missing case {name}: {cases}"))
}

fn shared_fixture_cases_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../tests/fixtures/audit-core-blind-zones/cases.json")
}

#[test]
fn shared_parity_fixture_cases_cover_expected_blind_zones() -> Result<()> {
    let cases = serde_json::from_slice::<Value>(&fs::read(shared_fixture_cases_path())?)?;
    let cases = cases
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("blind-zone cases fixture must be an array"))?;

    for case in cases {
        let name = case
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("<unnamed>");
        let input = case
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("{name}: missing input"))?;
        let zones = serialized_zones(BlindZoneInput {
            triage: input.get("triage"),
            symbols: input.get("symbols"),
            dead_classify: input.get("deadClassify"),
            entry_surface: input.get("entrySurface"),
            resolver_diagnostics: input.get("resolverDiagnostics"),
            rust_analysis: input.get("rustAnalysis"),
        })?;
        for expected in case
            .get("expectedZones")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[])
        {
            let area = expected
                .get("area")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("{name}: expected zone missing area"))?;
            let zone =
                zone_by_area(&zones, area).map_err(|error| anyhow::anyhow!("{name}: {error}"))?;
            if let Some(severity) = expected.get("severity").and_then(Value::as_str) {
                assert_eq!(
                    zone.get("severity").and_then(Value::as_str),
                    Some(severity),
                    "{name}: wrong severity for {area}: {zones}"
                );
            }
        }
        for expected in case
            .get("expectedDetails")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[])
        {
            let area = expected
                .get("area")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("{name}: expected detail missing area"))?;
            let path_values = expected
                .get("path")
                .and_then(Value::as_array)
                .ok_or_else(|| anyhow::anyhow!("{name}: expected detail missing path"))?;
            let mut path = Vec::new();
            for part in path_values {
                path.push(part.as_str().ok_or_else(|| {
                    anyhow::anyhow!("{name}: expected detail path segments must be strings")
                })?);
            }
            let expected_value = expected
                .get("equals")
                .ok_or_else(|| anyhow::anyhow!("{name}: expected detail missing equals"))?;
            let zone =
                zone_by_area(&zones, area).map_err(|error| anyhow::anyhow!("{name}: {error}"))?;
            let actual = value_at_path(zone, &path)
                .map_err(|error| anyhow::anyhow!("{name}: {area}.{path:?}: {error}"))?;
            assert_eq!(
                actual, expected_value,
                "{name}: wrong detail for {area}.{path:?}: {zones}"
            );
        }
        for area in case
            .get("absentAreas")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .filter_map(Value::as_str)
        {
            assert!(
                zone_by_area(&zones, area).is_err(),
                "{name}: unexpectedly emitted {area}: {zones}"
            );
        }
    }
    Ok(())
}

#[test]
fn rust_scan_gap_depends_on_current_run_rust_analysis() -> Result<()> {
    let triage = json!({ "byLanguage": { "ts": 3, "rs": 2 } });
    let zones = serialized_zones(BlindZoneInput {
        triage: Some(&triage),
        ..empty_input()
    })?;
    let rust = zone_by_area(&zones, "rs")?;
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
        zone_by_area(&zones, "rs").is_err(),
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
fn cli_blind_zones_summary_emits_shared_case_batch_json() -> Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("blind-zones-summary")
        .arg("--cases")
        .arg(shared_fixture_cases_path())
        .output()?;

    if !output.status.success() {
        bail!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = serde_json::from_slice::<Value>(&output.stdout)?;
    let clean = case_by_name(&stdout, "clean-ts-js")?;
    assert_eq!(clean["blindZones"], json!([]));

    let rust_gap = case_by_name(&stdout, "rust-scan-gap-without-current-rust-analysis")?;
    assert_eq!(
        zone_by_area(&rust_gap["blindZones"], "rs")?["severity"],
        "scan-gap"
    );

    let parser_gap = case_by_name(&stdout, "parser-cjs-html-precision-gaps")?;
    assert_eq!(
        zone_by_area(&parser_gap["blindZones"], "commonjs-dynamic-require")?["severity"],
        "precision-gap"
    );
    Ok(())
}

#[test]
fn cli_blind_zones_summary_hard_stops_on_broken_case_batch() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let cases_path = temp.path().join("blind-zone-cases.json");
    fs::write(&cases_path, r#"[{"name":"broken"}]"#)?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("blind-zones-summary")
        .arg("--cases")
        .arg(&cases_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("blind-zones-summary: case 'broken' missing input"));
    Ok(())
}

#[test]
fn cli_blind_zones_summary_rejects_input_and_cases_together() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("blind-zones-fixture.json");
    fs::write(&input_path, "{}")?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("blind-zones-summary")
        .arg("--input")
        .arg(&input_path)
        .arg("--cases")
        .arg(shared_fixture_cases_path())
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("use either --input"));
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
