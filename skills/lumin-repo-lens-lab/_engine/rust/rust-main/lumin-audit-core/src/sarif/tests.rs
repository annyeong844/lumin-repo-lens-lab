use super::*;
use serde_json::json;

#[test]
fn fix_plan_tiers_drive_ga001_sarif_levels_and_skip_muted() -> Result<()> {
    let artifact = build_sarif_artifact(SarifRequest {
        schema_version: SARIF_REQUEST_SCHEMA_VERSION.to_string(),
        root: "C:/repo".to_string(),
        generated: Some("2026-07-04T00:00:00.000Z".to_string()),
        fix_plan: Some(json!({
            "safeFixes": [{
                "finding": { "file": "src/safe.ts", "line": 10, "symbol": "SafeSym", "kind": "FunctionDeclaration", "bucket": "C" },
                "evidence": { "runtime": { "status": "dead-confirmed", "grounding": "grounded", "confidence": "high", "hitsInSymbol": 0 } },
                "reason": "runtime-dead-confirmed"
            }],
            "reviewFixes": [{
                "finding": { "file": "src/review.ts", "line": 20, "symbol": "ReviewSym", "kind": "FunctionDeclaration", "bucket": "A", "fileInternalUses": 2 },
                "evidence": {},
                "reason": "manual-review"
            }],
            "degraded": [{
                "finding": { "file": "src/deg.ts", "line": 30, "symbol": "DegSym", "kind": "FunctionDeclaration", "bucket": "C" },
                "evidence": { "runtime": { "status": "executed", "grounding": "grounded", "confidence": "high", "hitsInSymbol": 7 } },
                "reason": "runtime-executed"
            }],
            "muted": [{
                "finding": { "file": "eslint.config.mjs", "line": 1, "symbol": "default", "kind": "default" },
                "reason": "policy-excluded"
            }]
        })),
        runtime_evidence: None,
        staleness: None,
        dead_classify: None,
        symbols: None,
        topology: None,
        discipline: None,
        barrels: None,
    })?;

    let results = artifact["runs"][0]["results"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("SARIF results must be an array"))?;
    assert_eq!(results.len(), 3);
    assert_eq!(results[0]["level"], "warning");
    assert_eq!(results[1]["level"], "note");
    assert_eq!(results[2]["properties"]["hitsInSymbol"], 7);
    assert_eq!(
        artifact["runs"][0]["properties"]["artifactsUsed"],
        json!(["fix-plan.json"])
    );
    Ok(())
}

#[test]
fn topology_discipline_and_barrel_artifacts_project_secondary_rules() -> Result<()> {
    let artifact = build_sarif_artifact(SarifRequest {
        schema_version: SARIF_REQUEST_SCHEMA_VERSION.to_string(),
        root: "C:/repo".to_string(),
        generated: Some("2026-07-04T00:00:00.000Z".to_string()),
        fix_plan: None,
        runtime_evidence: None,
        staleness: None,
        dead_classify: None,
        symbols: None,
        topology: Some(json!({
            "sccs": [{ "size": 2, "members": ["src/a.ts", "src/b.ts"] }],
            "largestFiles": [{ "file": "src/huge.ts", "loc": 1200 }],
            "crossSubmoduleTop": [{ "edge": "a -> b", "count": 30 }]
        })),
        discipline: Some(json!({
            "overallTopOffenders": [{ "file": "src/a.ts", "breakdown": { "as any": 2, "ignored": 0 } }]
        })),
        barrels: Some(json!({
            "byPackage": {
                "@scope/pkg": {
                    "sampleRootImporters": [
                        { "file": "src/c.ts", "line": 4, "symbols": ["x"], "reExport": false }
                    ]
                }
            }
        })),
    })?;
    let results = artifact["runs"][0]["results"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("SARIF results must be an array"))?;
    let rule_ids: Vec<_> = results
        .iter()
        .filter_map(|result| result["ruleId"].as_str())
        .collect();
    assert!(rule_ids.contains(&"GA002"));
    assert!(rule_ids.contains(&"GA003"));
    assert!(rule_ids.contains(&"GA004"));
    assert!(rule_ids.contains(&"GA005"));
    assert!(rule_ids.contains(&"GA006"));
    Ok(())
}

#[test]
fn rejects_bad_request_schema() {
    let result = build_sarif_artifact(SarifRequest {
        schema_version: "wrong".to_string(),
        root: "C:/repo".to_string(),
        generated: None,
        fix_plan: None,
        runtime_evidence: None,
        staleness: None,
        dead_classify: None,
        symbols: None,
        topology: None,
        discipline: None,
        barrels: None,
    });
    assert!(result.is_err());
}
