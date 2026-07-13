mod blind_zones;
mod capabilities;
mod classification;
mod protocol;
mod rows;
mod summary;
mod value_support;

#[cfg(test)]
mod tests;

use anyhow::{bail, Result};
use blind_zones::{build_blind_zones, build_blocked_candidate_hints};
pub use capabilities::build_resolver_capabilities_artifact;
pub use protocol::ResolverDiagnosticsArtifactsRequest;
use rows::{build_candidate_targets, build_unresolved_imports, build_unsupported_imports};
use serde_json::{json, Value};
use summary::{
    counter_object_from_values, top_affected_package_scopes, top_families, top_specifier_roots,
    top_unresolved_reasons,
};
use value_support::array_field;

pub const RESOLVER_CAPABILITIES_SCHEMA_VERSION: &str = "resolver-capabilities.v1";
pub const RESOLVER_DIAGNOSTICS_SCHEMA_VERSION: &str = "resolver-diagnostics.v1";
pub const RESOLVER_DIAGNOSTICS_REQUEST_SCHEMA_VERSION: &str =
    "lumin-resolver-diagnostics-producer-request.v1";
pub const RESOLVER_VERSION: &str = "resolver-2026-05-v1";

const CAPABILITY_ARTIFACT_NAME: &str = "resolver-capabilities.json";
const UNKNOWN_INTERNAL_RESOLUTION: &str = "unknown-internal-resolution";
const GENERATED_ARTIFACT_MISSING_REASON: &str = "workspace-generated-artifact-missing";
const RESOLVER_BLIND_ZONE_RELEVANCE_POLICY_VERSION: &str = "resolver-blind-zone-relevance.v1";
const GENERATED_BLIND_ZONE_RELEVANCE_POLICY_VERSION: &str = "generated-blind-zone-relevance.v1";

pub fn build_resolver_diagnostics_artifacts(
    request: ResolverDiagnosticsArtifactsRequest,
) -> Result<Value> {
    if request.schema_version != RESOLVER_DIAGNOSTICS_REQUEST_SCHEMA_VERSION {
        bail!(
            "resolver-diagnostics-artifacts: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if !request.symbols.is_object() {
        bail!("resolver-diagnostics-artifacts: symbols must be an object");
    }

    let capability_artifact = request
        .capability_artifact
        .unwrap_or_else(|| CAPABILITY_ARTIFACT_NAME.to_string());
    let capabilities = build_resolver_capabilities_artifact();
    let diagnostics = build_resolver_diagnostics_artifact(&request.symbols, &capability_artifact);
    Ok(json!({
        "capabilities": capabilities,
        "diagnostics": diagnostics,
    }))
}

fn build_resolver_diagnostics_artifact(symbols: &Value, capability_artifact: &str) -> Value {
    let records = array_field(symbols, "unresolvedInternalSpecifierRecords");
    let generated_consumer_blind_zones = array_field(symbols, "generatedConsumerBlindZones");
    let unresolved_imports = build_unresolved_imports(records);
    let unsupported_imports = build_unsupported_imports(records);
    let candidate_targets = build_candidate_targets(records);
    let blind_zones = build_blind_zones(records, generated_consumer_blind_zones);
    let blocked_candidate_hints = build_blocked_candidate_hints(&blind_zones);

    json!({
        "schemaVersion": RESOLVER_DIAGNOSTICS_SCHEMA_VERSION,
        "resolverVersion": RESOLVER_VERSION,
        "capabilityArtifact": capability_artifact,
        "capabilityReference": {
            "artifact": capability_artifact,
            "schemaVersion": RESOLVER_CAPABILITIES_SCHEMA_VERSION,
            "resolverVersion": RESOLVER_VERSION,
        },
        "summary": {
            "unresolvedInternal": symbols.pointer("/uses/unresolvedInternal").cloned().unwrap_or_else(|| json!(records.len())),
            "unresolvedInternalRatio": symbols.pointer("/uses/unresolvedInternalRatio").cloned().unwrap_or(Value::Null),
            "externalImports": symbols.pointer("/uses/external").cloned().unwrap_or(Value::Null),
            "blindZoneCount": blind_zones.len(),
            "blockedCandidateHintCount": blocked_candidate_hints.len(),
            "candidateTargetCount": candidate_targets.len(),
            "unresolvedImportCount": unresolved_imports.len(),
            "unsupportedImportCount": unsupported_imports.len(),
            "topFamilies": top_families(&unresolved_imports, &blind_zones),
            "topAffectedPackageScopes": top_affected_package_scopes(&blind_zones),
            "topUnresolvedReasons": top_unresolved_reasons(records),
            "topSpecifierRoots": top_specifier_roots(records),
            "reasonCounts": counter_object_from_values(records, |record| {
                record.str("reason").unwrap_or(UNKNOWN_INTERNAL_RESOLUTION).to_string()
            }),
        },
        "blindZones": blind_zones,
        "blockedCandidateHints": blocked_candidate_hints,
        "candidateTargets": candidate_targets,
        "unsupportedImports": unsupported_imports,
        "unresolvedImports": unresolved_imports,
        "topUnresolvedSpecifiers": symbols.get("topUnresolvedSpecifiers")
            .and_then(Value::as_array)
            .map(|items| items.iter().take(20).cloned().collect::<Vec<_>>())
            .unwrap_or_default(),
    })
}
