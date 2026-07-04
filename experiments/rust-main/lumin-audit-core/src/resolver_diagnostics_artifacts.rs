use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverDiagnosticsArtifactsRequest {
    pub schema_version: String,
    #[serde(default)]
    pub symbols: Value,
    #[serde(default)]
    pub capability_artifact: Option<String>,
}

#[derive(Debug)]
struct Record<'a> {
    value: &'a Value,
}

impl<'a> Record<'a> {
    fn new(value: &'a Value) -> Self {
        Self { value }
    }

    fn get(&self, field: &str) -> Option<&'a Value> {
        self.value.as_object()?.get(field)
    }

    fn str(&self, field: &str) -> Option<&'a str> {
        self.get(field).and_then(Value::as_str)
    }

    fn bool(&self, field: &str) -> Option<bool> {
        self.get(field).and_then(Value::as_bool)
    }

    fn number(&self, field: &str) -> Option<Value> {
        self.get(field).filter(|value| value.is_number()).cloned()
    }
}

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

pub fn build_resolver_capabilities_artifact() -> Value {
    json!({
        "schemaVersion": RESOLVER_CAPABILITIES_SCHEMA_VERSION,
        "resolverVersion": RESOLVER_VERSION,
        "conditionProfiles": [
            {
                "profileId": "node-esm-default",
                "conditions": ["node", "import", "default"],
                "configuredBy": "default",
            },
        ],
        "families": resolver_capability_families(),
    })
}

fn resolver_capability_families() -> Value {
    json!([
        {
            "family": "relative-paths",
            "status": "supported",
            "supportedCases": ["extensionless JS/TS files", "directory index files", "runtime JS extension mapped to source TS"],
            "unsupportedCases": [],
            "reasonCodes": ["relative-target-missing"],
            "absenceClaimPolicy": "fail-closed-when-relevant",
            "fixtureRefs": ["resolver-relative-basic"],
        },
        {
            "family": "absolute-project-paths",
            "status": "partial",
            "supportedCases": ["scoped tsconfig baseUrl imports", "root-prefix imports when root segment exists"],
            "unsupportedCases": ["ambiguous project-reference redirected output"],
            "reasonCodes": ["baseurl-target-missing", "unknown-internal-resolution"],
            "absenceClaimPolicy": "fail-closed-when-relevant",
            "fixtureRefs": ["resolver-baseurl-scoped"],
        },
        {
            "family": "node-packages",
            "status": "partial",
            "supportedCases": ["external package sentinel", "workspace package name detection"],
            "unsupportedCases": ["package manager runtime hooks without static package metadata"],
            "reasonCodes": ["unknown-internal-resolution"],
            "absenceClaimPolicy": "fail-closed-when-encountered",
            "fixtureRefs": ["resolver-external-vs-internal"],
        },
        {
            "family": "tsconfig-paths",
            "status": "partial",
            "supportedCases": ["extends chain discovery", "single-star paths", "nearest scope wins", "baseUrl fallback"],
            "unsupportedCases": ["ambiguous multi-target fallback", "project-reference redirected output"],
            "reasonCodes": ["tsconfig-path-target-missing", "exact-alias-target-missing", "wildcard-alias-target-missing"],
            "absenceClaimPolicy": "fail-closed-when-relevant",
            "fixtureRefs": ["resolver-tsconfig-paths-basic"],
        },
        {
            "family": "workspace-packages",
            "status": "partial",
            "supportedCases": ["workspace package root imports", "source-direct main/module/types entries", "legacy subpath source probing"],
            "unsupportedCases": ["generated workspace subpaths without generated artifact support"],
            "reasonCodes": ["workspace-package-subpath-target-missing"],
            "absenceClaimPolicy": "fail-closed-when-relevant",
            "fixtureRefs": ["resolver-workspace-package-subpath"],
        },
        {
            "family": "package-json-exports",
            "status": "partial",
            "supportedCases": ["string targets", "subpath wildcard targets", "source remapping for dist outputs"],
            "unsupportedCases": ["ambiguous conditional maps without configured condition profile", "array fallback ordering beyond supported source probes", "non-standard output-to-source layouts without explicit source metadata"],
            "reasonCodes": ["workspace-package-subpath-target-missing", "condition-profile-ambiguous", "output-source-layout-unsupported"],
            "absenceClaimPolicy": "fail-closed-when-relevant",
            "fixtureRefs": ["resolver-package-json-exports"],
        },
        {
            "family": "output-to-source-mapping",
            "status": "partial",
            "supportedCases": ["dist/build/out/es/esm/distribution output directories mapped to source conventions"],
            "unsupportedCases": ["workspace package exports pointing at non-standard compiled output directories without an explicit source condition"],
            "reasonCodes": ["output-source-layout-unsupported"],
            "absenceClaimPolicy": "fail-closed-when-relevant",
            "fixtureRefs": ["resolver-output-source-layout-unsupported"],
        },
        {
            "family": "package-json-entry-fields",
            "status": "partial",
            "supportedCases": ["main", "module", "types", "browser as package entry candidates"],
            "unsupportedCases": ["environment-specific browser/node divergence without explicit condition profile"],
            "reasonCodes": ["workspace-package-subpath-target-missing"],
            "absenceClaimPolicy": "fail-closed-when-relevant",
            "fixtureRefs": ["resolver-package-entry-fields"],
        },
        {
            "family": "node-imports",
            "status": "partial",
            "supportedCases": ["package-local #imports wildcard maps"],
            "unsupportedCases": ["ambiguous condition maps in imports", "custom condition profiles not configured by scan"],
            "reasonCodes": ["condition-profile-ambiguous", "hash-import-target-missing", "hash-imports-unsupported"],
            "absenceClaimPolicy": "fail-closed-when-encountered",
            "fixtureRefs": ["resolver-node-imports-hash-wildcard"],
        },
        {
            "family": "json-imports",
            "status": "partial",
            "supportedCases": ["file-level non-source asset reachability"],
            "unsupportedCases": ["named JS export identity from JSON without an explicit transform"],
            "reasonCodes": [],
            "absenceClaimPolicy": "file-reachability-only",
            "fixtureRefs": ["resolver-json-file-edge"],
        },
        {
            "family": "generated-artifacts",
            "status": "partial",
            "supportedCases": ["generated artifact miss taxonomy", "generated consumer blind-zone diagnostics", "Prisma enum virtual surface"],
            "unsupportedCases": ["generator execution by default", "runtime equivalence for virtual surfaces"],
            "reasonCodes": ["workspace-generated-artifact-missing", "generated-consumer-blind-zone"],
            "absenceClaimPolicy": "fail-closed-when-relevant",
            "fixtureRefs": ["resolver-generated-artifact-missing"],
        },
        {
            "family": "dynamic-modules",
            "status": "partial",
            "supportedCases": ["literal dynamic import() member precision"],
            "unsupportedCases": ["import.meta.glob expansion and non-literal dynamic module discovery"],
            "reasonCodes": ["import-meta-glob-unsupported"],
            "absenceClaimPolicy": "fail-closed-when-relevant",
            "fixtureRefs": ["resolver-import-meta-glob-unsupported"],
        },
        {
            "family": "conditional-exports",
            "status": "partial",
            "supportedCases": ["default node/import condition profile"],
            "unsupportedCases": ["browser/node ambiguity", "custom conditions without configured profile"],
            "reasonCodes": ["condition-profile-ambiguous"],
            "absenceClaimPolicy": "fail-closed-when-relevant",
            "fixtureRefs": ["resolver-conditional-exports-basic"],
        },
        {
            "family": "re-export-aliases",
            "status": "supported",
            "supportedCases": ["exported-name to local-name tracking", "definition id preservation"],
            "unsupportedCases": [],
            "reasonCodes": [],
            "absenceClaimPolicy": "definition-identity-preserved",
            "fixtureRefs": ["resolver-re-export-alias-identity"],
        },
    ])
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

fn build_unresolved_imports(records: &[Value]) -> Vec<Value> {
    sort_by_key(
        records
            .iter()
            .map(|value| {
                let record = Record::new(value);
                compact_object(vec![
                    ("specifier", record.get("specifier").cloned()),
                    (
                        "importer",
                        record
                            .get("consumerFile")
                            .or_else(|| record.get("fromHint"))
                            .cloned(),
                    ),
                    ("kind", record.get("kind").cloned()),
                    ("typeOnly", record.bool("typeOnly").map(Value::Bool)),
                    ("family", Some(json!(family_for_record(&record)))),
                    (
                        "reason",
                        Some(json!(record
                            .str("reason")
                            .unwrap_or(UNKNOWN_INTERNAL_RESOLUTION))),
                    ),
                    ("resolverStage", record.get("resolverStage").cloned()),
                    (
                        "outputLevel",
                        Some(json!(record
                            .str("outputLevel")
                            .unwrap_or("unresolved_with_reason"))),
                    ),
                    (
                        "unsupportedFamily",
                        record.get("unsupportedFamily").cloned(),
                    ),
                    ("createsGraphEdge", Some(Value::Bool(false))),
                    ("matchedPattern", record.get("matchedPattern").cloned()),
                    ("source", record.get("source").cloned()),
                    (
                        "targetCandidates",
                        non_empty_string_array(target_candidates(&record)),
                    ),
                    ("hint", record.get("hint").cloned()),
                    ("matchCount", record.number("matchCount")),
                    ("cap", record.number("cap")),
                    ("scanPolicy", record.get("scanPolicy").cloned()),
                    (
                        "generatedArtifact",
                        record.get("generatedArtifact").cloned(),
                    ),
                ])
            })
            .collect(),
        unresolved_import_key,
    )
}

fn build_unsupported_imports(records: &[Value]) -> Vec<Value> {
    let records = records
        .iter()
        .filter(|value| Record::new(value).str("outputLevel") == Some("unsupported"))
        .cloned()
        .collect::<Vec<_>>();
    build_unresolved_imports(&records)
}

fn build_candidate_targets(records: &[Value]) -> Vec<Value> {
    let mut items = Vec::new();
    for value in records {
        let record = Record::new(value);
        let candidates = target_candidates(&record);
        if candidates.is_empty() {
            continue;
        }
        items.push(compact_object(vec![
            ("specifier", record.get("specifier").cloned()),
            (
                "importer",
                record
                    .get("consumerFile")
                    .or_else(|| record.get("fromHint"))
                    .cloned(),
            ),
            ("family", Some(json!(family_for_record(&record)))),
            ("outputLevel", Some(json!("candidate"))),
            ("proofUse", Some(json!("diagnostic-only"))),
            ("createsGraphEdge", Some(Value::Bool(false))),
            ("candidatePaths", Some(json!(sort_strings(candidates)))),
            (
                "notResolvedBecause",
                Some(json!(record
                    .str("reason")
                    .unwrap_or(UNKNOWN_INTERNAL_RESOLUTION))),
            ),
            ("resolverStage", record.get("resolverStage").cloned()),
        ]));
    }
    sort_by_key(items, candidate_target_key)
}

fn build_blind_zones(records: &[Value], generated_consumer_blind_zones: &[Value]) -> Vec<Value> {
    let mut zones = records
        .iter()
        .map(blind_zone_from_record)
        .chain(
            generated_consumer_blind_zones
                .iter()
                .map(blind_zone_from_generated_consumer),
        )
        .collect::<Vec<_>>();
    zones = sort_by_key(zones, blind_zone_key);
    dedupe_by_key(zones, blind_zone_key)
}

fn blind_zone_from_record(value: &Value) -> Value {
    let record = Record::new(value);
    let family = family_for_record(&record);
    let reason = record.str("reason").unwrap_or(UNKNOWN_INTERNAL_RESOLUTION);
    let generated = family == "generated-artifacts";
    let relevance_policy = if generated {
        generated_blind_zone_blocking_policy(false)
    } else {
        resolver_blind_zone_blocking_policy(&record)
    };
    compact_object(vec![
        ("family", Some(json!(family))),
        ("reason", Some(json!(reason))),
        (
            "importer",
            record
                .get("consumerFile")
                .or_else(|| record.get("fromHint"))
                .cloned(),
        ),
        ("specifier", record.get("specifier").cloned()),
        ("resolverStage", record.get("resolverStage").cloned()),
        (
            "outputLevel",
            Some(json!(record
                .str("outputLevel")
                .unwrap_or("unresolved_with_reason"))),
        ),
        (
            "unsupportedFamily",
            record.get("unsupportedFamily").cloned(),
        ),
        (
            "affectedPackageScope",
            affected_package_scope_for_record(&record).map(Value::String),
        ),
        ("blocksAbsenceClaims", Some(Value::Bool(true))),
        (
            "blockingScope",
            relevance_policy.get("blockingScope").cloned(),
        ),
        ("relevancePolicy", Some(relevance_policy)),
        (
            "relevance",
            Some(json!(if generated {
                "generated-provider-surface"
            } else {
                "unresolved-internal-surface"
            })),
        ),
        (
            "targetCandidates",
            non_empty_string_array(target_candidates(&record)),
        ),
        ("matchCount", record.number("matchCount")),
        ("cap", record.number("cap")),
        ("scanPolicy", record.get("scanPolicy").cloned()),
        ("typeOnly", record.bool("typeOnly").map(Value::Bool)),
        (
            "generatedArtifact",
            record.get("generatedArtifact").cloned(),
        ),
    ])
}

fn blind_zone_from_generated_consumer(value: &Value) -> Value {
    let zone = Record::new(value);
    let relevance_policy = generated_blind_zone_blocking_policy(true);
    compact_object(vec![
        ("family", Some(json!("generated-artifacts"))),
        ("reason", zone.get("reason").cloned()),
        ("sourceReason", zone.get("sourceReason").cloned()),
        ("importer", zone.get("consumerFile").cloned()),
        ("specifier", zone.get("specifier").cloned()),
        ("outputLevel", Some(json!("unresolved_with_reason"))),
        (
            "affectedPackageScope",
            zone.get("scopePackageRoot").cloned(),
        ),
        ("blocksAbsenceClaims", Some(Value::Bool(true))),
        (
            "blockingScope",
            relevance_policy.get("blockingScope").cloned(),
        ),
        ("relevancePolicy", Some(relevance_policy)),
        ("relevance", Some(json!("generated-consumer-scope"))),
        ("candidatePath", zone.get("candidatePath").cloned()),
        ("status", zone.get("status").cloned()),
        ("mode", zone.get("mode").cloned()),
        ("staleStatus", zone.get("staleStatus").cloned()),
        ("staleReason", zone.get("staleReason").cloned()),
        ("matchedPackage", zone.get("matchedPackage").cloned()),
        ("targetSubpath", zone.get("targetSubpath").cloned()),
        ("generatorFamily", zone.get("generatorFamily").cloned()),
        ("confidence", zone.get("confidence").cloned()),
    ])
}

fn build_blocked_candidate_hints(blind_zones: &[Value]) -> Vec<Value> {
    let mut hints = Vec::new();
    for zone_value in blind_zones {
        let zone = Record::new(zone_value);
        if zone.get("blocksAbsenceClaims") != Some(&Value::Bool(true))
            || zone.str("blockingScope") != Some("candidate-relevant")
        {
            continue;
        }
        let base = compact_object(vec![
            ("family", zone.get("family").cloned()),
            ("reason", zone.get("reason").cloned()),
            ("importer", zone.get("importer").cloned()),
            ("specifier", zone.get("specifier").cloned()),
            (
                "affectedPackageScope",
                zone.get("affectedPackageScope").cloned(),
            ),
            ("blockingScope", zone.get("blockingScope").cloned()),
            ("relevance", zone.get("relevance").cloned()),
            ("proofUse", Some(json!("blocks-absence-claim"))),
            ("outputLevel", zone.get("outputLevel").cloned()),
        ]);
        let mut paths = Vec::new();
        if let Some(candidate_path) = zone.str("candidatePath") {
            paths.push(candidate_path.to_string());
        }
        if let Some(targets) = zone.get("targetCandidates").and_then(Value::as_array) {
            paths.extend(
                targets
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned),
            );
        }
        paths = sort_strings(paths);
        if paths.is_empty() {
            hints.push(base);
            continue;
        }
        for candidate_path in paths {
            let mut object = base.as_object().cloned().unwrap_or_default();
            object.insert("candidatePath".to_string(), json!(candidate_path));
            hints.push(Value::Object(object));
        }
    }
    let hints = sort_by_key(hints, blocked_candidate_hint_key);
    dedupe_by_key(hints, blocked_candidate_hint_key)
}

fn top_families(unresolved_imports: &[Value], blind_zones: &[Value]) -> Vec<Value> {
    let values = unresolved_imports
        .iter()
        .chain(blind_zones.iter())
        .cloned()
        .collect::<Vec<_>>();
    count_by(&values, |record| {
        record
            .str("family")
            .filter(|family| !family.is_empty())
            .map(ToOwned::to_owned)
    })
    .into_iter()
    .take(20)
    .map(|(family, count)| json!({ "family": family, "count": count }))
    .collect()
}

fn top_affected_package_scopes(blind_zones: &[Value]) -> Vec<Value> {
    count_by(blind_zones, |record| {
        record
            .str("affectedPackageScope")
            .filter(|scope| !scope.is_empty())
            .map(ToOwned::to_owned)
    })
    .into_iter()
    .take(20)
    .map(|(affected_package_scope, count)| {
        json!({ "affectedPackageScope": affected_package_scope, "count": count })
    })
    .collect()
}

fn top_unresolved_reasons(records: &[Value]) -> Vec<Value> {
    count_by(records, |record| {
        Some(
            record
                .str("reason")
                .unwrap_or(UNKNOWN_INTERNAL_RESOLUTION)
                .to_string(),
        )
    })
    .into_iter()
    .take(20)
    .map(|(reason, count)| json!({ "reason": reason, "count": count }))
    .collect()
}

fn top_specifier_roots(records: &[Value]) -> Vec<Value> {
    let mut groups = BTreeMap::<String, SpecifierRootGroup>::new();
    for value in records {
        let record = Record::new(value);
        let Some(specifier_root) = record.str("specifier").and_then(unresolved_specifier_root)
        else {
            continue;
        };
        let group = groups
            .entry(specifier_root.clone())
            .or_insert_with(|| SpecifierRootGroup::new(specifier_root));
        let reason = record.str("reason").unwrap_or(UNKNOWN_INTERNAL_RESOLUTION);
        group.count += 1;
        *group.reasons.entry(reason.to_string()).or_default() += 1;
        group.examples.push(compact_object(vec![
            ("specifier", record.get("specifier").cloned()),
            (
                "consumerFile",
                record
                    .get("consumerFile")
                    .or_else(|| record.get("fromHint"))
                    .cloned()
                    .or(Some(Value::Null)),
            ),
        ]));
    }
    let mut out = groups
        .into_values()
        .map(SpecifierRootGroup::finish)
        .collect::<Vec<_>>();
    out.sort_by(|left, right| {
        value_usize(right, "count")
            .cmp(&value_usize(left, "count"))
            .then_with(|| {
                value_string(left, "specifierRoot").cmp(&value_string(right, "specifierRoot"))
            })
    });
    out.truncate(20);
    out
}

fn counter_object_from_values(
    records: &[Value],
    key_fn: impl Fn(Record<'_>) -> String,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in records {
        let key = key_fn(Record::new(value));
        if key.is_empty() {
            continue;
        }
        *counts.entry(key).or_default() += 1;
    }
    counts
}

fn count_by(
    values: &[Value],
    key_fn: impl Fn(Record<'_>) -> Option<String>,
) -> Vec<(String, usize)> {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in values {
        let Some(key) = key_fn(Record::new(value)) else {
            continue;
        };
        if key.is_empty() {
            continue;
        }
        *counts.entry(key).or_default() += 1;
    }
    let mut out = counts.into_iter().collect::<Vec<_>>();
    out.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    out
}

fn family_for_record(record: &Record<'_>) -> &'static str {
    let reason = record.str("reason");
    let stage = record.str("resolverStage");
    let specifier = record.str("specifier").unwrap_or_default();
    if reason == Some(GENERATED_ARTIFACT_MISSING_REASON)
        || record.str("hint") == Some("generated-artifact-missing")
        || record.get("generatedArtifact").is_some()
    {
        return "generated-artifacts";
    }
    if stage == Some("import-meta-glob")
        || record.str("unsupportedFamily") == Some("dynamic-modules")
    {
        return "dynamic-modules";
    }
    if record.str("unsupportedFamily") == Some("output-to-source-mapping")
        || reason == Some("output-source-layout-unsupported")
    {
        return "output-to-source-mapping";
    }
    if stage == Some("hash-imports") || specifier.starts_with('#') {
        return "node-imports";
    }
    if stage == Some("relative") {
        return "relative-paths";
    }
    if stage == Some("tsconfig-baseurl") || reason == Some("baseurl-target-missing") {
        return "absolute-project-paths";
    }
    if reason == Some("workspace-package-subpath-target-missing") {
        return "workspace-packages";
    }
    if matches!(
        stage,
        Some("tsconfig-paths" | "exact-alias" | "wildcard-alias")
    ) {
        return "tsconfig-paths";
    }
    if reason == Some(UNKNOWN_INTERNAL_RESOLUTION) {
        return "unknown-internal-resolution";
    }
    "unknown"
}

fn resolver_blind_zone_blocking_policy(record: &Record<'_>) -> Value {
    let candidates = target_candidates(record);
    let has_explicit_scope =
        record.str("affectedPackageScope").is_some() || record.str("packageRoot").is_some();
    let mut candidate_relevant_when = Vec::new();
    if !candidates.is_empty() {
        candidate_relevant_when.extend([
            "target-candidate-file",
            "target-candidate-package-scope",
            "target-candidate-submodule",
        ]);
    }
    if has_explicit_scope {
        candidate_relevant_when.push("affected-package-scope");
    }
    if !candidate_relevant_when.is_empty() {
        return json!({
            "policyVersion": RESOLVER_BLIND_ZONE_RELEVANCE_POLICY_VERSION,
            "blockingScope": "candidate-relevant",
            "candidateRelevantWhen": unique_sorted(candidate_relevant_when),
            "mustNotBlockUnrelatedCandidates": true,
        });
    }
    json!({
        "policyVersion": RESOLVER_BLIND_ZONE_RELEVANCE_POLICY_VERSION,
        "blockingScope": "repo-confidence-limited",
        "candidateRelevantWhen": ["owner-unknown-internal"],
        "mustNotBlockUnrelatedCandidates": false,
    })
}

fn generated_blind_zone_blocking_policy(consumer_zone: bool) -> Value {
    json!({
        "policyVersion": GENERATED_BLIND_ZONE_RELEVANCE_POLICY_VERSION,
        "blockingScope": "candidate-relevant",
        "candidateRelevantWhen": if consumer_zone {
            json!(["generated-consumer-scope", "generated-consumer-target-submodule"])
        } else {
            json!(["matched-package-root", "target-candidate-submodule"])
        },
        "mustNotBlockUnrelatedCandidates": true,
    })
}

fn affected_package_scope_for_record(record: &Record<'_>) -> Option<String> {
    if let Some(scope) = record.str("affectedPackageScope") {
        return Some(scope.to_string());
    }
    let artifact = record.get("generatedArtifact").and_then(Value::as_object);
    for field in ["packageRoot", "packageDir", "workspaceRoot"] {
        if let Some(value) = artifact
            .and_then(|artifact| artifact.get(field))
            .and_then(Value::as_str)
        {
            return Some(value.to_string());
        }
    }
    for candidate in target_candidates(record) {
        if let Some(root) = package_root_from_path(&candidate) {
            return Some(root);
        }
    }
    record
        .str("consumerFile")
        .or_else(|| record.str("fromHint"))
        .and_then(package_root_from_path)
}

fn target_candidates(record: &Record<'_>) -> Vec<String> {
    record
        .get("targetCandidates")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn package_root_from_path(candidate_path: &str) -> Option<String> {
    let normalized = slash(candidate_path).trim_start_matches("./").to_string();
    let parts = normalized.split('/').collect::<Vec<_>>();
    if matches!(parts.first(), Some(&"apps" | &"packages")) && parts.len() >= 2 {
        return Some(format!("{}/{}", parts[0], parts[1]));
    }
    None
}

fn unresolved_specifier_root(specifier: &str) -> Option<String> {
    if specifier.is_empty() {
        return None;
    }
    if specifier.starts_with("@/") || specifier.starts_with("~/") || specifier.starts_with("#/") {
        return Some(specifier[..2].to_string());
    }
    if specifier.starts_with('#') {
        return Some("#".to_string());
    }
    if specifier.starts_with('@') {
        let parts = specifier.split('/').collect::<Vec<_>>();
        if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Some(format!("{}/{}", parts[0], parts[1]));
        }
    }
    specifier
        .split('/')
        .next()
        .filter(|first| !first.is_empty())
        .map(ToOwned::to_owned)
}

fn compact_object(entries: Vec<(&str, Option<Value>)>) -> Value {
    let mut object = Map::new();
    for (key, value) in entries {
        let Some(value) = value else {
            continue;
        };
        if value.is_null() {
            continue;
        }
        object.insert(key.to_string(), value);
    }
    Value::Object(object)
}

fn non_empty_string_array(values: Vec<String>) -> Option<Value> {
    (!values.is_empty()).then(|| json!(sort_strings(values)))
}

fn sort_strings(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn unique_sorted(values: Vec<&str>) -> Vec<&str> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sort_by_key(values: Vec<Value>, key_fn: impl Fn(&Value) -> String) -> Vec<Value> {
    let mut values = values;
    values.sort_by_key(|value| key_fn(value));
    values
}

fn dedupe_by_key(values: Vec<Value>, key_fn: impl Fn(&Value) -> String) -> Vec<Value> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for value in values {
        let key = key_fn(&value);
        if seen.insert(key) {
            out.push(value);
        }
    }
    out
}

fn array_field<'a>(value: &'a Value, field: &str) -> &'a [Value] {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn slash(value: &str) -> String {
    value
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

fn value_string(value: &Value, field: &str) -> String {
    value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn value_usize(value: &Value, field: &str) -> usize {
    value
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|value| value.try_into().ok())
        .unwrap_or_default()
}

fn unresolved_import_key(item: &Value) -> String {
    let item = Record::new(item);
    [
        item.str("importer").unwrap_or_default(),
        item.str("specifier").unwrap_or_default(),
        item.str("kind").unwrap_or_default(),
        item.str("reason").unwrap_or_default(),
    ]
    .join("|")
}

fn blind_zone_key(item: &Value) -> String {
    let item = Record::new(item);
    [
        item.str("family").unwrap_or_default(),
        item.str("reason").unwrap_or_default(),
        item.str("importer").unwrap_or_default(),
        item.str("specifier").unwrap_or_default(),
        item.str("affectedPackageScope").unwrap_or_default(),
        item.str("candidatePath").unwrap_or_default(),
    ]
    .join("|")
}

fn candidate_target_key(item: &Value) -> String {
    let item = Record::new(item);
    let mut parts = vec![
        item.str("importer").unwrap_or_default().to_string(),
        item.str("specifier").unwrap_or_default().to_string(),
        item.str("family").unwrap_or_default().to_string(),
        item.str("notResolvedBecause")
            .unwrap_or_default()
            .to_string(),
    ];
    if let Some(paths) = item.get("candidatePaths").and_then(Value::as_array) {
        parts.extend(
            paths
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned),
        );
    }
    parts.join("|")
}

fn blocked_candidate_hint_key(item: &Value) -> String {
    let item = Record::new(item);
    [
        item.str("family").unwrap_or_default(),
        item.str("reason").unwrap_or_default(),
        item.str("importer").unwrap_or_default(),
        item.str("specifier").unwrap_or_default(),
        item.str("affectedPackageScope").unwrap_or_default(),
        item.str("candidatePath").unwrap_or_default(),
        item.str("relevance").unwrap_or_default(),
    ]
    .join("|")
}

struct SpecifierRootGroup {
    specifier_root: String,
    count: usize,
    reasons: BTreeMap<String, usize>,
    examples: Vec<Value>,
}

impl SpecifierRootGroup {
    fn new(specifier_root: String) -> Self {
        Self {
            specifier_root,
            count: 0,
            reasons: BTreeMap::new(),
            examples: Vec::new(),
        }
    }

    fn finish(mut self) -> Value {
        self.examples.sort_by_key(|example| {
            format!(
                "{}|{}",
                example
                    .get("consumerFile")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                example
                    .get("specifier")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
            )
        });
        self.examples.truncate(5);
        json!({
            "specifierRoot": self.specifier_root,
            "count": self.count,
            "reasons": self.reasons,
            "examples": self.examples,
        })
    }
}

#[cfg(test)]
mod tests {
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
}
