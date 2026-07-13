use super::{FunctionSizeEntry, FunctionSizeFacts};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub(super) fn function_size(facts: &FunctionSizeFacts) -> Value {
    let mut buckets = RoleBucket::default();
    let mut role_buckets = BTreeMap::<String, RoleBucket>::from([
        ("production".to_string(), RoleBucket::default()),
        ("test".to_string(), RoleBucket::default()),
        ("script".to_string(), RoleBucket::default()),
    ]);
    let mut all_loc = Vec::<usize>::new();
    let mut oversized = Vec::<(usize, FunctionSizeEntry)>::new();
    let mut watch = Vec::<(usize, FunctionSizeEntry)>::new();

    for (index, entry) in facts.entries.iter().cloned().enumerate() {
        let loc = entry.loc.max(1);
        all_loc.push(loc);
        let role = normalized_role(&entry.file_role);
        let role_bucket = role_buckets.entry(role).or_default();
        role_bucket.total += 1;
        if loc > 150 {
            buckets.big += 1;
            role_bucket.big += 1;
            oversized.push((index, entry));
        } else if loc > 100 {
            buckets.medium += 1;
            role_bucket.medium += 1;
            watch.push((index, entry));
        } else {
            buckets.small += 1;
            role_bucket.small += 1;
        }
    }

    all_loc.sort_unstable();
    let p95 = if all_loc.is_empty() {
        0
    } else {
        all_loc[all_loc.len() * 95 / 100]
    };
    let sort_by_loc = |items: &mut Vec<(usize, FunctionSizeEntry)>| {
        items.sort_by(|(left_index, left), (right_index, right)| {
            right
                .loc
                .cmp(&left.loc)
                .then_with(|| left_index.cmp(right_index))
        });
    };
    sort_by_loc(&mut oversized);
    sort_by_loc(&mut watch);

    let gate = if buckets.big >= 3 {
        "fix"
    } else if buckets.big >= 1 {
        "watch"
    } else {
        "ok"
    };

    json!({
        "gate": gate,
        "buckets": buckets.to_value(),
        "roleBuckets": role_buckets_to_value(&role_buckets),
        "p95Loc": p95,
        "total": all_loc.len(),
        "parseErrors": facts.parse_errors,
        "oversized": entries_to_values(&oversized, 20),
        "watch": entries_to_values(&watch, 20),
        "oversizedByRole": entries_by_role(&oversized),
        "watchByRole": entries_by_role(&watch),
    })
}

#[derive(Debug, Clone, Copy, Default)]
struct RoleBucket {
    big: usize,
    medium: usize,
    small: usize,
    total: usize,
}

impl RoleBucket {
    fn to_value(self) -> Value {
        json!({
            "big": self.big,
            "medium": self.medium,
            "small": self.small,
        })
    }

    fn to_role_value(self) -> Value {
        json!({
            "big": self.big,
            "medium": self.medium,
            "small": self.small,
            "total": self.total,
        })
    }
}

fn normalized_role(role: &str) -> String {
    match role {
        "test" | "script" | "production" => role.to_string(),
        _ => "production".to_string(),
    }
}

fn role_buckets_to_value(role_buckets: &BTreeMap<String, RoleBucket>) -> Value {
    let mut object = Map::new();
    for role in ["production", "test", "script"] {
        object.insert(
            role.to_string(),
            role_buckets
                .get(role)
                .copied()
                .unwrap_or_default()
                .to_role_value(),
        );
    }
    Value::Object(object)
}

fn function_entry_value(entry: &FunctionSizeEntry) -> Value {
    json!({
        "file": entry.file,
        "line": entry.line,
        "name": entry.name,
        "loc": entry.loc.max(1),
        "fileRole": normalized_role(&entry.file_role),
    })
}

fn entries_to_values(items: &[(usize, FunctionSizeEntry)], limit: usize) -> Vec<Value> {
    items
        .iter()
        .take(limit)
        .map(|(_, entry)| function_entry_value(entry))
        .collect()
}

fn entries_by_role(items: &[(usize, FunctionSizeEntry)]) -> Value {
    let mut object = Map::new();
    for role in ["production", "test", "script"] {
        object.insert(
            role.to_string(),
            Value::Array(
                items
                    .iter()
                    .filter(|(_, entry)| normalized_role(&entry.file_role) == role)
                    .take(10)
                    .map(|(_, entry)| function_entry_value(entry))
                    .collect(),
            ),
        );
    }
    Value::Object(object)
}
