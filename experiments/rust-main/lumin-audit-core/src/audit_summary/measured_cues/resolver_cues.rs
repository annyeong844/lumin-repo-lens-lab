use serde_json::Value;

use super::super::support::{
    arr, format_counter_object, format_unresolved_reason_counts, get, n_or,
};

pub(super) fn format_top_unresolved_roots(roots: Option<&Value>, limit: usize) -> Option<String> {
    let parts = arr(roots)
        .iter()
        .take(limit)
        .filter_map(|root| {
            let name = get(root, "specifierRoot").and_then(Value::as_str)?;
            let count = n_or(get(root, "count"), i64::MIN);
            if count == i64::MIN {
                return None;
            }
            let reasons = format_unresolved_reason_counts(get(root, "reasons"), 3);
            Some(format!(
                "{name} {count}{}",
                reasons
                    .map(|reasons| format!(" ({reasons})"))
                    .unwrap_or_default()
            ))
        })
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("; "))
}

pub(super) fn format_top_affected_package_scopes(
    scopes: Option<&Value>,
    limit: usize,
) -> Option<String> {
    let parts = arr(scopes)
        .iter()
        .take(limit)
        .filter_map(|scope| {
            let name = get(scope, "affectedPackageScope").and_then(Value::as_str)?;
            let count = n_or(get(scope, "count"), i64::MIN);
            (count != i64::MIN).then(|| format!("{name} {count}"))
        })
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("; "))
}

pub(super) fn format_blocked_candidate_hints(
    hints: Option<&Value>,
    limit: usize,
) -> Option<String> {
    let parts = arr(hints)
        .iter()
        .take(limit)
        .filter_map(|hint| {
            let target = get(hint, "candidatePath")
                .or_else(|| get(hint, "affectedPackageScope"))
                .and_then(Value::as_str)?;
            let specifier = get(hint, "specifier").and_then(Value::as_str)?;
            let reason = get(hint, "reason").and_then(Value::as_str)?;
            Some(format!("{target} via {specifier} ({reason})"))
        })
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("; "))
}

fn format_distribution_list(
    items: Option<&Value>,
    label_key: &str,
    nested_key: &str,
    limit: usize,
) -> Option<String> {
    let parts = arr(items)
        .iter()
        .take(limit)
        .filter_map(|item| {
            let label = get(item, label_key).and_then(Value::as_str)?;
            let count = n_or(get(item, "count"), i64::MIN);
            if count == i64::MIN {
                return None;
            }
            let nested = format_counter_object(get(item, nested_key));
            Some(format!(
                "{label} {count}{}",
                nested.map(|text| format!(" ({text})")).unwrap_or_default()
            ))
        })
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join(", "))
}

pub(super) fn format_blocked_candidate_hint_distribution(
    resolver_diagnostics: &Value,
) -> Option<String> {
    let reason_text = format_distribution_list(
        get(resolver_diagnostics, "blockedCandidateHintReasonCounts"),
        "reason",
        "families",
        3,
    );
    let family_text = format_distribution_list(
        get(resolver_diagnostics, "blockedCandidateHintFamilyCounts"),
        "family",
        "reasons",
        3,
    );
    let parts = [
        reason_text.map(|text| format!("reasons {text}")),
        family_text.map(|text| format!("families {text}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("; "))
}
