use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

use lumin_audit_core::generated_artifacts::GeneratedArtifactsMode;
use lumin_audit_core::manifest_evidence::{
    summarize_manifest_evidence, ManifestEvidenceArtifacts, ManifestEvidenceOptions,
};

#[test]
fn manifest_evidence_composes_rust_owned_fields_without_blind_zones() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("LUMIN_AUDIT.md"), "audit")?;
    let root_text = root.path().to_string_lossy().to_string();
    let triage = json!({
        "shape": {
            "totalFiles": 8,
            "tsFiles": 5
        }
    });
    let symbols = json!({
        "uses": {
            "external": 2,
            "resolvedInternal": 3,
            "unresolvedInternal": 1,
            "unresolvedInternalRatio": 0.25,
            "sfcTemplateComponentRefs": 2
        },
        "generatedArtifactBlindZones": [
            {
                "path": "src/generated/client.ts",
                "reason": "generated-client"
            }
        ],
        "unresolvedInternalSpecifierRecords": [
            {
                "specifier": "@/missing",
                "consumerFile": "src/app.ts",
                "reason": "alias-prefix-unresolved"
            }
        ]
    });
    let resolver_diagnostics = json!({
        "resolverVersion": "resolver-v1",
        "summary": {
            "blindZoneCount": 1,
            "blockedCandidateHintCount": 0,
            "candidateTargetCount": 3
        },
        "blockedCandidateHints": []
    });
    let framework_resource_surfaces = json!({
        "schemaVersion": "framework-resource-surfaces.v1",
        "policyVersion": "framework-resource-surface-policy-v1",
        "files": [
            {
                "file": "src/App.stories.tsx",
                "surfaceLanes": [
                    {
                        "lane": "framework-dispatch-entry",
                        "capabilityPack": "framework.storybook",
                        "reason": "storybook-story-file"
                    }
                ]
            }
        ]
    });
    let unused_deps = json!({
        "schemaVersion": "unused-deps.v1",
        "policyVersion": "unused-deps-review-policy-v1",
        "status": "complete",
        "summary": {
            "packageCount": 1,
            "declaredDependencyCount": 1,
            "usedCount": 1,
            "reviewUnusedCount": 0,
            "mutedCount": 0,
            "confidenceLimitedCount": 0,
            "unavailableCount": 0,
            "byReason": {}
        },
        "packages": []
    });
    let block_clones = json!({
        "schemaVersion": "block-clones.v1",
        "policyVersion": "block-clone-review-policy-v1",
        "status": "complete",
        "normalization": {
            "policyId": "block-clone-normalization-v1",
            "mode": "alpha-identifier"
        },
        "thresholds": {
            "policyId": "block-clone-threshold-policy-v1",
            "minTokens": 50,
            "minLines": 5,
            "minOccurrences": 2,
            "maxInstancesPerGroup": 20,
            "maxTokensPerFile": 200000
        },
        "summary": {
            "fileCount": 1,
            "tokenCount": 100,
            "groupCount": 0,
            "instanceCount": 0
        },
        "groups": []
    });

    let summary = serde_json::to_value(summarize_manifest_evidence(
        ManifestEvidenceOptions {
            root: root_text,
            include_tests: false,
            production: true,
            excludes: vec!["dist".to_string()],
            auto_excludes: vec![".audit".to_string()],
            generated_artifacts_mode: GeneratedArtifactsMode::Present,
        },
        ManifestEvidenceArtifacts {
            triage: Some(&triage),
            symbols: Some(&symbols),
            resolver_capabilities: None,
            resolver_diagnostics: Some(&resolver_diagnostics),
            framework_resource_surfaces: Some(&framework_resource_surfaces),
            unused_deps: Some(&unused_deps),
            block_clones: Some(&block_clones),
            rust_analysis: None,
        },
    ))?;

    assert!(summary.get("blindZones").is_none());
    assert_eq!(summary["scanRange"]["includeTests"], false);
    assert_eq!(summary["confidence"]["unresolvedInternal"], 1);
    assert_eq!(
        summary["resolverDiagnostics"]["resolverVersion"],
        "resolver-v1"
    );
    assert_eq!(summary["generatedArtifacts"]["mode"], "present");
    assert_eq!(
        summary["frameworkResourceSurfaces"]["artifact"],
        "framework-resource-surfaces.json"
    );
    assert_eq!(
        summary["unusedDependencies"]["artifact"],
        "unused-deps.json"
    );
    assert_eq!(summary["blockClones"]["artifact"], "block-clones.json");
    assert_eq!(summary["sfcEvidence"]["totalEvidenceCount"], 2);
    assert_eq!(
        summary["livingAudit"]["existingDocs"][0]["path"],
        "LUMIN_AUDIT.md"
    );
    Ok(())
}

#[test]
fn cli_manifest_evidence_summary_reads_output_artifacts() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output_dir = root.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(
        output_dir.join("triage.json"),
        serde_json::to_vec(&json!({
            "summary": { "files": 3 },
            "shape": { "jsFiles": 1 }
        }))?,
    )?;
    fs::write(
        output_dir.join("symbols.json"),
        serde_json::to_vec(&json!({
            "uses": {
                "external": 1,
                "resolvedInternal": 1,
                "unresolvedInternal": 0,
                "unresolvedInternalRatio": 0
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-summary")
        .arg("--root")
        .arg(root.path())
        .arg("--output")
        .arg(&output_dir)
        .arg("--include-tests")
        .arg("--no-production")
        .arg("--generated-artifacts")
        .arg("default")
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["scanRange"]["files"], 3);
    assert_eq!(stdout["confidence"]["externalImports"], 1);
    assert_eq!(
        stdout["resolverDiagnostics"]["resolverVersion"],
        json!(null)
    );
    assert_eq!(stdout["rustAnalysis"], json!(null));
    assert_eq!(stdout["frameworkResourceSurfaces"], json!(null));
    assert_eq!(
        stdout["livingAudit"]["action"],
        "create-only-on-explicit-tracking-request"
    );
    assert!(stdout.get("blindZones").is_none());
    Ok(())
}

#[test]
fn cli_manifest_evidence_summary_hard_stops_on_malformed_existing_artifact() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output_dir = root.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(output_dir.join("symbols.json"), "{not-json")?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-evidence-summary")
        .arg("--root")
        .arg(root.path())
        .arg("--output")
        .arg(&output_dir)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("manifest-evidence-summary: invalid JSON"));
    assert!(stderr.contains("symbols.json"));
    Ok(())
}
