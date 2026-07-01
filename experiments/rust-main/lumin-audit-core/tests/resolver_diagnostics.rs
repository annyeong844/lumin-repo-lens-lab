use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

use lumin_audit_core::resolver_diagnostics::summarize_resolver_diagnostics;

#[test]
fn resolver_diagnostics_summary_prefers_diagnostic_artifact_fields() -> Result<()> {
    let symbols = json!({
        "uses": {
            "unresolvedInternal": 7,
            "unresolvedInternalRatio": 0.25
        },
        "topUnresolvedSpecifiers": [
            { "specifierPrefix": "@/", "count": 5 },
            { "specifierPrefix": "~/", "count": 4 }
        ]
    });
    let resolver_capabilities = json!({
        "resolverVersion": "resolver-capabilities-v1"
    });
    let resolver_diagnostics = json!({
        "resolverVersion": "resolver-diagnostics-v2",
        "summary": {
            "blindZoneCount": 3,
            "blockedCandidateHintCount": 4,
            "candidateTargetCount": 9,
            "topFamilies": [{ "family": "export", "count": 2 }],
            "topAffectedPackageScopes": [{ "packageScope": "packages/app", "count": 2 }],
            "topUnresolvedReasons": [{ "reason": "alias-prefix-unresolved", "count": 4 }],
            "topSpecifierRoots": [{ "specifierRoot": "@/", "count": 4, "reasons": { "alias-prefix-unresolved": 4 }, "examples": [] }]
        },
        "blockedCandidateHints": [
            {
                "reason": "resolver-blind-zone",
                "family": "export",
                "candidatePath": "src/a.ts",
                "specifier": "@/a"
            },
            {
                "reason": "resolver-blind-zone",
                "family": "import",
                "candidatePath": "src/b.ts",
                "specifier": "@/b"
            },
            {
                "reason": "package-scope-unavailable",
                "family": "export",
                "candidatePath": "packages/app/src/c.ts",
                "specifier": "pkg/c"
            }
        ]
    });

    let summary = serde_json::to_value(summarize_resolver_diagnostics(
        Some(&symbols),
        Some(&resolver_capabilities),
        Some(&resolver_diagnostics),
    ))?;

    assert_eq!(summary["resolverVersion"], "resolver-diagnostics-v2");
    assert_eq!(
        summary["resolverCapabilityArtifact"],
        "resolver-capabilities.json"
    );
    assert_eq!(
        summary["resolverDiagnosticsArtifact"],
        "resolver-diagnostics.json"
    );
    assert_eq!(summary["unresolvedInternal"], 7);
    assert_eq!(summary["unresolvedInternalRatio"], 0.25);
    assert_eq!(summary["blindZoneCount"], 3);
    assert_eq!(summary["blockedCandidateHintCount"], 4);
    assert_eq!(summary["blockedCandidateHintSampleLimit"], 10);
    assert_eq!(
        summary["blockedCandidateHintReasonCounts"],
        json!([
            {
                "reason": "resolver-blind-zone",
                "count": 2,
                "families": {
                    "export": 1,
                    "import": 1
                }
            },
            {
                "reason": "package-scope-unavailable",
                "count": 1,
                "families": {
                    "export": 1
                }
            }
        ])
    );
    assert_eq!(
        summary["blockedCandidateHintFamilyCounts"],
        json!([
            {
                "family": "export",
                "count": 2,
                "reasons": {
                    "package-scope-unavailable": 1,
                    "resolver-blind-zone": 1
                }
            },
            {
                "family": "import",
                "count": 1,
                "reasons": {
                    "resolver-blind-zone": 1
                }
            }
        ])
    );
    assert_eq!(
        summary["topUnresolvedReasons"],
        json!([{ "reason": "alias-prefix-unresolved", "count": 4 }])
    );
    assert_eq!(summary["topSpecifierRoots"][0]["specifierRoot"], "@/");
    assert_eq!(
        summary["topUnresolvedSpecifiers"].as_array().unwrap().len(),
        2
    );
    Ok(())
}

#[test]
fn resolver_diagnostics_summary_falls_back_to_symbols_records() -> Result<()> {
    let symbols = json!({
        "uses": {
            "unresolvedInternal": 5,
            "unresolvedInternalRatio": 0.5
        },
        "unresolvedInternalSummaryByReason": {
            "alias-prefix-unresolved": { "count": 3 },
            "missing-extension": 2,
            "ignored": { "count": "2" }
        },
        "unresolvedInternalSpecifierRecords": [
            {
                "specifier": "@/components/Button",
                "consumerFile": "src/view.ts",
                "reason": "alias-prefix-unresolved"
            },
            {
                "specifier": "@/components/Card",
                "consumerFile": "src/card.ts",
                "reason": "alias-prefix-unresolved"
            },
            {
                "specifier": "@scope/pkg/sub",
                "consumerFile": "src/pkg.ts",
                "reason": "missing-extension"
            },
            {
                "specifier": "~/util",
                "consumerFile": "src/util.ts"
            }
        ],
        "topUnresolvedSpecifiers": [
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j",
            "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u"
        ]
    });

    let summary = serde_json::to_value(summarize_resolver_diagnostics(Some(&symbols), None, None))?;

    assert_eq!(summary["resolverVersion"], json!(null));
    assert_eq!(summary["resolverCapabilityArtifact"], json!(null));
    assert_eq!(summary["resolverDiagnosticsArtifact"], json!(null));
    assert_eq!(summary["blockedCandidateHintSampleLimit"], json!(null));
    assert_eq!(
        summary["topUnresolvedReasons"],
        json!([
            { "reason": "alias-prefix-unresolved", "count": 3 },
            { "reason": "missing-extension", "count": 2 }
        ])
    );
    assert_eq!(
        summary["topSpecifierRoots"],
        json!([
            {
                "specifierRoot": "@/",
                "count": 2,
                "reasons": {
                    "alias-prefix-unresolved": 2
                },
                "examples": [
                    {
                        "specifier": "@/components/Card",
                        "consumerFile": "src/card.ts"
                    },
                    {
                        "specifier": "@/components/Button",
                        "consumerFile": "src/view.ts"
                    }
                ]
            },
            {
                "specifierRoot": "@scope/pkg",
                "count": 1,
                "reasons": {
                    "missing-extension": 1
                },
                "examples": [
                    {
                        "specifier": "@scope/pkg/sub",
                        "consumerFile": "src/pkg.ts"
                    }
                ]
            },
            {
                "specifierRoot": "~/",
                "count": 1,
                "reasons": {
                    "unknown-internal-resolution": 1
                },
                "examples": [
                    {
                        "specifier": "~/util",
                        "consumerFile": "src/util.ts"
                    }
                ]
            }
        ])
    );
    assert_eq!(
        summary["topUnresolvedSpecifiers"].as_array().unwrap().len(),
        20
    );
    Ok(())
}

#[test]
fn resolver_diagnostics_summary_reports_empty_shape_without_inputs() -> Result<()> {
    let summary = serde_json::to_value(summarize_resolver_diagnostics(None, None, None))?;

    assert_eq!(
        summary,
        json!({
            "resolverVersion": null,
            "resolverCapabilityArtifact": null,
            "resolverDiagnosticsArtifact": null,
            "unresolvedInternal": null,
            "unresolvedInternalRatio": null,
            "blindZoneCount": null,
            "blockedCandidateHintCount": null,
            "blockedCandidateHintSampleLimit": null,
            "blockedCandidateHints": [],
            "blockedCandidateHintReasonCounts": [],
            "blockedCandidateHintFamilyCounts": [],
            "candidateTargetCount": null,
            "topFamilies": [],
            "topAffectedPackageScopes": [],
            "topUnresolvedReasons": [],
            "topSpecifierRoots": [],
            "topUnresolvedSpecifiers": []
        })
    );
    Ok(())
}

#[test]
fn cli_resolver_diagnostics_summary_reads_optional_artifacts() -> Result<()> {
    let tempdir = tempfile::tempdir()?;
    let symbols_path = tempdir.path().join("symbols.json");
    let resolver_diagnostics_path = tempdir.path().join("resolver-diagnostics.json");
    fs::write(
        &symbols_path,
        serde_json::to_vec(&json!({
            "uses": {
                "unresolvedInternal": 1,
                "unresolvedInternalRatio": 0.125
            }
        }))?,
    )?;
    fs::write(
        &resolver_diagnostics_path,
        serde_json::to_vec(&json!({
            "resolverVersion": "resolver-v1",
            "summary": {
                "blindZoneCount": 1,
                "blockedCandidateHintCount": 0,
                "candidateTargetCount": 2
            },
            "blockedCandidateHints": []
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("resolver-diagnostics-summary")
        .arg("--symbols")
        .arg(&symbols_path)
        .arg("--resolver-diagnostics")
        .arg(&resolver_diagnostics_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["resolverVersion"], "resolver-v1");
    assert_eq!(stdout["unresolvedInternal"], 1);
    assert_eq!(stdout["blockedCandidateHintSampleLimit"], 10);
    Ok(())
}
