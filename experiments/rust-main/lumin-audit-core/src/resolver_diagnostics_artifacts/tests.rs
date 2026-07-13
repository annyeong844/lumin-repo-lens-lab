use super::*;
use anyhow::{Context, Result};

fn fixture_request() -> Result<ResolverDiagnosticsArtifactsRequest> {
    Ok(serde_json::from_value(json!({
        "schemaVersion": RESOLVER_DIAGNOSTICS_REQUEST_SCHEMA_VERSION,
        "symbols": {
            "uses": {
                "resolvedInternal": 7,
                "unresolvedInternal": 4,
                "unresolvedInternalRatio": 0.3636,
                "external": 2
            },
            "topUnresolvedSpecifiers": [
                { "specifierPrefix": "@scope/orm", "count": 2, "example": "@scope/orm/client" }
            ],
            "unresolvedInternalSpecifierRecords": [
                {
                    "specifier": "#app/config",
                    "consumerFile": "packages/app/src/a.ts",
                    "kind": "import",
                    "reason": "hash-import-target-missing",
                    "resolverStage": "hash-imports",
                    "matchedPattern": "#app/*",
                    "targetCandidates": ["packages/app/src/config"]
                },
                {
                    "specifier": "@scope/orm/client",
                    "consumerFile": "apps/api/src/b.ts",
                    "kind": "import",
                    "reason": "workspace-generated-artifact-missing",
                    "resolverStage": "workspace-package-subpath",
                    "hint": "generated-artifact-missing",
                    "targetCandidates": ["packages/orm/client"],
                    "generatedArtifact": {
                        "policyVersion": "generated-artifact-policy-v1",
                        "matchedPackage": "@scope/orm",
                        "targetSubpath": "client",
                        "generatorFamily": "prisma",
                        "confidence": "strong",
                        "packageRoot": "packages/orm"
                    }
                },
                {
                    "specifier": "app/routes/root",
                    "consumerFile": "apps/web/src/c.ts",
                    "kind": "import",
                    "reason": "tsconfig-path-target-missing",
                    "resolverStage": "tsconfig-paths",
                    "matchedPattern": "app/*",
                    "targetCandidates": ["apps/web/app/routes/root"]
                }
            ],
            "generatedConsumerBlindZones": [
                {
                    "reason": "generated-consumer-blind-zone",
                    "sourceReason": "workspace-generated-artifact-missing",
                    "specifier": "@scope/orm/client",
                    "consumerFile": "apps/api/src/b.ts",
                    "matchedPackage": "@scope/orm",
                    "targetSubpath": "client",
                    "generatorFamily": "prisma",
                    "confidence": "strong",
                    "candidatePath": "packages/orm/client",
                    "status": "missing",
                    "scopePackageRoot": "packages/orm",
                    "mode": "prepared",
                    "staleStatus": "unknown",
                    "staleReason": "generator-input-hash-not-recorded"
                }
            ]
        }
    }))?)
}

fn fixture_artifacts() -> Result<Value> {
    build_resolver_diagnostics_artifacts(fixture_request()?)
}

fn array_field<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .with_context(|| format!("{field} should be an array"))
}

#[test]
fn writes_capability_matrix_and_diagnostics_reference() -> Result<()> {
    let artifacts = fixture_artifacts()?;
    let capabilities = artifacts
        .get("capabilities")
        .context("capabilities should exist")?;
    let diagnostics = artifacts
        .get("diagnostics")
        .context("diagnostics should exist")?;

    assert_eq!(
        capabilities["schemaVersion"],
        RESOLVER_CAPABILITIES_SCHEMA_VERSION
    );
    assert_eq!(capabilities["resolverVersion"], RESOLVER_VERSION);
    assert!(array_field(capabilities, "families")?.iter().any(|family| {
        family["family"] == "node-imports"
            && array_field(family, "reasonCodes")
                .is_ok_and(|codes| codes.contains(&json!("hash-import-target-missing")))
    }));
    assert_eq!(
        diagnostics["capabilityReference"]["schemaVersion"],
        RESOLVER_CAPABILITIES_SCHEMA_VERSION
    );
    assert_eq!(diagnostics["capabilityArtifact"], CAPABILITY_ARTIFACT_NAME);
    Ok(())
}

#[test]
fn preserves_unresolved_imports_candidates_and_blind_zones() -> Result<()> {
    let artifacts = fixture_artifacts()?;
    let diagnostics = artifacts
        .get("diagnostics")
        .context("diagnostics should exist")?;
    assert!(array_field(diagnostics, "unresolvedImports")?
        .iter()
        .any(|item| item["specifier"] == "#app/config"
            && item["family"] == "node-imports"
            && item["outputLevel"] == "unresolved_with_reason"
            && item["reason"] == "hash-import-target-missing"));
    assert!(array_field(diagnostics, "candidateTargets")?
        .iter()
        .any(|item| item["specifier"] == "#app/config"
            && item["proofUse"] == "diagnostic-only"
            && item["createsGraphEdge"] == false));
    assert!(array_field(diagnostics, "blindZones")?
        .iter()
        .any(|zone| zone["reason"] == "generated-consumer-blind-zone"
            && zone["family"] == "generated-artifacts"
            && zone["affectedPackageScope"] == "packages/orm"
            && zone["staleStatus"] == "unknown"));
    Ok(())
}

#[test]
fn emits_candidate_relevant_policies_and_blocked_hints() -> Result<()> {
    let artifacts = fixture_artifacts()?;
    let diagnostics = artifacts
        .get("diagnostics")
        .context("diagnostics should exist")?;
    let hash_zone = array_field(diagnostics, "blindZones")?
        .iter()
        .find(|zone| zone["specifier"] == "#app/config")
        .context("hash import blind zone should exist")?;
    assert_eq!(hash_zone["blockingScope"], "candidate-relevant");
    assert_eq!(
        hash_zone["relevancePolicy"]["policyVersion"],
        RESOLVER_BLIND_ZONE_RELEVANCE_POLICY_VERSION
    );
    assert_eq!(
        hash_zone["relevancePolicy"]["mustNotBlockUnrelatedCandidates"],
        true
    );

    assert!(array_field(diagnostics, "blockedCandidateHints")?
        .iter()
        .any(|hint| hint["family"] == "node-imports"
            && hint["reason"] == "hash-import-target-missing"
            && hint["candidatePath"] == "packages/app/src/config"
            && hint["proofUse"] == "blocks-absence-claim"));
    Ok(())
}

#[test]
fn summary_pivots_are_machine_readable() -> Result<()> {
    let artifacts = fixture_artifacts()?;
    let summary = artifacts
        .pointer("/diagnostics/summary")
        .context("summary should exist")?;
    assert_eq!(summary["unresolvedInternal"], 4);
    assert_eq!(summary["blindZoneCount"], 4);
    assert_eq!(summary["candidateTargetCount"], 3);
    assert!(array_field(summary, "topFamilies")?
        .iter()
        .any(|item| item["family"] == "generated-artifacts" && item["count"] == 3));
    assert!(array_field(summary, "topSpecifierRoots")?
        .iter()
        .any(|item| item["specifierRoot"] == "@scope/orm" && item["count"] == 1));
    Ok(())
}

#[test]
fn rejects_bad_request_shape() {
    let bad_schema = ResolverDiagnosticsArtifactsRequest {
        schema_version: "wrong".to_string(),
        symbols: json!({}),
        capability_artifact: None,
    };
    let err = match build_resolver_diagnostics_artifacts(bad_schema) {
        Ok(_) => panic!("bad schema should fail"),
        Err(error) => error,
    };
    assert!(err.to_string().contains("unsupported schemaVersion"));
}
