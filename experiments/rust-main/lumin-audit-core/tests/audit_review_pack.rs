use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_audit_review_pack_render_writes_markdown_and_small_result() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let output_path = temp.path().join("audit-review-pack.latest.md");
    let result = temp.path().join("result.json");
    fs::write(&output_path, "stale review pack\n")?;
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-audit-review-pack-render-request.v1",
            "outputPath": output_path,
            "manifest": {
                "profile": "full",
                "scanRange": {
                    "files": 7,
                    "languages": ["ts", "rs"],
                    "includeTests": false
                },
                "rustAnalysis": {
                    "status": "complete",
                    "available": true,
                    "files": 2,
                    "scanScope": {
                        "includeTests": false,
                        "exclude": ["target/**"]
                    }
                },
                "resolverDiagnostics": {
                    "blockedCandidateHintCount": 2,
                    "blockedCandidateHintSampleLimit": 10,
                    "blockedCandidateHints": [{
                        "candidatePath": "packages/core/src/dead.ts",
                        "specifier": "@repo/core/dead",
                        "reason": "workspace-package-subpath-target-missing"
                    }],
                    "blockedCandidateHintReasonCounts": [{
                        "reason": "workspace-package-subpath-target-missing",
                        "count": 2,
                        "families": { "workspace-packages": 2 }
                    }],
                    "blockedCandidateHintFamilyCounts": [{
                        "family": "workspace-packages",
                        "count": 2,
                        "reasons": { "workspace-package-subpath-target-missing": 2 }
                    }]
                },
                "frameworkResourceSurfaces": {
                    "totalFilesWithSurfaces": 4,
                    "byLane": { "framework-dispatch-entry": 2, "codemod-template": 1 }
                },
                "unusedDependencies": {
                    "status": "complete",
                    "reviewUnusedCount": 1,
                    "mutedCount": 3,
                    "confidenceLimitedCount": 0
                },
                "sfcEvidence": {
                    "totalEvidenceCount": 2,
                    "reviewOnlyEvidenceCount": 1,
                    "byLane": {
                        "scriptImportConsumers": 1,
                        "templateComponentRefs": 1
                    }
                },
                "blindZones": [{ "area": "resolver" }]
            },
            "checklistFacts": {
                "B1B2_shape_drift": {
                    "exactDuplicateGroups": 1,
                    "nearShapeCandidateCount": 2
                },
                "B1_duplicate_implementation": {
                    "exactBodyGroups": 3,
                    "structureGroupCandidates": 4,
                    "signatureGroupCandidates": 5,
                    "nearFunctionCandidates": 6
                },
                "E2_silent_catch": {
                    "count": 1,
                    "nonEmptyAnonymousCount": 2,
                    "unusedParamCount": 3
                }
            },
            "fixPlan": {
                "summary": {
                    "SAFE_FIX": 1,
                    "REVIEW_FIX": 2,
                    "DEGRADED": 3,
                    "MUTED": 4
                }
            },
            "topology": {
                "summary": { "sccCount": 1 },
                "sccs": []
            },
            "discipline": {
                "totals": {
                    ":any": 1,
                    "as any": 2
                }
            },
            "callGraph": {
                "summary": { "semiDead": 2 }
            },
            "barrels": {
                "root": {}
            },
            "shapeIndex": {
                "facts": [{ "hash": "sha256:0" }]
            },
            "functionClones": {
                "meta": {
                    "exactBodyGroupCount": 30,
                    "structureGroupCount": 40,
                    "signatureGroupCount": 50,
                    "nearFunctionCandidateCount": 60
                }
            },
            "deadClassify": {
                "summary": {
                    "excluded": {
                        "public-surface": 2
                    }
                }
            },
            "symbols": {
                "meta": {
                    "supports": {
                        "anyContamination": true
                    }
                },
                "typeOwnersByIdentity": {
                    "src/types.ts::Api": {
                        "anyContamination": {
                            "label": "severely-any-contaminated",
                            "labels": ["severely-any-contaminated", "any-contaminated"]
                        }
                    }
                },
                "helperOwnersByIdentity": {}
            },
            "moduleReachability": {
                "summary": {
                    "unreachableStronglyConnectedComponents": 1,
                    "unreachableStronglyConnectedFiles": 2
                }
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("audit-review-pack-render")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&result)
        .output()?;

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let markdown = fs::read_to_string(&output_path)?;
    assert!(markdown.starts_with("# Audit Review Pack"));
    assert!(markdown.contains("Scan range: 7 files; ts, rs; production only."));
    assert!(markdown.contains("Lane 1"));
    assert!(markdown.contains("Runtime SCC count from topology: 1"));
    assert!(markdown.contains("Identity-level anyContamination: 1 severe type owner"));
    assert!(markdown.contains("Rust analyzer artifact available for 2 file(s)"));
    assert!(markdown.contains("Resolver blocked absence hints: 2"));
    assert!(markdown.contains("Framework/resource surfaces: 4 files"));
    assert!(markdown.contains("Dependency hygiene review: inspect unused-deps.json"));
    assert!(markdown.contains("Unreachable SCCs: 1 group, 2 files"));
    assert!(markdown.contains("Merge Instructions"));
    assert!(!markdown.contains("stale review pack"));
    let result: Value = serde_json::from_slice(&fs::read(&result)?)?;
    assert_eq!(
        result["schemaVersion"],
        "lumin-audit-review-pack-render-result.v1"
    );
    assert_eq!(result["path"], output_path.to_string_lossy().to_string());
    assert_eq!(result["bytes"], markdown.len());
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
