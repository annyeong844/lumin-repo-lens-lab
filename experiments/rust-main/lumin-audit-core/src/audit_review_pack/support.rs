use serde_json::Value;

pub(super) fn value_to_string(value: Option<&Value>, fallback: &str) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        _ => fallback.to_string(),
    }
}

pub(super) fn n(value: Option<&Value>) -> i64 {
    number_as_i64(value).unwrap_or(0)
}

pub(super) fn n_or(value: Option<&Value>, fallback: i64) -> i64 {
    number_as_i64(value).unwrap_or(fallback)
}

pub(super) fn number_as_i64(value: Option<&Value>) -> Option<i64> {
    match value {
        Some(Value::Number(number)) => Some(
            number
                .as_i64()
                .or_else(|| number.as_u64().and_then(|value| i64::try_from(value).ok()))
                .or_else(|| number.as_f64().map(|value| value as i64))
                .unwrap_or(0),
        ),
        _ => None,
    }
}

pub(super) fn arr(value: Option<&Value>) -> &[Value] {
    value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

pub(super) fn object(value: &Value) -> Option<&serde_json::Map<String, Value>> {
    value.as_object()
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

pub(super) fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

pub(super) fn format_counter_object(counter: Option<&Value>) -> Option<String> {
    let object = counter?.as_object()?;
    let mut parts = object
        .iter()
        .filter_map(|(label, count)| Some((label.as_str(), number_as_i64(Some(count))?)))
        .collect::<Vec<_>>();
    parts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    if parts.is_empty() {
        None
    } else {
        Some(
            parts
                .into_iter()
                .map(|(label, count)| format!("{label} {count}"))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

pub(super) fn format_rust_scope(summary: &Value) -> String {
    let Some(scope) = get(summary, "scanScope").filter(|value| value.is_object()) else {
        return String::new();
    };
    let tests = if get(scope, "includeTests").and_then(Value::as_bool) == Some(false) {
        "production files only"
    } else {
        "including tests"
    };
    let exclude_count = arr(get(scope, "exclude")).len();
    let excludes = if exclude_count > 0 {
        format!(
            ", {exclude_count} exclude {}",
            if exclude_count == 1 {
                "pattern"
            } else {
                "patterns"
            }
        )
    } else {
        String::new()
    };
    format!(" ({tests}{excludes})")
}

pub(super) fn scan_range(manifest: &Value) -> String {
    let empty = Value::Object(Default::default());
    let scan_range = get(manifest, "scanRange").unwrap_or(&empty);
    let langs = {
        let values = arr(get(scan_range, "languages"))
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>();
        if values.is_empty() {
            "unknown".to_string()
        } else {
            values.join(", ")
        }
    };
    let tests = if get(scan_range, "includeTests").and_then(Value::as_bool) == Some(false) {
        "production only"
    } else {
        "includes tests"
    };
    format!(
        "{} files; {langs}; {tests}",
        value_to_string(get(scan_range, "files"), "unknown")
    )
}

pub(super) fn lane(title: &str, body: String) -> String {
    format!("## {title}\n\n{}\n", body.trim())
}

pub(super) fn render_lane_prompt(
    title: &str,
    mission: String,
    artifacts: Vec<String>,
    checks: Vec<String>,
    report: &str,
) -> String {
    let mut lines = vec![
        "Controller-only lane. Read this in the main context as an artifact brief; do not paste the lane wholesale into a subagent.".to_string(),
        String::new(),
        format!("Role: {title}"),
        String::new(),
        format!("Mission: {mission}"),
        String::new(),
        format!("Artifacts for the controller to inspect first: {}", artifacts.join(", ")),
        String::new(),
        "Checks to convert into code questions:".to_string(),
    ];
    lines.extend(checks.into_iter().map(|check| format!("- {check}")));
    lines.extend([
        String::new(),
        format!("Report back with: {report}"),
        String::new(),
        "Subagent rule: if you dispatch a reviewer subagent, give it specific files, symbols, or hypotheses from this lane and ask it to read the codebase with file:line evidence. Do not ask the subagent to trust checklist or artifact summaries.".to_string(),
        String::new(),
        "Rules: cite artifact fields or file:line evidence; do not turn a gate value into a verdict; mark unknowns as \"not enough evidence yet\"; keep recommendations to the smallest useful slice.".to_string(),
    ]);
    lines.join("\n")
}
