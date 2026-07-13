use super::*;

pub(super) fn fan_in(symbols: &Value, identity: &str) -> (Value, &'static str, String) {
    if symbols
        .pointer("/meta/supports/identityFanIn")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return (
            Value::Null,
            "unavailable",
            "[확인 불가, reason: symbols.meta.supports.identityFanIn is not true; identity fan-in not emitted by this producer]".to_string(),
        );
    }
    if let Some(value) = symbols.pointer(&format!(
        "/fanInByIdentity/{}",
        json_pointer_escape(identity)
    )) {
        return (
            value.clone(),
            "grounded",
            format!("[grounded, symbols.json.fanInByIdentity['{identity}'] = {value}]"),
        );
    }
    (
        Value::Null,
        "unavailable",
        format!("[확인 불가, reason: supports.identityFanIn=true but fanInByIdentity['{identity}'] is absent — producer contract violation. symbols.topSymbolFanIn is name-keyed and MUST NOT be substituted]"),
    )
}

pub(super) fn fan_in_space(symbols: &Value, identity: &str) -> (Value, &'static str, String) {
    if symbols
        .pointer("/meta/supports/identityFanInSpace")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return (
            Value::Null,
            "unavailable",
            "[확인 불가, reason: symbols.meta.supports.identityFanInSpace is not true; type/value fan-in breakdown not emitted by this producer]".to_string(),
        );
    }
    if let Some(record) = symbols.pointer(&format!(
        "/fanInByIdentitySpace/{}",
        json_pointer_escape(identity)
    )) {
        let normalized = json!({
            "value": record.get("value").and_then(Value::as_u64).unwrap_or(0),
            "type": record.get("type").and_then(Value::as_u64).unwrap_or(0),
            "broad": record.get("broad").and_then(Value::as_u64).unwrap_or(0),
        });
        return (
            normalized.clone(),
            "grounded",
            format!("[grounded, symbols.json.fanInByIdentitySpace['{identity}'] = {normalized}]"),
        );
    }
    (
        Value::Null,
        "unavailable",
        format!("[확인 불가, reason: supports.identityFanInSpace=true but fanInByIdentitySpace['{identity}'] is absent — producer contract violation]"),
    )
}

pub(super) fn contamination(definition: &Value, supports: &Value) -> (Value, String) {
    if supports.get("anyContamination").and_then(Value::as_bool) != Some(true) {
        return (
            json!({ "state": "capability-absent" }),
            "[확인 불가, reason: producer did not emit anyContamination capability (symbols.meta.supports.anyContamination !== true)]".to_string(),
        );
    }
    let Some(annotation) = definition.get("anyContamination") else {
        return (
            json!({ "state": "clean" }),
            "[grounded, anyContamination annotation absent → clean]".to_string(),
        );
    };
    let labels = annotation
        .get("labels")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let has = |label: &str| labels.iter().any(|value| value.as_str() == Some(label));
    let state = if has("severely-any-contaminated") {
        "severely-any-contaminated"
    } else if has("any-contaminated") {
        "any-contaminated"
    } else if has("has-any") {
        "has-any-only"
    } else if has("unknown-surface") {
        "unknown-surface-only"
    } else {
        "clean"
    };
    let mut result = json!({
        "state": state,
        "labels": labels,
        "measurements": annotation.get("measurements").cloned().unwrap_or(Value::Null),
    });
    if matches!(state, "severely-any-contaminated" | "any-contaminated") {
        result["recommendation"] = json!({
            "action": "warn-on-reuse",
            "confidence": "low",
            "reason": format!("{state} semantic reuse caution"),
        });
    }
    let citation = if state == "clean" {
        format!("[확인 불가, reason: anyContamination annotation present but labels[] empty or unrecognized: {}]", Value::Array(labels))
    } else {
        format!(
            "[grounded, anyContamination.label = '{state}', measurements = {}]",
            annotation
                .get("measurements")
                .cloned()
                .unwrap_or_else(|| json!({}))
        )
    };
    (result, citation)
}

pub(super) fn resolver_confidence(
    owner_file: &str,
    symbols: &Value,
) -> (&'static str, Option<String>) {
    if symbols
        .get("filesWithParseErrors")
        .and_then(Value::as_array)
        .is_some_and(|files| files.iter().any(|file| file.as_str() == Some(owner_file)))
    {
        return (
            "low",
            Some(format!("[degraded, resolver-confidence: low, taints: [\"defining-file-parse-error: '{owner_file}'\"]]")),
        );
    }
    let matching = symbols
        .get("unresolvedInternalSpecifiers")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .find(|specifier| specifier_could_match_file(specifier, owner_file));
    match matching {
        Some(specifier) => (
            "medium",
            Some(format!("[degraded, resolver-confidence: medium, taints: [\"unresolved-specifier-could-match: '{specifier}' ↔ '{owner_file}'\"]]")),
        ),
        None => ("high", None),
    }
}

fn specifier_could_match_file(specifier: &str, owner_file: &str) -> bool {
    if !specifier.starts_with('.') {
        return false;
    }
    let spec = specifier
        .trim_start_matches("./")
        .trim_end_matches(['s', 'x', 'j', 't', 'm', 'c', '.']);
    let owner = owner_file.replace('\\', "/");
    owner.contains(spec) || owner.ends_with(&format!("/{spec}"))
}
