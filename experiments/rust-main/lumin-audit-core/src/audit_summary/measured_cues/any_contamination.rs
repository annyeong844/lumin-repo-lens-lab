use serde_json::Value;

use super::super::support::{arr, get, object, plural};

#[derive(Debug, Default)]
struct OwnerSummary {
    annotated: i64,
    severe: i64,
    any_contaminated: i64,
    severe_examples: Vec<String>,
}

#[derive(Debug, Default)]
struct AnyContaminationSummary {
    present: bool,
    supported: bool,
    helper: OwnerSummary,
    type_owner: OwnerSummary,
    annotated: i64,
}

fn summarize_any_contamination_owners(symbols: &Value) -> AnyContaminationSummary {
    if !symbols.is_object() {
        return AnyContaminationSummary::default();
    }
    let supported = symbols
        .pointer("/meta/supports/anyContamination")
        .and_then(Value::as_bool)
        == Some(true);
    let helper = summarize_owners(get(symbols, "helperOwnersByIdentity"));
    let type_owner = summarize_owners(get(symbols, "typeOwnersByIdentity"));
    AnyContaminationSummary {
        present: true,
        supported,
        annotated: helper.annotated + type_owner.annotated,
        helper,
        type_owner,
    }
}

fn summarize_owners(map: Option<&Value>) -> OwnerSummary {
    let mut rows = object(map)
        .map(|object| {
            object
                .iter()
                .filter(|(_, owner)| owner.is_object())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    rows.sort_by(|left, right| left.0.cmp(right.0));
    let mut summary = OwnerSummary::default();
    for (identity, owner) in rows {
        let Some(annotation) = get(owner, "anyContamination") else {
            continue;
        };
        summary.annotated += 1;
        if has_label(annotation, "severely-any-contaminated") {
            summary.severe += 1;
            if summary.severe_examples.len() < 3 {
                summary.severe_examples.push(identity.clone());
            }
        }
        if has_label(annotation, "any-contaminated")
            || get(annotation, "label").and_then(Value::as_str) == Some("severely-any-contaminated")
        {
            summary.any_contaminated += 1;
        }
    }
    summary
}

fn has_label(annotation: &Value, label: &str) -> bool {
    arr(get(annotation, "labels"))
        .iter()
        .any(|value| value.as_str() == Some(label))
        || get(annotation, "label").and_then(Value::as_str) == Some(label)
}

fn example_text(summary: &AnyContaminationSummary) -> String {
    let examples = summary
        .type_owner
        .severe_examples
        .iter()
        .map(|id| format!("type {id}"))
        .chain(
            summary
                .helper
                .severe_examples
                .iter()
                .map(|id| format!("helper {id}")),
        )
        .take(3)
        .collect::<Vec<_>>();
    if examples.is_empty() {
        String::new()
    } else {
        format!(" Examples: {}.", examples.join("; "))
    }
}

pub(super) fn format_any_contamination_cue(symbols: &Value) -> Option<String> {
    let summary = summarize_any_contamination_owners(symbols);
    if !summary.present {
        return None;
    }
    if !summary.supported {
        return Some("- Exported any-contamination: not measured by this symbols.json. Treat semantic reuse/shape safety claims as not enough evidence yet.".to_string());
    }
    if summary.annotated == 0 {
        return Some("- Exported any-contamination: measured; no contaminated exported owner identities observed. Read `symbols.json.helperOwnersByIdentity` and `symbols.json.typeOwnersByIdentity` before semantic reuse or shape-merge claims.".to_string());
    }
    Some(format!(
        "- Exported any-contamination: {} severe type {}, {} severe helper {} ({} any-contaminated type {}, {} helper {}). Read `symbols.json.typeOwnersByIdentity` and `symbols.json.helperOwnersByIdentity` before semantic reuse or shape-merge claims.{}",
        summary.type_owner.severe,
        plural(summary.type_owner.severe, "owner", None),
        summary.helper.severe,
        plural(summary.helper.severe, "owner", None),
        summary.type_owner.any_contaminated,
        plural(summary.type_owner.any_contaminated, "owner", None),
        summary.helper.any_contaminated,
        plural(summary.helper.any_contaminated, "owner", None),
        example_text(&summary)
    ))
}
