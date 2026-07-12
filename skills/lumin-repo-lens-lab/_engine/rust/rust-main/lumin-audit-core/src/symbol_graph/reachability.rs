use super::evidence::{is_absolute_like_path, normalize_path_segments, rel_path, value_string};
use super::normalize_slashes;
use super::prepare::{DefinitionFile, FileDataRecord};
use super::protocol::{
    DeadCandidateInputs, FanInConsumerEntry, FanInInputs, FanInNamespaceUserEntry,
};
use crate::source_use_assembly::SourceUseAssemblyResponse;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn top_symbol_fan_in(mut values: Vec<Value>) -> Vec<Value> {
    values.sort_by(|left, right| {
        let left_count = left.get("count").and_then(Value::as_i64).unwrap_or(0);
        let right_count = right.get("count").and_then(Value::as_i64).unwrap_or(0);
        right_count.cmp(&left_count)
    });
    values.truncate(50);
    values
}

pub(super) struct ComputedFanIn {
    pub(super) symbol_fan_in: Vec<Value>,
    pub(super) fan_in_by_identity: Map<String, Value>,
    pub(super) fan_in_by_identity_space: Map<String, Value>,
}

#[derive(Default)]
struct DirectFanIn {
    all: BTreeSet<String>,
    value: BTreeSet<String>,
    type_only: BTreeSet<String>,
}

pub(super) struct ComputedDeadCandidates {
    pub(super) dead: Vec<Value>,
    pub(super) truly_dead: Vec<Value>,
    pub(super) dead_in_prod: Vec<Value>,
    pub(super) dead_in_test: Vec<Value>,
}

pub(super) fn merge_source_use_fan_in_inputs(
    root: &str,
    base: FanInInputs,
    source_use_assembly: &SourceUseAssemblyResponse,
) -> FanInInputs {
    let FanInInputs {
        mut consumer_entries,
        mut namespace_user_entries,
    } = base;

    for direct in &source_use_assembly.direct_consumers {
        consumer_entries.push(FanInConsumerEntry {
            def_file: request_path_from_response_path(root, &direct.def_file),
            symbol: direct.symbol.clone(),
            consumer_file: request_path_from_response_path(root, &direct.consumer_file),
            space: Some(direct.space.to_string()),
        });
    }
    for namespace_user in &source_use_assembly.namespace_users {
        namespace_user_entries.push(FanInNamespaceUserEntry {
            def_file: request_path_from_response_path(root, &namespace_user.def_file),
            consumer_file: request_path_from_response_path(root, &namespace_user.consumer_file),
        });
    }

    FanInInputs {
        consumer_entries,
        namespace_user_entries,
    }
}

fn request_path_from_response_path(root: &str, path: &str) -> String {
    let normalized = normalize_slashes(path);
    if normalized == "." || normalized.is_empty() {
        return normalize_slashes(root).trim_end_matches('/').to_string();
    }
    if is_absolute_like_path(&normalized) {
        return normalized;
    }
    let root = normalize_slashes(root).trim_end_matches('/').to_string();
    normalize_path_segments(&format!("{root}/{normalized}"))
}

pub(super) fn build_fan_in(
    root: &str,
    def_index: &[DefinitionFile],
    inputs: &FanInInputs,
) -> ComputedFanIn {
    let mut def_kind_by_key = BTreeMap::<(String, String), String>::new();
    let mut fan_in_by_identity = Map::new();
    let mut fan_in_by_identity_space = Map::new();

    for file in def_index {
        let rel_file = rel_path(root, &file.file_path);
        for (symbol, definition) in &file.definitions {
            let identity = format!("{rel_file}::{symbol}");
            fan_in_by_identity.insert(identity.clone(), json!(0));
            fan_in_by_identity_space.insert(
                identity,
                json!({
                    "value": 0,
                    "type": 0,
                    "broad": 0,
                }),
            );
            def_kind_by_key.insert(
                (rel_file.clone(), symbol.clone()),
                value_string(definition, "kind"),
            );
        }
    }

    let mut direct = BTreeMap::<(String, String), DirectFanIn>::new();
    let mut direct_order = Vec::<(String, String)>::new();
    let mut direct_seen = BTreeSet::<(String, String)>::new();
    for entry in &inputs.consumer_entries {
        let key = (rel_path(root, &entry.def_file), entry.symbol.clone());
        if direct_seen.insert(key.clone()) {
            direct_order.push(key.clone());
        }
        let fan_in = direct.entry(key).or_default();
        let consumer_file = rel_path(root, &entry.consumer_file);
        fan_in.all.insert(consumer_file.clone());
        if entry.space.as_deref() == Some("type") {
            fan_in.type_only.insert(consumer_file);
        } else {
            fan_in.value.insert(consumer_file);
        }
    }

    let mut namespace_users = BTreeMap::<String, BTreeSet<String>>::new();
    for entry in &inputs.namespace_user_entries {
        namespace_users
            .entry(rel_path(root, &entry.def_file))
            .or_default()
            .insert(rel_path(root, &entry.consumer_file));
    }

    let mut symbol_fan_in = Vec::new();
    for (def_file, symbol) in direct_order {
        let key = (def_file.clone(), symbol.clone());
        let Some(fan_in) = direct.get(&key) else {
            continue;
        };
        let identity = format!("{def_file}::{symbol}");
        let count = fan_in.all.len();
        symbol_fan_in.push(json!({
            "defFile": def_file,
            "symbol": symbol,
            "count": count,
            "kind": def_kind_by_key
                .get(&(def_file.clone(), symbol.clone()))
                .filter(|kind| !kind.is_empty())
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
        }));
        fan_in_by_identity.insert(identity.clone(), json!(count));
        fan_in_by_identity_space.insert(
            identity,
            json!({
                "value": fan_in.value.len(),
                "type": fan_in.type_only.len(),
                "broad": namespace_users.get(&def_file).map(BTreeSet::len).unwrap_or(0),
            }),
        );
    }

    for file in def_index {
        let file_path = rel_path(root, &file.file_path);
        let Some(broad_consumers) = namespace_users.get(&file_path) else {
            continue;
        };
        for symbol in file.definitions.keys() {
            let identity = format!("{file_path}::{symbol}");
            let mut existing = fan_in_by_identity_space
                .get(&identity)
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_else(|| {
                    let mut object = Map::new();
                    object.insert("value".to_string(), json!(0));
                    object.insert("type".to_string(), json!(0));
                    object.insert("broad".to_string(), json!(0));
                    object
                });
            existing.insert("broad".to_string(), json!(broad_consumers.len()));
            fan_in_by_identity_space.insert(identity, Value::Object(existing));
        }
    }

    ComputedFanIn {
        symbol_fan_in,
        fan_in_by_identity,
        fan_in_by_identity_space,
    }
}

pub(super) fn build_dead_candidates(
    root: &str,
    def_index: &[DefinitionFile],
    file_data: &[FileDataRecord],
    fan_in_inputs: &FanInInputs,
    inputs: &DeadCandidateInputs,
) -> ComputedDeadCandidates {
    let barrel_files = inputs
        .barrel_files
        .iter()
        .map(|file| rel_path(root, file))
        .collect::<BTreeSet<_>>();
    let test_like_files = inputs
        .test_like_files
        .iter()
        .map(|file| rel_path(root, file))
        .collect::<BTreeSet<_>>();
    let direct_consumers = fan_in_inputs
        .consumer_entries
        .iter()
        .map(|entry| (rel_path(root, &entry.def_file), entry.symbol.clone()))
        .collect::<BTreeSet<_>>();
    let namespace_files = fan_in_inputs
        .namespace_user_entries
        .iter()
        .map(|entry| rel_path(root, &entry.def_file))
        .collect::<BTreeSet<_>>();
    let file_data_by_path = file_data
        .iter()
        .map(|file| (rel_path(root, &file.file_path), file))
        .collect::<BTreeMap<_, _>>();

    let mut dead = Vec::new();
    for file in def_index {
        let file_path = rel_path(root, &file.file_path);
        if barrel_files.contains(&file_path) {
            continue;
        }
        let file_namespace_used = namespace_files.contains(&file_path);
        let file_info = file_data_by_path.get(&file_path).copied();
        let public_set = file_info
            .and_then(|info| info.py_dunder_all.as_ref())
            .map(|items| items.iter().cloned().collect::<BTreeSet<_>>());
        let rel_file = file_path.clone();

        for (symbol, definition) in &file.definitions {
            if direct_consumers.contains(&(file_path.clone(), symbol.clone())) {
                continue;
            }
            if public_set
                .as_ref()
                .is_some_and(|symbols| !symbols.contains(symbol))
            {
                continue;
            }
            if definition
                .get("frameworkRegistered")
                .and_then(Value::as_bool)
                == Some(true)
            {
                continue;
            }

            let mut candidate = Map::new();
            candidate.insert("file".to_string(), json!(rel_file));
            candidate.insert("symbol".to_string(), json!(symbol));
            candidate.insert(
                "kind".to_string(),
                definition
                    .get("kind")
                    .cloned()
                    .unwrap_or_else(|| json!("unknown")),
            );
            candidate.insert(
                "line".to_string(),
                definition.get("line").cloned().unwrap_or(Value::Null),
            );
            if let Some(local_name) = definition.get("localName") {
                candidate.insert("localName".to_string(), local_name.clone());
            }
            candidate.insert("namespaceShadowed".to_string(), json!(file_namespace_used));
            dead.push(Value::Object(candidate));
        }
    }

    let mut truly_dead = Vec::new();
    let mut dead_in_prod = Vec::new();
    let mut dead_in_test = Vec::new();
    for candidate in &dead {
        if candidate
            .get("namespaceShadowed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            continue;
        }
        truly_dead.push(candidate.clone());
        let file = value_string(candidate, "file");
        if test_like_files.contains(&file) {
            dead_in_test.push(candidate.clone());
        } else {
            dead_in_prod.push(candidate.clone());
        }
    }

    ComputedDeadCandidates {
        dead,
        truly_dead,
        dead_in_prod,
        dead_in_test,
    }
}
