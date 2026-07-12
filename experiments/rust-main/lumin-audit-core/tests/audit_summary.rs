use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_audit_summary_render_writes_markdown_preview_and_small_result() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let output_path = temp.path().join("audit-summary.latest.md");
    let result = temp.path().join("result.json");
    fs::write(&output_path, "stale summary\n")?;
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-audit-summary-render-request.v1",
            "outputPath": output_path,
            "manifest": {
                "meta": { "generated": "2026-07-05T00:00:00.000Z" },
                "profile": "full",
                "scanRange": {
                    "files": 7,
                    "languages": ["ts", "rs"],
                    "includeTests": false,
                    "excludes": ["target/**"]
                },
                "confidence": {
                    "parseErrors": 0,
                    "unresolvedInternalRatio": 0.125
                },
                "rustAnalysis": {
                    "status": "complete",
                    "available": true,
                    "files": 2,
                    "syntaxReviewSignals": 3,
                    "syntaxReviewOpaqueSurfaces": 1,
                    "syntaxFunctionCloneExactBodyGroups": 4,
                    "syntaxFunctionCloneStructureGroups": 5,
                    "syntaxFunctionCloneSignatureGroups": 6,
                    "syntaxFunctionCloneNearCandidates": 7,
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
                    }]
                },
                "frameworkResourceSurfaces": {
                    "totalFilesWithSurfaces": 4,
                    "byLane": { "framework-dispatch-entry": 2, "codemod-template": 1 },
                    "byConfidence": { "grounded": 3, "path-shaped-review": 1 },
                    "topExamples": [{
                        "file": "src/routes/+page.svelte",
                        "reasons": ["svelte-route"]
                    }]
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
                "blindZones": [{
                    "area": "resolver",
                    "details": {
                        "topUnresolvedReasons": [{
                            "reason": "alias-missing",
                            "count": 2
                        }]
                    }
                }],
                "livingAudit": {
                    "existingDocs": [{ "path": "docs/audit.md" }]
                },
                "artifactsProduced": [
                    "symbols.json",
                    "topology.json",
                    "module-reachability.json",
                    "function-clones.json",
                    "unused-deps.json",
                    "topology.mermaid.md"
                ]
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
            "functionClones": {
                "meta": {
                    "exactBodyGroupCount": 30,
                    "structureGroupCount": 40,
                    "signatureGroupCount": 50,
                    "nearFunctionCandidateCount": 60
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
        .arg("audit-summary-render")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&result)
        .output()?;

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let markdown = fs::read_to_string(&output_path)?;
    assert!(markdown.starts_with("# Audit Artifact Brief"));
    assert!(markdown.contains("Generated: 2026-07-05T00:00:00.000Z"));
    assert!(markdown
        .contains("Scan range: 7 files, ts, rs, production files only; excludes: target/**"));
    assert!(
        markdown.contains("Confidence: parse errors 0, unresolved internal 12.5%, blind zones 1")
    );
    assert!(markdown.contains("## Measured Cues (Unranked)"));
    assert!(markdown.contains("Exported any-contamination"));
    assert!(markdown.contains("Rust analyzer: 2 files"));
    assert!(markdown.contains("Resolver blocked absence hints: 2"));
    assert!(markdown.contains("Framework/resource surfaces: 4 files"));
    assert!(markdown
        .contains("Dependency hygiene: 1 review-only dependency declaration needs inspection"));
    assert!(markdown.contains("Unreachable SCCs: 1 group, 2 files"));
    assert!(markdown.contains("## Living Audit Tracking"));
    assert!(markdown.contains("## Expansion Hint"));
    assert!(markdown.contains("## Guardrails"));
    assert!(!markdown.contains("stale summary"));

    let result: Value = serde_json::from_slice(&fs::read(&result)?)?;
    assert_eq!(
        result["schemaVersion"],
        "lumin-audit-summary-render-result.v1"
    );
    assert_eq!(result["path"], output_path.to_string_lossy().to_string());
    assert_eq!(result["bytes"], markdown.len());
    let preview = result["preview"]
        .as_str()
        .context("preview should be a string")?;
    assert!(preview.starts_with("[audit-repo] artifact brief preview:"));
    assert!(preview.contains("[audit-repo]   Read First:"));
    assert!(preview.contains("[audit-repo]   Measured Cues:"));
    assert!(preview.contains("[audit-repo]   Living Audit Tracking:"));
    assert!(preview.contains("[audit-repo]   Guardrails:"));
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
