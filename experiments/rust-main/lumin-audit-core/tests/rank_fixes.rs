use anyhow::Result;
use lumin_audit_core::rank_fixes::{build_rank_fixes_artifact, RankFixesRequest};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_rank_fixes_artifact_rejects_missing_input() -> Result<()> {
    let output = Command::new(audit_core_bin())
        .arg("rank-fixes-artifact")
        .output()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.status.success());
    assert!(
        text.contains("rank-fixes-artifact: missing --input <path|->"),
        "{text}"
    );
    Ok(())
}

#[test]
fn cli_rank_fixes_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-rank-fixes-producer-request.v1",
            "root": "C:/repo",
            "generated": "2026-07-03T00:00:00.000Z",
            "artifacts": {
                "deadClassify": empty_dead_classify()
            },
            "publicDeepImportRiskByFile": { "__sentinel__": { "risk": false } }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("rank-fixes-artifact")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&result)
        .output()?;

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    let artifact: Value = serde_json::from_slice(&fs::read(&result)?)?;
    assert_eq!(artifact["meta"]["tool"], "rank-fixes.mjs");
    assert_eq!(artifact["summary"]["total"], 0);
    Ok(())
}

#[test]
fn rank_fixes_materializes_safe_review_degraded_and_muted_findings() -> Result<()> {
    let artifact = build_rank_fixes_artifact(rank_request(
        json!({
            "proposal_C_remove_symbol": [
                { "file": "src/safe.ts", "line": 1, "symbol": "Safe", "kind": "FunctionDeclaration", "action": "" }
            ],
            "proposal_A_demote_to_internal": [],
            "proposal_B_review": [
                { "file": "src/review.ts", "line": 2, "symbol": "Review", "kind": "FunctionDeclaration", "action": "" }
            ],
            "proposal_remove_export_specifier": [],
            "proposal_DEGRADED_unprocessed": [
                { "file": "src/bounded.ts", "line": 3, "symbol": "Bounded", "kind": "FunctionDeclaration", "action": "classification incomplete" }
            ],
            "excludedCandidates": [
                { "file": "src/public.ts", "line": 4, "symbol": "Public", "kind": "FunctionDeclaration", "reason": "publicApi_FP23" }
            ]
        }),
        Some(json!({
            "findings": [
                {
                    "id": "dead-export:src/safe.ts:Safe:1",
                    "safeAction": { "kind": "demote_export_declaration", "proofComplete": true, "actionBlockers": [] },
                    "actionBlockers": []
                },
                {
                    "id": "dead-export:src/review.ts:Review:2",
                    "safeAction": { "kind": "demote_export_declaration", "proofComplete": true, "actionBlockers": [] },
                    "actionBlockers": []
                }
            ]
        })),
        None,
        None,
        None,
        None,
        None,
    ))?;
    assert_eq!(artifact.summary["SAFE_FIX"], 1);
    assert_eq!(artifact.summary["REVIEW_FIX"], 1);
    assert_eq!(artifact.summary["DEGRADED"], 1);
    assert_eq!(artifact.summary["MUTED"], 1);
    assert_eq!(
        artifact.safe_fixes[0]["finding"]["id"],
        "dead-export:src/safe.ts:Safe:1"
    );
    assert_eq!(artifact.muted[0]["tier"], "MUTED");
    Ok(())
}

#[test]
fn rank_predicate_blocks_safe_fix_for_soft_taint_and_unknown_public_risk() -> Result<()> {
    let artifact = build_rank_fixes_artifact(rank_request(
        json!({
            "proposal_C_remove_symbol": [
                {
                    "file": "src/safe.ts",
                    "line": 1,
                    "symbol": "Safe",
                    "kind": "FunctionDeclaration",
                    "action": "",
                    "taintedBy": [{ "kind": "parse-errors-present", "file": "src/other.ts" }]
                },
                { "file": "src/missing-risk.ts", "line": 2, "symbol": "UnknownRisk", "kind": "FunctionDeclaration", "action": "" }
            ],
            "proposal_A_demote_to_internal": [],
            "proposal_B_review": [],
            "proposal_remove_export_specifier": [],
            "proposal_DEGRADED_unprocessed": [],
            "excludedCandidates": []
        }),
        Some(json!({
            "findings": [
                {
                    "id": "dead-export:src/safe.ts:Safe:1",
                    "safeAction": { "kind": "demote_export_declaration", "proofComplete": true, "actionBlockers": [] },
                    "actionBlockers": []
                },
                {
                    "id": "dead-export:src/missing-risk.ts:UnknownRisk:2",
                    "safeAction": { "kind": "demote_export_declaration", "proofComplete": true, "actionBlockers": [] },
                    "actionBlockers": []
                }
            ]
        })),
        None,
        None,
        None,
        None,
        None,
    ))?;
    assert_eq!(artifact.summary["SAFE_FIX"], 0);
    assert_eq!(artifact.summary["REVIEW_FIX"], 2);
    assert!(artifact.review_fixes.iter().any(|entry| entry["reason"]
        .as_str()
        .unwrap_or("")
        .contains("parse-errors-elsewhere")));
    assert!(artifact.review_fixes.iter().any(|entry| entry["reason"]
        .as_str()
        .unwrap_or("")
        .contains("public-deep-import-risk")));
    Ok(())
}

#[test]
fn entry_unreachable_support_requires_complete_unbounded_private_file() -> Result<()> {
    let artifact = build_rank_fixes_artifact(rank_request(
        json!({
            "proposal_C_remove_symbol": [
                { "file": "src/isolated.ts", "line": 1, "symbol": "Isolated", "kind": "FunctionDeclaration", "action": "" }
            ],
            "proposal_A_demote_to_internal": [],
            "proposal_B_review": [],
            "proposal_remove_export_specifier": [],
            "proposal_DEGRADED_unprocessed": [],
            "excludedCandidates": []
        }),
        Some(json!({
            "findings": [{
                "id": "dead-export:src/isolated.ts:Isolated:1",
                "safeAction": { "kind": "demote_export_declaration", "proofComplete": true, "actionBlockers": [] },
                "actionBlockers": []
            }]
        })),
        Some(json!({ "dynamicImportOpacity": [] })),
        None,
        Some(json!({
            "entryFiles": ["src/index.ts"],
            "completenessBySubmodule": { ".": "high" }
        })),
        Some(module_reachability_for_unreachable("src/isolated.ts")),
        Some(json!({ "src/isolated.ts": { "risk": false } })),
    ))?;
    assert_eq!(artifact.summary["SAFE_FIX"], 1);
    let safe = &artifact.safe_fixes[0];
    assert_eq!(safe["confidence"], "medium");
    assert_eq!(safe["confidenceDetail"], "medium_with_evidence");
    assert_eq!(
        safe["finding"]["supportedBy"][0]["kind"],
        "entry-unreachable"
    );
    Ok(())
}

#[test]
fn call_graph_support_requires_bounded_member_stats() -> Result<()> {
    let artifact = build_rank_fixes_artifact(rank_request(
        json!({
            "proposal_C_remove_symbol": [
                { "file": "src/worker.ts", "line": 1, "symbol": "Worker", "kind": "FunctionDeclaration", "action": "" }
            ],
            "proposal_A_demote_to_internal": [],
            "proposal_B_review": [],
            "proposal_remove_export_specifier": [],
            "proposal_DEGRADED_unprocessed": [],
            "excludedCandidates": []
        }),
        Some(json!({
            "findings": [{
                "id": "dead-export:src/worker.ts:Worker:1",
                "safeAction": {
                    "kind": "demote_export_declaration",
                    "proofComplete": true,
                    "actionBlockers": [],
                    "target": { "definitionId": "src/worker.ts#FunctionDeclaration:1-40" }
                },
                "actionBlockers": []
            }]
        })),
        Some(json!({
            "fanInByIdentity": { "src/worker.ts::Worker": 0 }
        })),
        Some(json!({
            "meta": {
                "supports": {
                    "callFanInByDefinitionId": true,
                    "callFanInByIdentity": true
                }
            },
            "callFanInByDefinitionId": {
                "src/worker.ts#FunctionDeclaration:1-40": 0
            },
            "callFanInByIdentity": {
                "src/worker.ts::Worker": 0
            }
        })),
        None,
        None,
        Some(json!({ "src/worker.ts": { "risk": false } })),
    ))?;
    assert_eq!(artifact.summary["SAFE_FIX"], 1);
    assert!(artifact.safe_fixes[0]["finding"]["supportedBy"]
        .as_array()
        .is_none_or(|items| {
            !items
                .iter()
                .any(|item| item["kind"] == "call-graph-no-observed-callers")
        }));
    Ok(())
}

fn empty_dead_classify() -> Value {
    json!({
        "proposal_C_remove_symbol": [],
        "proposal_A_demote_to_internal": [],
        "proposal_B_review": [],
        "proposal_remove_export_specifier": [],
        "proposal_DEGRADED_unprocessed": [],
        "excludedCandidates": []
    })
}

fn module_reachability_for_unreachable(file: &str) -> Value {
    json!({
        "meta": {
            "completenessBySubmodule": { ".": "high" },
            "supports": {
                "runtimeReachableFiles": true,
                "typeReachableFiles": true,
                "boundedOutFiles": true
            }
        },
        "runtimeReachableFiles": [],
        "typeReachableFiles": [],
        "boundedOutFiles": [],
        "unreachableFiles": [file]
    })
}

fn rank_request(
    dead_classify: Value,
    export_action_safety: Option<Value>,
    symbols: Option<Value>,
    call_graph: Option<Value>,
    entry_surface: Option<Value>,
    module_reachability: Option<Value>,
    extra_public_risk: Option<Value>,
) -> RankFixesRequest {
    let mut public_deep_import_risk_by_file = serde_json::Map::new();
    public_deep_import_risk_by_file.insert("__sentinel__".to_string(), json!({ "risk": false }));
    for file in [
        "src/safe.ts",
        "src/review.ts",
        "src/bounded.ts",
        "src/public.ts",
    ] {
        public_deep_import_risk_by_file.insert(file.to_string(), json!({ "risk": false }));
    }
    if let Some(Value::Object(extra)) = extra_public_risk {
        public_deep_import_risk_by_file.extend(extra);
    }
    match serde_json::from_value(json!({
        "schemaVersion": "lumin-rank-fixes-producer-request.v1",
        "root": "C:/repo",
        "generated": "2026-07-03T00:00:00.000Z",
        "artifacts": {
            "deadClassify": dead_classify,
            "exportActionSafety": export_action_safety,
            "symbols": symbols,
            "callGraph": call_graph,
            "entrySurface": entry_surface,
            "moduleReachability": module_reachability
        },
        "publicDeepImportRiskByFile": public_deep_import_risk_by_file
    })) {
        Ok(request) => request,
        Err(error) => panic!("valid request: {error}"),
    }
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
