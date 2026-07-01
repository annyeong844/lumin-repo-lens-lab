use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

use lumin_audit_core::generated_artifacts::{
    summarize_generated_artifacts, GeneratedArtifactsMode, GeneratedArtifactsOptions,
};

#[test]
fn generated_artifacts_summary_groups_generated_misses() -> Result<()> {
    let root = tempfile::tempdir()?;
    let symbols = json!({
        "unresolvedInternalSpecifierRecords": [
            {
                "specifier": "@scope/prisma/enums",
                "consumerFile": "apps/web/src/a.ts",
                "reason": "workspace-generated-artifact-missing",
                "generatedArtifact": {
                    "policyVersion": "generated-artifact-policy-v1",
                    "generatorFamily": "prisma",
                    "confidence": "strong",
                    "matchedPackage": "@scope/prisma",
                    "targetSubpath": "enums"
                }
            },
            {
                "specifier": "@scope/prisma/enums",
                "consumerFile": "apps/api/src/b.ts",
                "reason": "workspace-generated-artifact-missing",
                "generatedArtifact": {
                    "policyVersion": "generated-artifact-policy-v1",
                    "generatorFamily": "prisma",
                    "confidence": "strong",
                    "matchedPackage": "@scope/prisma",
                    "targetSubpath": "enums"
                }
            },
            {
                "specifier": "@scope/types/missing",
                "consumerFile": "apps/web/src/c.ts",
                "reason": "workspace-package-subpath-target-missing"
            }
        ]
    });

    let summary = summarize_generated_artifacts(
        root.path(),
        Some(&symbols),
        &GeneratedArtifactsOptions::default(),
    );

    assert_eq!(
        serde_json::to_value(summary)?,
        json!({
            "mode": "default",
            "generatedArtifactPolicyVersion": "generated-artifact-policy-v1",
            "executedGenerators": false,
            "reasonSummary": {
                "workspace-generated-artifact-missing": 2
            },
            "topGeneratedMisses": [
                {
                    "specifier": "@scope/prisma/enums",
                    "matchedPackage": "@scope/prisma",
                    "targetSubpath": "enums",
                    "count": 2,
                    "generatorFamily": "prisma",
                    "confidence": "strong"
                }
            ],
            "generatedConsumerBlindZoneCount": 0,
            "topGeneratedConsumerBlindZones": [],
            "presentButOutOfScopeCount": 0,
            "presentButOutOfScope": [],
            "supportedGenerators": []
        })
    );
    Ok(())
}

#[test]
fn generated_artifacts_summary_groups_blind_zones() -> Result<()> {
    let root = tempfile::tempdir()?;
    let symbols = json!({
        "generatedConsumerBlindZones": [
            {
                "reason": "generated-consumer-blind-zone",
                "specifier": "@scope/prisma/enums",
                "consumerFile": "apps/web/src/a.ts",
                "candidatePath": "packages/prisma/generated/enums.ts",
                "status": "missing",
                "scopePackageRoot": "packages/prisma",
                "mode": "default"
            },
            {
                "reason": "generated-consumer-blind-zone",
                "specifier": "@scope/prisma/enums",
                "consumerFile": "apps/api/src/b.ts",
                "candidatePath": "packages/prisma/generated/enums.ts",
                "status": "present-but-out-of-scope",
                "scanScopeReason": "excluded",
                "scopePackageRoot": "packages/prisma",
                "mode": "prepared"
            },
            {
                "reason": "generated-consumer-blind-zone",
                "specifier": "@scope/kysely/types",
                "consumerFile": "apps/api/src/c.ts",
                "candidatePath": "packages/kysely/generated/types.ts",
                "status": "missing",
                "scopePackageRoot": "packages/kysely",
                "mode": "default"
            }
        ]
    });

    let summary = summarize_generated_artifacts(
        root.path(),
        Some(&symbols),
        &GeneratedArtifactsOptions::default(),
    );

    assert_eq!(
        serde_json::to_value(summary)?
            .get("topGeneratedConsumerBlindZones")
            .cloned(),
        Some(json!([
            {
                "scopePackageRoot": "packages/prisma",
                "count": 2,
                "statuses": {
                    "missing": 1,
                    "present-but-out-of-scope": 1
                },
                "topSpecifiers": [{ "specifier": "@scope/prisma/enums", "count": 2 }],
                "examples": [
                    {
                        "specifier": "@scope/prisma/enums",
                        "consumerFile": "apps/api/src/b.ts",
                        "candidatePath": "packages/prisma/generated/enums.ts",
                        "status": "present-but-out-of-scope",
                        "scanScopeReason": "excluded",
                        "mode": "prepared"
                    },
                    {
                        "specifier": "@scope/prisma/enums",
                        "consumerFile": "apps/web/src/a.ts",
                        "candidatePath": "packages/prisma/generated/enums.ts",
                        "status": "missing",
                        "mode": "default"
                    }
                ]
            },
            {
                "scopePackageRoot": "packages/kysely",
                "count": 1,
                "statuses": { "missing": 1 },
                "topSpecifiers": [{ "specifier": "@scope/kysely/types", "count": 1 }],
                "examples": [
                    {
                        "specifier": "@scope/kysely/types",
                        "consumerFile": "apps/api/src/c.ts",
                        "candidatePath": "packages/kysely/generated/types.ts",
                        "status": "missing",
                        "mode": "default"
                    }
                ]
            }
        ]))
    );
    Ok(())
}

#[test]
fn generated_artifacts_present_mode_reports_existing_targets_excluded_by_scan_policy() -> Result<()>
{
    let root = tempfile::tempdir()?;
    let target = root.path().join("packages/prisma/generated/enums.ts");
    fs::create_dir_all(target.parent().expect("fixture target has a parent"))?;
    fs::write(&target, "export enum Kind { A = 'A' }\n")?;
    let symbols = json!({
        "unresolvedInternalSpecifierRecords": [
            {
                "specifier": "@scope/prisma/generated/enums",
                "consumerFile": "apps/web/src/a.ts",
                "reason": "workspace-generated-artifact-missing",
                "targetCandidates": ["packages/prisma/generated/enums.ts"],
                "generatedArtifact": {
                    "policyVersion": "generated-artifact-policy-v1",
                    "generatorFamily": "prisma",
                    "confidence": "strong",
                    "matchedPackage": "@scope/prisma",
                    "targetSubpath": "generated/enums"
                }
            }
        ]
    });

    let summary = summarize_generated_artifacts(
        root.path(),
        Some(&symbols),
        &GeneratedArtifactsOptions {
            mode: GeneratedArtifactsMode::Present,
            excludes: names(&["packages/prisma/generated"]),
            ..GeneratedArtifactsOptions::default()
        },
    );

    assert_eq!(
        serde_json::to_value(summary)?
            .get("presentButOutOfScope")
            .cloned(),
        Some(json!([
            {
                "specifier": "@scope/prisma/generated/enums",
                "consumerFile": "apps/web/src/a.ts",
                "matchedPackage": "@scope/prisma",
                "targetSubpath": "generated/enums",
                "candidatePath": "packages/prisma/generated/enums.ts",
                "reason": "present-but-out-of-scope",
                "mode": "present"
            }
        ]))
    );
    Ok(())
}

#[test]
fn generated_artifacts_prepared_mode_marks_existing_excluded_targets_stale_unknown() -> Result<()> {
    let root = tempfile::tempdir()?;
    let target = root.path().join("packages/prisma/generated/enums.ts");
    fs::create_dir_all(target.parent().expect("fixture target has a parent"))?;
    fs::write(&target, "export enum Kind { A = 'A' }\n")?;
    let symbols = json!({
        "unresolvedInternalSpecifierRecords": [
            {
                "specifier": "@scope/prisma/generated/enums",
                "consumerFile": "apps/web/src/a.ts",
                "reason": "workspace-generated-artifact-missing",
                "targetCandidates": [target],
                "generatedArtifact": {
                    "policyVersion": "generated-artifact-policy-v1",
                    "generatorFamily": "prisma",
                    "confidence": "strong",
                    "matchedPackage": "@scope/prisma",
                    "targetSubpath": "generated/enums"
                }
            }
        ]
    });

    let summary = summarize_generated_artifacts(
        root.path(),
        Some(&symbols),
        &GeneratedArtifactsOptions {
            mode: GeneratedArtifactsMode::Prepared,
            excludes: names(&["packages/prisma/generated"]),
            ..GeneratedArtifactsOptions::default()
        },
    );
    let summary_json = serde_json::to_value(summary)?;

    assert_eq!(summary_json["mode"], "prepared");
    assert_eq!(
        summary_json["presentButOutOfScope"][0]["staleStatus"],
        "unknown"
    );
    assert_eq!(
        summary_json["presentButOutOfScope"][0]["staleReason"],
        "generator-input-hash-not-recorded"
    );
    Ok(())
}

#[test]
fn cli_generated_artifacts_summary_emits_present_scope_json() -> Result<()> {
    let root = tempfile::tempdir()?;
    let target = root.path().join("packages/prisma/generated/enums.ts");
    fs::create_dir_all(target.parent().expect("fixture target has a parent"))?;
    fs::write(&target, "export enum Kind { A = 'A' }\n")?;
    let symbols = root.path().join("symbols.json");
    fs::write(
        &symbols,
        serde_json::to_vec(&json!({
            "unresolvedInternalSpecifierRecords": [
                {
                    "specifier": "@scope/prisma/generated/enums",
                    "consumerFile": "apps/web/src/a.ts",
                    "reason": "workspace-generated-artifact-missing",
                    "targetCandidates": ["packages/prisma/generated/enums.ts"],
                    "generatedArtifact": {
                        "matchedPackage": "@scope/prisma",
                        "targetSubpath": "generated/enums"
                    }
                }
            ]
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("generated-artifacts-summary")
        .arg("--root")
        .arg(root.path())
        .arg("--symbols")
        .arg(&symbols)
        .arg("--generated-artifacts")
        .arg("present")
        .arg("--exclude")
        .arg("packages/prisma/generated")
        .output()?;

    assert!(output.status.success());
    let summary = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(summary["mode"], "present");
    assert_eq!(summary["presentButOutOfScopeCount"], 1);
    assert_eq!(
        summary["presentButOutOfScope"][0]["candidatePath"],
        "packages/prisma/generated/enums.ts"
    );
    Ok(())
}

fn names(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
