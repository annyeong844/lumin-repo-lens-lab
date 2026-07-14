use serde_json::Value;

pub(super) fn value_to_string(value: Option<&Value>, fallback: &str) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        _ => fallback.to_string(),
    }
}

pub(super) fn pointer_string(value: &Value, pointer: &str, fallback: &str) -> String {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}

pub(super) fn n(value: Option<&Value>) -> i64 {
    n_or(value, 0)
}

pub(super) fn n_or(value: Option<&Value>, fallback: i64) -> i64 {
    match value {
        Some(Value::Number(number)) => number
            .as_i64()
            .or_else(|| number.as_u64().and_then(|value| i64::try_from(value).ok()))
            .or_else(|| number.as_f64().map(|value| value as i64))
            .unwrap_or(fallback),
        _ => fallback,
    }
}

fn f64_value(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn pct(value: Option<&Value>) -> String {
    let Some(value) = f64_value(value) else {
        return "unknown".to_string();
    };
    if value < 0.01 {
        format!("{:.2}%", value * 100.0)
    } else {
        format!("{:.1}%", value * 100.0)
    }
}

pub(super) fn arr(value: Option<&Value>) -> &[Value] {
    value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

pub(super) fn object(value: Option<&Value>) -> Option<&serde_json::Map<String, Value>> {
    value?.as_object()
}

pub(super) fn get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_object().and_then(|object| object.get(key))
}

pub(super) fn plural(count: i64, singular: &str, plural_value: Option<&str>) -> String {
    if count == 1 {
        singular.to_string()
    } else {
        plural_value.unwrap_or(&format!("{singular}s")).to_string()
    }
}

pub(super) fn format_counter_object(counter: Option<&Value>) -> Option<String> {
    let mut parts = object(counter)?
        .iter()
        .map(|(label, count)| (label.as_str(), n_or(Some(count), i64::MIN)))
        .filter(|(_, count)| *count != i64::MIN)
        .collect::<Vec<_>>();
    parts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    (!parts.is_empty()).then(|| {
        parts
            .into_iter()
            .map(|(label, count)| format!("{label} {count}"))
            .collect::<Vec<_>>()
            .join(", ")
    })
}

pub(super) fn format_unresolved_reason_counts(
    reasons: Option<&Value>,
    limit: usize,
) -> Option<String> {
    let mut items = match reasons {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| {
                Some((
                    get(item, "reason")?.as_str()?.to_string(),
                    n_or(get(item, "count"), i64::MIN),
                ))
            })
            .filter(|(_, count)| *count != i64::MIN)
            .collect::<Vec<_>>(),
        Some(Value::Object(object)) => object
            .iter()
            .map(|(reason, count)| (reason.clone(), n_or(Some(count), i64::MIN)))
            .filter(|(_, count)| *count != i64::MIN)
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    if matches!(reasons, Some(Value::Object(_))) {
        items.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    }
    let parts = items
        .into_iter()
        .take(limit)
        .filter(|(reason, _)| !reason.is_empty())
        .map(|(reason, count)| format!("{reason} {count}"))
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join(", "))
}

pub(super) fn base_evidence_not_refreshed(manifest: &Value) -> bool {
    manifest
        .pointer("/baseEvidence/status")
        .and_then(Value::as_str)
        == Some("not-refreshed")
}

pub(super) fn summarize_scan_range(manifest: &Value) -> String {
    if base_evidence_not_refreshed(manifest) {
        return "base audit not refreshed (lifecycle-only); use lifecycle evidence for this command"
            .to_string();
    }
    let empty = Value::Object(Default::default());
    let scan_range = get(manifest, "scanRange").unwrap_or(&empty);
    let langs = arr(get(scan_range, "languages"))
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let langs = if langs.is_empty() {
        "unknown".to_string()
    } else {
        langs.join(", ")
    };
    let tests = if get(scan_range, "includeTests").and_then(Value::as_bool) == Some(false) {
        "production files only"
    } else {
        "including tests"
    };
    let excludes = arr(get(scan_range, "excludes"));
    let exclude_text = if excludes.is_empty() {
        String::new()
    } else {
        format!(
            "; excludes: {}",
            excludes
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    format!(
        "{} files, {langs}, {tests}{exclude_text}",
        value_to_string(get(scan_range, "files"), "unknown")
    )
}

pub(super) fn summarize_confidence(manifest: &Value) -> String {
    if base_evidence_not_refreshed(manifest) {
        return "base audit not evaluated; lifecycle evidence status is independent".to_string();
    }
    let confidence = get(manifest, "confidence").unwrap_or(&Value::Null);
    let blind_count = arr(get(manifest, "blindZones"))
        .iter()
        .filter(|zone| get(zone, "area").and_then(Value::as_str) != Some("base-audit"))
        .count();
    format!(
        "parse errors {}, unresolved internal {}, blind zones {blind_count}",
        value_to_string(get(confidence, "parseErrors"), "unknown"),
        pct(get(confidence, "unresolvedInternalRatio"))
    )
}
