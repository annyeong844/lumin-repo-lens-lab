use super::facts::{bool_field, f64_field, string_field, string_value, usize_field, FunctionFact};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::BTreeSet;

pub(super) fn sort_diagnostics(mut diagnostics: Vec<Value>) -> Vec<Value> {
    diagnostics.sort_by(|left, right| {
        string_field(left, "file")
            .cmp(&string_field(right, "file"))
            .then_with(|| string_field(left, "code").cmp(&string_field(right, "code")))
            .then_with(|| string_value(left.get("line")).cmp(&string_value(right.get("line"))))
            .then_with(|| string_field(left, "message").cmp(&string_field(right, "message")))
    });
    diagnostics
}

pub(super) fn sort_clone_groups(groups: &mut [Value], body_loc_tiebreak: bool) {
    groups.sort_by(|left, right| {
        generated_rank(right)
            .cmp(&generated_rank(left))
            .then_with(|| usize_field(right, "size").cmp(&usize_field(left, "size")))
            .then_with(|| {
                if body_loc_tiebreak {
                    body_loc_range_max(right).cmp(&body_loc_range_max(left))
                } else {
                    Ordering::Equal
                }
            })
            .then_with(|| identities_join(left).cmp(&identities_join(right)))
    });
}

pub(super) fn sort_near_candidates(candidates: &mut [Value]) {
    candidates.sort_by(|left, right| {
        generated_rank(right)
            .cmp(&generated_rank(left))
            .then_with(|| f64_field(right, "score").total_cmp(&f64_field(left, "score")))
            .then_with(|| identities_join(left).cmp(&identities_join(right)))
    });
}

fn generated_rank(value: &Value) -> usize {
    if bool_field(value, "generatedOnly") {
        0
    } else {
        1
    }
}

fn body_loc_range_max(value: &Value) -> i64 {
    value
        .get("bodyLocRange")
        .and_then(Value::as_array)
        .and_then(|values| values.get(1))
        .and_then(Value::as_i64)
        .unwrap_or(0)
}

fn identities_join(value: &Value) -> String {
    value
        .get("identities")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("|")
        })
        .unwrap_or_default()
}

pub(super) fn non_generated_count(groups: &[Value]) -> usize {
    groups
        .iter()
        .filter(|group| !bool_field(group, "generatedOnly"))
        .count()
}

pub(super) fn line_json(fact: &FunctionFact) -> Value {
    json!({
        "identity": fact.identity,
        "file": fact.owner_file,
        "line": fact.line,
    })
}

pub(super) fn sorted_unique(values: impl Iterator<Item = String>) -> Vec<String> {
    values.collect::<BTreeSet<_>>().into_iter().collect()
}

pub(super) fn shared_call_tokens(sorted: &[&FunctionFact]) -> Vec<String> {
    if sorted.is_empty() {
        return Vec::new();
    }
    let mut shared = sorted[0]
        .call_tokens
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for fact in sorted.iter().skip(1) {
        let set = fact.call_tokens.iter().cloned().collect::<BTreeSet<_>>();
        shared = shared.intersection(&set).cloned().collect();
    }
    shared.into_iter().collect()
}
