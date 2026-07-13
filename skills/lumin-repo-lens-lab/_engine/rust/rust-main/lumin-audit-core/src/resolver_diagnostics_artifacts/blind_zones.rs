use super::classification::{
    affected_package_scope_for_record, family_for_record, generated_blind_zone_blocking_policy,
    resolver_blind_zone_blocking_policy, target_candidates,
};
use super::protocol::Record;
use super::value_support::{
    blind_zone_key, blocked_candidate_hint_key, compact_object, dedupe_by_key,
    non_empty_string_array, sort_by_key, sort_strings,
};
use super::UNKNOWN_INTERNAL_RESOLUTION;
use serde_json::{json, Value};

pub(super) fn build_blind_zones(
    records: &[Value],
    generated_consumer_blind_zones: &[Value],
) -> Vec<Value> {
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

pub(super) fn build_blocked_candidate_hints(blind_zones: &[Value]) -> Vec<Value> {
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
