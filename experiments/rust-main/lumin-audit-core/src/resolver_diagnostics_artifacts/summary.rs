use super::classification::unresolved_specifier_root;
use super::protocol::Record;
use super::value_support::{compact_object, value_string, value_usize};
use super::UNKNOWN_INTERNAL_RESOLUTION;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub(super) fn top_families(unresolved_imports: &[Value], blind_zones: &[Value]) -> Vec<Value> {
    let values = unresolved_imports
        .iter()
        .chain(blind_zones.iter())
        .cloned()
        .collect::<Vec<_>>();
    count_by(&values, |record| {
        record
            .str("family")
            .filter(|family| !family.is_empty())
            .map(ToOwned::to_owned)
    })
    .into_iter()
    .take(20)
    .map(|(family, count)| json!({ "family": family, "count": count }))
    .collect()
}

pub(super) fn top_affected_package_scopes(blind_zones: &[Value]) -> Vec<Value> {
    count_by(blind_zones, |record| {
        record
            .str("affectedPackageScope")
            .filter(|scope| !scope.is_empty())
            .map(ToOwned::to_owned)
    })
    .into_iter()
    .take(20)
    .map(|(affected_package_scope, count)| {
        json!({ "affectedPackageScope": affected_package_scope, "count": count })
    })
    .collect()
}

pub(super) fn top_unresolved_reasons(records: &[Value]) -> Vec<Value> {
    count_by(records, |record| {
        Some(
            record
                .str("reason")
                .unwrap_or(UNKNOWN_INTERNAL_RESOLUTION)
                .to_string(),
        )
    })
    .into_iter()
    .take(20)
    .map(|(reason, count)| json!({ "reason": reason, "count": count }))
    .collect()
}

pub(super) fn top_specifier_roots(records: &[Value]) -> Vec<Value> {
    let mut groups = BTreeMap::<String, SpecifierRootGroup>::new();
    for value in records {
        let record = Record::new(value);
        let Some(specifier_root) = record.str("specifier").and_then(unresolved_specifier_root)
        else {
            continue;
        };
        let group = groups
            .entry(specifier_root.clone())
            .or_insert_with(|| SpecifierRootGroup::new(specifier_root));
        let reason = record.str("reason").unwrap_or(UNKNOWN_INTERNAL_RESOLUTION);
        group.count += 1;
        *group.reasons.entry(reason.to_string()).or_default() += 1;
        group.examples.push(compact_object(vec![
            ("specifier", record.get("specifier").cloned()),
            (
                "consumerFile",
                record
                    .get("consumerFile")
                    .or_else(|| record.get("fromHint"))
                    .cloned()
                    .or(Some(Value::Null)),
            ),
        ]));
    }
    let mut out = groups
        .into_values()
        .map(SpecifierRootGroup::finish)
        .collect::<Vec<_>>();
    out.sort_by(|left, right| {
        value_usize(right, "count")
            .cmp(&value_usize(left, "count"))
            .then_with(|| {
                value_string(left, "specifierRoot").cmp(&value_string(right, "specifierRoot"))
            })
    });
    out.truncate(20);
    out
}

pub(super) fn counter_object_from_values(
    records: &[Value],
    key_fn: impl Fn(Record<'_>) -> String,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in records {
        let key = key_fn(Record::new(value));
        if key.is_empty() {
            continue;
        }
        *counts.entry(key).or_default() += 1;
    }
    counts
}

fn count_by(
    values: &[Value],
    key_fn: impl Fn(Record<'_>) -> Option<String>,
) -> Vec<(String, usize)> {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in values {
        let Some(key) = key_fn(Record::new(value)) else {
            continue;
        };
        if key.is_empty() {
            continue;
        }
        *counts.entry(key).or_default() += 1;
    }
    let mut out = counts.into_iter().collect::<Vec<_>>();
    out.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    out
}

struct SpecifierRootGroup {
    specifier_root: String,
    count: usize,
    reasons: BTreeMap<String, usize>,
    examples: Vec<Value>,
}

impl SpecifierRootGroup {
    fn new(specifier_root: String) -> Self {
        Self {
            specifier_root,
            count: 0,
            reasons: BTreeMap::new(),
            examples: Vec::new(),
        }
    }

    fn finish(mut self) -> Value {
        self.examples.sort_by_key(|example| {
            format!(
                "{}|{}",
                example
                    .get("consumerFile")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                example
                    .get("specifier")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
            )
        });
        self.examples.truncate(5);
        json!({
            "specifierRoot": self.specifier_root,
            "count": self.count,
            "reasons": self.reasons,
            "examples": self.examples,
        })
    }
}
