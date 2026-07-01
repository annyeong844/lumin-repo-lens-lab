use lumin_rust_common::sha256_text;
use serde_json::Value;

pub(crate) const RESOLVER_RATIO_THRESHOLD: f64 = 0.15;
pub(crate) const RESOLVER_ABSOLUTE_UNRESOLVED_THRESHOLD: u64 = 1000;
pub(crate) const RESOLVER_PREFIX_CONCENTRATION_MIN_UNRESOLVED: u64 = 100;
pub(crate) const RESOLVER_PREFIX_CONCENTRATION_MIN_COUNT: u64 = 100;
pub(crate) const RESOLVER_PREFIX_CONCENTRATION_SHARE: f64 = 0.8;
pub(crate) const SHAPE_UNKNOWN_FILE_SHARE: f64 = 0.1;

pub(crate) fn resolver_blind_zone_policy_summary() -> Value {
    let thresholds = serde_json::json!({
        "unresolvedRatio": RESOLVER_RATIO_THRESHOLD,
        "absoluteUnresolvedCount": RESOLVER_ABSOLUTE_UNRESOLVED_THRESHOLD,
        "prefixConcentrationMinUnresolved": RESOLVER_PREFIX_CONCENTRATION_MIN_UNRESOLVED,
        "prefixConcentrationMinCount": RESOLVER_PREFIX_CONCENTRATION_MIN_COUNT,
        "prefixConcentrationShare": RESOLVER_PREFIX_CONCENTRATION_SHARE,
        "shapeUnknownFileShare": SHAPE_UNKNOWN_FILE_SHARE,
    });
    let threshold_hash = sha256_text(&canonical_json(&thresholds));
    let calibration = serde_json::json!({
        "corpus": "calibration-2026-05-resolver-v1",
        "note": "agent-entry resolver completeness contract",
    });
    let policy_without_hash = serde_json::json!({
        "schemaVersion": "threshold-policy.v1",
        "policyId": "resolver-blind-zone-policy",
        "policyVersion": "resolver-blind-zone-policy-v1",
        "policyClass": "confidence",
        "thresholds": thresholds,
        "calibration": calibration,
        "notes": [
            "Resolver confidence gaps limit absence claims.",
            "The policy should not become a repo-global blocker when relevance can be scoped.",
        ],
        "thresholdHash": threshold_hash,
    });
    let policy_hash = sha256_text(&canonical_json(&policy_without_hash));
    serde_json::json!({
        "schemaVersion": "threshold-policy.v1",
        "policyId": "resolver-blind-zone-policy",
        "policyVersion": "resolver-blind-zone-policy-v1",
        "policyClass": "confidence",
        "policyHash": policy_hash,
        "thresholdHash": threshold_hash,
        "thresholds": policy_without_hash["thresholds"].clone(),
        "calibration": policy_without_hash["calibration"].clone(),
        "calibrationCorpus": {
            "schemaVersion": "calibration-corpus.v1",
            "corpusId": "calibration-2026-05-resolver-v1",
            "purpose": "resolver blind-zone and completeness calibration",
            "status": "registry-anchor",
            "metrics": [
                "unresolvedInternalRate",
                "blindZoneCount",
                "falseGlobalBlockerCount",
                "affectedPackageScopeCount",
                "runtimeMs",
            ],
            "entryCount": 3,
        },
    })
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
        }
        Value::Array(values) => {
            let inner = values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{inner}]")
        }
        Value::Object(map) => {
            let mut keys = map
                .keys()
                .filter(|key| key.as_str() != "policyHash")
                .collect::<Vec<_>>();
            keys.sort();
            let inner = keys
                .into_iter()
                .map(|key| {
                    let encoded_key =
                        serde_json::to_string(key).unwrap_or_else(|_| "\"\"".to_string());
                    format!("{encoded_key}:{}", canonical_json(&map[key]))
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{inner}}}")
        }
    }
}
