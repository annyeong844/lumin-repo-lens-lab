use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

use lumin_audit_core::artifact_summaries::{summarize_artifact, ArtifactSummaryKind};

#[test]
fn framework_resource_surfaces_summary_uses_summary_and_fallback_examples() -> Result<()> {
    let artifact = json!({
        "schemaVersion": "framework-resource-surfaces.v1",
        "policyVersion": "framework-resource-surface-policy-v1",
        "files": [
            {
                "file": "src/Button.stories.tsx",
                "surfaceLanes": [
                    {
                        "lane": "framework-dispatch-entry",
                        "capabilityPack": "framework.storybook",
                        "reason": "storybook-story-file"
                    }
                ]
            },
            {
                "file": "templates/controller.ts.hbs",
                "surfaceLanes": [
                    {
                        "lane": "scaffold-template-resource",
                        "capabilityPack": "surface.scaffold-template",
                        "reason": "handlebars-template-resource"
                    }
                ]
            }
        ],
        "summary": {
            "totalFilesWithSurfaces": 2,
            "totalSurfaceLanes": 2,
            "byLane": {
                "framework-dispatch-entry": 1,
                "scaffold-template-resource": 1
            },
            "byCapabilityPack": {
                "framework.storybook": 1,
                "surface.scaffold-template": 1
            },
            "byConfidence": {
                "grounded": 1,
                "resource-only": 1
            }
        }
    });

    let summary = summarize_artifact(ArtifactSummaryKind::FrameworkResourceSurfaces, &artifact)
        .ok_or_else(|| anyhow::anyhow!("object artifact should summarize"))?;

    assert_eq!(
        serde_json::to_value(summary)?,
        json!({
            "artifact": "framework-resource-surfaces.json",
            "schemaVersion": "framework-resource-surfaces.v1",
            "policyVersion": "framework-resource-surface-policy-v1",
            "totalFilesWithSurfaces": 2,
            "totalSurfaceLanes": 2,
            "byLane": {
                "framework-dispatch-entry": 1,
                "scaffold-template-resource": 1
            },
            "byCapabilityPack": {
                "framework.storybook": 1,
                "surface.scaffold-template": 1
            },
            "byConfidence": {
                "grounded": 1,
                "resource-only": 1
            },
            "byReason": {},
            "byFramework": {},
            "topExamples": [
                {
                    "file": "src/Button.stories.tsx",
                    "lanes": ["framework-dispatch-entry"],
                    "capabilityPacks": ["framework.storybook"],
                    "reasons": ["storybook-story-file"]
                },
                {
                    "file": "templates/controller.ts.hbs",
                    "lanes": ["scaffold-template-resource"],
                    "capabilityPacks": ["surface.scaffold-template"],
                    "reasons": ["handlebars-template-resource"]
                }
            ]
        })
    );
    Ok(())
}

#[test]
fn framework_resource_surfaces_summary_preserves_malformed_artifact_as_unavailable() -> Result<()> {
    let artifact = json!({
        "schemaVersion": null,
        "artifact": "framework-resource-surfaces.json",
        "status": "unavailable",
        "reason": {
            "kind": "malformed-json",
            "message": "expected value"
        },
        "summary": {
            "status": "unavailable",
            "unavailableReason": "malformed-json"
        }
    });

    let summary = summarize_artifact(ArtifactSummaryKind::FrameworkResourceSurfaces, &artifact)
        .ok_or_else(|| anyhow::anyhow!("unavailable framework artifact should summarize"))?;
    let summary = serde_json::to_value(summary)?;

    assert_eq!(summary["status"], "unavailable");
    assert_eq!(summary["reason"]["kind"], "malformed-json");
    assert_eq!(summary["totalFilesWithSurfaces"], json!(null));
    assert_eq!(summary["totalSurfaceLanes"], json!(null));
    assert_eq!(summary["topExamples"], json!([]));
    Ok(())
}

#[test]
fn unused_dependencies_summary_sorts_review_unused_and_preserves_unavailable_reason() -> Result<()>
{
    let artifact = json!({
        "schemaVersion": "unused-deps.v1",
        "policyVersion": "unused-deps-review-policy-v1",
        "status": "unavailable",
        "reason": "input-artifact-missing",
        "summary": {
            "packageCount": 2,
            "declaredDependencyCount": 5,
            "usedCount": 1,
            "reviewUnusedCount": 2,
            "mutedCount": 2,
            "confidenceLimitedCount": 0,
            "unavailableCount": 0,
            "byReason": {
                "external-import-consumer": 1,
                "no-observed-consumer": 2
            }
        },
        "packages": [
            {
                "packageDir": "packages/app",
                "manifestPath": "packages/app/package.json",
                "dependencies": [
                    {
                        "name": "left-pad",
                        "field": "dependencies",
                        "status": "review-unused",
                        "reason": "no-observed-consumer",
                        "confidence": "review"
                    }
                ]
            },
            {
                "packageDir": ".",
                "manifestPath": "package.json",
                "dependencies": [
                    {
                        "name": "unused-lib",
                        "field": "devDependencies",
                        "status": "review-unused",
                        "reason": "no-observed-consumer",
                        "confidence": "review"
                    },
                    {
                        "name": "tsx",
                        "field": "devDependencies",
                        "status": "muted",
                        "reason": "package-script-tool",
                        "confidence": "grounded"
                    }
                ]
            }
        ]
    });

    let summary = summarize_artifact(ArtifactSummaryKind::UnusedDeps, &artifact)
        .ok_or_else(|| anyhow::anyhow!("object artifact should summarize"))?;

    assert_eq!(
        serde_json::to_value(summary)?,
        json!({
            "artifact": "unused-deps.json",
            "schemaVersion": "unused-deps.v1",
            "policyVersion": "unused-deps-review-policy-v1",
            "status": "unavailable",
            "reason": "input-artifact-missing",
            "packageCount": 2,
            "declaredDependencyCount": 5,
            "usedCount": 1,
            "reviewUnusedCount": 2,
            "mutedCount": 2,
            "confidenceLimitedCount": 0,
            "unavailableCount": 0,
            "byReason": {
                "external-import-consumer": 1,
                "no-observed-consumer": 2
            },
            "topReviewUnused": [
                {
                    "packageDir": ".",
                    "manifestPath": "package.json",
                    "name": "unused-lib",
                    "field": "devDependencies",
                    "reason": "no-observed-consumer",
                    "confidence": "review"
                },
                {
                    "packageDir": "packages/app",
                    "manifestPath": "packages/app/package.json",
                    "name": "left-pad",
                    "field": "dependencies",
                    "reason": "no-observed-consumer",
                    "confidence": "review"
                }
            ]
        })
    );
    Ok(())
}

#[test]
fn unused_dependencies_summary_tolerates_malformed_package_dependency_lists() -> Result<()> {
    let artifact = json!({
        "schemaVersion": "unused-deps.v1",
        "policyVersion": "unused-deps-review-policy-v1",
        "status": "complete",
        "summary": {
            "packageCount": 1,
            "declaredDependencyCount": 1,
            "usedCount": 0,
            "reviewUnusedCount": 1,
            "mutedCount": 0,
            "confidenceLimitedCount": 0,
            "unavailableCount": 0,
            "byReason": { "no-observed-consumer": 1 }
        },
        "packages": [
            {
                "packageDir": ".",
                "manifestPath": "package.json",
                "dependencies": {}
            }
        ]
    });

    let summary = summarize_artifact(ArtifactSummaryKind::UnusedDeps, &artifact)
        .ok_or_else(|| anyhow::anyhow!("object artifact should summarize"))?;
    let summary = serde_json::to_value(summary)?;

    assert_eq!(summary["status"], "complete");
    assert_eq!(summary["reviewUnusedCount"], 1);
    assert_eq!(summary["topReviewUnused"], json!([]));
    Ok(())
}

#[test]
fn block_clones_summary_excludes_raw_groups_and_preserves_cap_fields() -> Result<()> {
    let artifact = json!({
        "schemaVersion": "block-clones.v1",
        "policyVersion": "block-clone-review-policy-v1",
        "status": "complete",
        "normalization": {
            "policyId": "block-clone-normalization-v1",
            "mode": "alpha-identifier"
        },
        "thresholds": {
            "policyId": "block-clone-threshold-policy-v2",
            "minTokens": 50,
            "minLines": 5,
            "minOccurrences": 2,
            "maxInstancesPerGroup": 20,
            "maxCandidateGroups": 1000,
            "maxReviewGroups": 100,
            "maxMutedGroups": 100,
            "maxGroups": 40,
            "maxTokensPerFile": 200000
        },
        "summary": {
            "fileCount": 12,
            "tokenCount": 3400,
            "groupCount": 2,
            "instanceCount": 5,
            "reviewGroupCount": 1,
            "mutedGroupCount": 1,
            "skippedFileCount": 1,
            "unavailableFileCount": 0
        },
        "noisePolicy": {
            "policyId": "block-clone-noise-policy-v1",
            "reviewGroupCount": 1,
            "mutedGroupCount": 1,
            "mutedByReason": {
                "node-vitest-mirror-pair": 1
            },
            "candidateCapSaturated": false,
            "reviewCapSaturated": false,
            "mutedCapSaturated": false
        },
        "groups": [
            {
                "claim": "repeated normalized token region",
                "instances": [
                    { "file": "src/a.ts", "startLine": 1, "endLine": 8 },
                    { "file": "src/b.ts", "startLine": 2, "endLine": 9 }
                ]
            }
        ]
    });

    let summary = summarize_artifact(ArtifactSummaryKind::BlockClones, &artifact)
        .ok_or_else(|| anyhow::anyhow!("object artifact should summarize"))?;
    let summary = serde_json::to_value(summary)?;

    assert_eq!(
        summary,
        json!({
            "artifact": "block-clones.json",
            "schemaVersion": "block-clones.v1",
            "policyVersion": "block-clone-review-policy-v1",
            "status": "complete",
            "reviewOnly": true,
            "normalizationPolicyId": "block-clone-normalization-v1",
            "normalizationMode": "alpha-identifier",
            "thresholdPolicyId": "block-clone-threshold-policy-v2",
            "noisePolicyId": "block-clone-noise-policy-v1",
            "thresholds": {
                "minTokens": 50,
                "minLines": 5,
                "minOccurrences": 2,
                "maxInstancesPerGroup": 20,
                "maxTokensPerFile": 200000,
                "maxCandidateGroups": 1000,
                "maxReviewGroups": 100,
                "maxMutedGroups": 100,
                "maxGroups": 40
            },
            "fileCount": 12,
            "tokenCount": 3400,
            "groupCount": 2,
            "instanceCount": 5,
            "reviewGroupCount": 1,
            "mutedGroupCount": 1,
            "mutedByReason": {
                "node-vitest-mirror-pair": 1
            },
            "candidateCapSaturated": false,
            "reviewCapSaturated": false,
            "mutedCapSaturated": false,
            "skippedFileCount": 1,
            "unavailableFileCount": 0
        })
    );
    assert!(summary.get("groups").is_none());
    assert!(summary.get("instances").is_none());
    Ok(())
}

#[test]
fn cli_artifact_summary_emits_framework_resource_surface_json() -> Result<()> {
    let root = tempfile::tempdir()?;
    let artifact = root.path().join("framework-resource-surfaces.json");
    fs::write(
        &artifact,
        serde_json::to_vec(&json!({
            "schemaVersion": "framework-resource-surfaces.v1",
            "policyVersion": "framework-resource-surface-policy-v1",
            "files": [],
            "summary": { "totalFilesWithSurfaces": 0, "totalSurfaceLanes": 0 }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("artifact-summary")
        .arg("--artifact-kind")
        .arg("framework-resource-surfaces")
        .arg("--artifact")
        .arg(&artifact)
        .output()?;

    assert!(output.status.success());
    let summary = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(summary["artifact"], "framework-resource-surfaces.json");
    assert_eq!(summary["totalFilesWithSurfaces"], 0);
    Ok(())
}

#[test]
fn non_object_artifact_summary_returns_null() -> Result<()> {
    let summary = summarize_artifact(ArtifactSummaryKind::UnusedDeps, &json!(null));

    assert!(summary.is_none());
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
