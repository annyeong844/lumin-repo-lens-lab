use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

use super::policy::{
    resolver_blind_zone_policy_summary, RESOLVER_ABSOLUTE_UNRESOLVED_THRESHOLD,
    RESOLVER_PREFIX_CONCENTRATION_MIN_COUNT, RESOLVER_PREFIX_CONCENTRATION_MIN_UNRESOLVED,
    RESOLVER_PREFIX_CONCENTRATION_SHARE, RESOLVER_RATIO_THRESHOLD,
};
use super::value::{
    first_number, first_u64, get_path_opt, insert_existing, insert_optional_number,
    insert_optional_u64, number_u64, optional_array,
};
use super::{zone, BlindZoneSeverity, BlindZoneSummary};

pub(crate) fn detect_resolver_zone(
    symbols: Option<&Value>,
    resolver_diagnostics: Option<&Value>,
) -> Option<BlindZoneSummary> {
    let diagnostic_summary = resolver_diagnostics_summary(resolver_diagnostics);
    let ratio = first_number(&[
        get_path_opt(symbols, &["uses", "unresolvedInternalRatio"]),
        diagnostic_summary.and_then(|summary| summary.get("unresolvedInternalRatio")),
    ]);
    let unresolved_internal = first_u64(&[
        get_path_opt(symbols, &["uses", "unresolvedInternal"]),
        diagnostic_summary.and_then(|summary| summary.get("unresolvedInternal")),
    ]);
    let top = optional_array(get_path_opt(symbols, &["topUnresolvedSpecifiers"]))
        .or_else(|| {
            optional_array(
                diagnostic_summary.and_then(|summary| summary.get("topUnresolvedSpecifiers")),
            )
        })
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let top_count = top
        .first()
        .and_then(|item| item.get("count"))
        .and_then(number_u64);
    let ratio_trigger = ratio.is_some_and(|value| value >= RESOLVER_RATIO_THRESHOLD);
    let absolute_trigger =
        unresolved_internal.is_some_and(|count| count >= RESOLVER_ABSOLUTE_UNRESOLVED_THRESHOLD);
    let prefix_trigger = unresolved_internal.is_some_and(|unresolved| {
        unresolved >= RESOLVER_PREFIX_CONCENTRATION_MIN_UNRESOLVED
            && top_count.is_some_and(|count| {
                count >= RESOLVER_PREFIX_CONCENTRATION_MIN_COUNT
                    && count as f64 / unresolved.max(1) as f64
                        >= RESOLVER_PREFIX_CONCENTRATION_SHARE
            })
    });
    if !ratio_trigger && !absolute_trigger && !prefix_trigger {
        return None;
    }

    let trigger = if ratio_trigger {
        "ratio"
    } else if absolute_trigger {
        "absolute-count"
    } else {
        "prefix-concentration"
    };
    let has_diagnostic_summary = diagnostic_summary.is_some();
    let mut details = Map::new();
    details.insert("trigger".to_string(), json!(trigger));
    details.insert(
        "thresholdPolicy".to_string(),
        resolver_blind_zone_policy_summary(),
    );
    insert_optional_number(&mut details, "unresolvedInternalRatio", ratio);
    insert_optional_u64(&mut details, "unresolvedInternal", unresolved_internal);
    details.insert(
        "sourceArtifact".to_string(),
        json!(if has_diagnostic_summary {
            "resolver-diagnostics.json"
        } else {
            "symbols.json"
        }),
    );
    if has_diagnostic_summary {
        insert_existing(
            &mut details,
            "resolverVersion",
            resolver_diagnostics.and_then(|artifact| artifact.get("resolverVersion")),
        );
        for key in [
            "blindZoneCount",
            "candidateTargetCount",
            "unresolvedImportCount",
            "reasonCounts",
            "topFamilies",
            "topAffectedPackageScopes",
            "topSpecifierRoots",
        ] {
            insert_existing(
                &mut details,
                key,
                diagnostic_summary.and_then(|summary| summary.get(key)),
            );
        }
    }
    details.insert(
        "topUnresolvedSpecifiers".to_string(),
        Value::Array(
            top.iter()
                .take(3)
                .map(|item| {
                    item.get("specifierPrefix")
                        .cloned()
                        .unwrap_or_else(|| item.clone())
                })
                .collect(),
        ),
    );
    details.insert(
        "topUnresolvedReasons".to_string(),
        top_unresolved_reasons_detail(symbols, diagnostic_summary).unwrap_or(Value::Null),
    );

    Some(zone(
        "resolver",
        BlindZoneSeverity::ConfidenceGap,
        "Tier C dead-export claims must be reviewed; the resolver failed to resolve a significant fraction, count, or concentrated prefix of internal imports. See FP-36 in references/false-positive-index.md.",
        Some(Value::Object(details)),
    ))
}

fn resolver_diagnostics_summary(
    resolver_diagnostics: Option<&Value>,
) -> Option<&Map<String, Value>> {
    let artifact = resolver_diagnostics?;
    if artifact.get("status").and_then(Value::as_str) == Some("unavailable") {
        return None;
    }
    let summary = artifact.get("summary").and_then(Value::as_object)?;
    if summary.get("status").and_then(Value::as_str) == Some("unavailable") {
        return None;
    }
    Some(summary)
}

fn top_unresolved_reasons_detail(
    symbols: Option<&Value>,
    diagnostic_summary: Option<&Map<String, Value>>,
) -> Option<Value> {
    optional_array(diagnostic_summary.and_then(|summary| summary.get("topUnresolvedReasons")))
        .map(|items| Value::Array(items.to_vec()))
        .or_else(|| {
            top_unresolved_reasons_from_summary(get_path_opt(
                symbols,
                &["unresolvedInternalSummaryByReason"],
            ))
        })
        .or_else(|| {
            top_unresolved_reasons(
                optional_array(get_path_opt(
                    symbols,
                    &["unresolvedInternalSpecifierRecords"],
                ))
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            )
        })
}

fn top_unresolved_reasons(records: &[Value]) -> Option<Value> {
    let mut counts = BTreeMap::<String, u64>::new();
    for rec in records {
        let Some(reason) = rec
            .get("reason")
            .and_then(Value::as_str)
            .filter(|reason| !reason.is_empty())
        else {
            continue;
        };
        *counts.entry(reason.to_string()).or_default() += 1;
    }
    if counts.is_empty() {
        return None;
    }
    let mut reasons = counts.into_iter().collect::<Vec<_>>();
    reasons.sort_by(|(left_reason, left_count), (right_reason, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_reason.cmp(right_reason))
    });
    Some(Value::Array(
        reasons
            .into_iter()
            .take(5)
            .map(|(reason, count)| json!({ "reason": reason, "count": count }))
            .collect(),
    ))
}

fn top_unresolved_reasons_from_summary(summary: Option<&Value>) -> Option<Value> {
    let summary = summary.and_then(Value::as_object)?;
    let mut reasons = Vec::new();
    for (reason, group) in summary {
        let Some(count) = group
            .get("count")
            .and_then(number_u64)
            .filter(|count| *count > 0)
        else {
            continue;
        };
        let mut item = Map::new();
        item.insert("reason".to_string(), json!(reason));
        item.insert("count".to_string(), json!(count));
        if let Some(spaces) = compact_unresolved_spaces(group.get("spaces")) {
            item.insert("spaces".to_string(), spaces);
        }
        insert_existing(&mut item, "resolverStages", group.get("resolverStages"));
        insert_existing(&mut item, "hints", group.get("hints"));
        reasons.push(Value::Object(item));
    }
    if reasons.is_empty() {
        return None;
    }
    reasons.sort_by(|left, right| {
        number_u64(left.get("count").unwrap_or(&Value::Null))
            .unwrap_or(0)
            .cmp(&number_u64(right.get("count").unwrap_or(&Value::Null)).unwrap_or(0))
            .reverse()
            .then_with(|| {
                left.get("reason")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .cmp(right.get("reason").and_then(Value::as_str).unwrap_or(""))
            })
    });
    Some(Value::Array(reasons.into_iter().take(5).collect()))
}

fn compact_unresolved_spaces(spaces: Option<&Value>) -> Option<Value> {
    let spaces = spaces.and_then(Value::as_object)?;
    let mut compact = Map::new();
    for key in ["type", "value", "unknown"] {
        if let Some(count) = spaces.get(key).and_then(number_u64) {
            compact.insert(key.to_string(), json!(count));
        }
    }
    (!compact.is_empty()).then_some(Value::Object(compact))
}
