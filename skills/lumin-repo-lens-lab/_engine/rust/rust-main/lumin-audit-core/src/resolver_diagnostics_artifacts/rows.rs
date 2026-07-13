use super::classification::{family_for_record, target_candidates};
use super::protocol::Record;
use super::value_support::{
    candidate_target_key, compact_object, non_empty_string_array, sort_by_key, sort_strings,
    unresolved_import_key,
};
use super::UNKNOWN_INTERNAL_RESOLUTION;
use serde_json::{json, Value};

pub(super) fn build_unresolved_imports(records: &[Value]) -> Vec<Value> {
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

pub(super) fn build_unsupported_imports(records: &[Value]) -> Vec<Value> {
    let records = records
        .iter()
        .filter(|value| Record::new(value).str("outputLevel") == Some("unsupported"))
        .cloned()
        .collect::<Vec<_>>();
    build_unresolved_imports(&records)
}

pub(super) fn build_candidate_targets(records: &[Value]) -> Vec<Value> {
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
