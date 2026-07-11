use super::{rel_path, value_string, DefinitionFile, FileDataRecord};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
struct AnyOwnerRow {
    identity: String,
    name: String,
    file: String,
    kind: String,
    line: Option<i64>,
}

#[derive(Debug)]
pub(super) struct ComputedAnyContamination {
    pub(super) helper_owners_by_identity: Value,
    pub(super) type_owners_by_identity: Value,
    pub(super) def_index: Value,
}

pub(super) fn build_any_contamination_facts(
    root: &str,
    def_index: &[DefinitionFile],
    file_data: &[FileDataRecord],
) -> ComputedAnyContamination {
    let (identity_to_row, defs_by_file) = build_any_owner_lookups(root, def_index);
    let mut facts_by_identity = BTreeMap::<String, Vec<Value>>::new();

    for file in file_data {
        for fact in &file.type_escapes {
            if let Some(identity) = identity_for_escape(fact, &identity_to_row, &defs_by_file) {
                facts_by_identity
                    .entry(identity)
                    .or_default()
                    .push(fact.clone());
            }
        }
    }

    let mut helper_owners = Map::new();
    let mut type_owners = Map::new();
    let mut annotations = BTreeMap::<String, Value>::new();

    for (identity, row) in &identity_to_row {
        let annotation = build_any_annotation(
            facts_by_identity
                .get(identity)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            &row.kind,
        );
        if let Some(annotation) = annotation.clone() {
            annotations.insert(identity.clone(), annotation);
        }

        let mut owner = Map::new();
        owner.insert("ownerFile".to_string(), json!(row.file));
        owner.insert("exportedName".to_string(), json!(row.name));
        owner.insert("kind".to_string(), json!(row.kind));
        owner.insert(
            "line".to_string(),
            row.line.map_or(Value::Null, |line| json!(line)),
        );
        owner.insert(
            "anyContamination".to_string(),
            annotation.unwrap_or(Value::Null),
        );

        if is_type_owner_kind(&row.kind) {
            type_owners.insert(identity.clone(), Value::Object(owner));
        } else if is_helper_owner_kind(&row.kind) {
            helper_owners.insert(identity.clone(), Value::Object(owner));
        }
    }

    ComputedAnyContamination {
        helper_owners_by_identity: Value::Object(helper_owners),
        type_owners_by_identity: Value::Object(type_owners),
        def_index: build_annotated_def_index(root, def_index, &annotations),
    }
}

fn build_any_owner_lookups(
    root: &str,
    def_index: &[DefinitionFile],
) -> (
    BTreeMap<String, AnyOwnerRow>,
    BTreeMap<String, Vec<AnyOwnerRow>>,
) {
    let mut identity_to_row = BTreeMap::new();
    let mut defs_by_file = BTreeMap::<String, Vec<AnyOwnerRow>>::new();

    for file in def_index {
        let rel_file = rel_path(root, &file.file_path);
        for (name, def) in &file.definitions {
            let kind = value_string(def, "kind");
            if !is_any_owner_kind(&kind) {
                continue;
            }
            let identity = format!("{rel_file}::{name}");
            let row = AnyOwnerRow {
                identity: identity.clone(),
                name: name.clone(),
                file: rel_file.clone(),
                kind,
                line: value_line(def, "line"),
            };
            identity_to_row.insert(
                identity,
                AnyOwnerRow {
                    identity: row.identity.clone(),
                    name: row.name.clone(),
                    file: row.file.clone(),
                    kind: row.kind.clone(),
                    line: row.line,
                },
            );
            defs_by_file.entry(rel_file.clone()).or_default().push(row);
        }
    }

    for rows in defs_by_file.values_mut() {
        rows.sort_by(|left, right| {
            left.line
                .unwrap_or(0)
                .cmp(&right.line.unwrap_or(0))
                .then_with(|| left.name.cmp(&right.name))
        });
    }

    (identity_to_row, defs_by_file)
}

fn identity_for_escape(
    fact: &Value,
    identity_to_row: &BTreeMap<String, AnyOwnerRow>,
    defs_by_file: &BTreeMap<String, Vec<AnyOwnerRow>>,
) -> Option<String> {
    if let Some(identity) = fact
        .get("insideExportedIdentity")
        .and_then(Value::as_str)
        .filter(|identity| identity_to_row.contains_key(*identity))
    {
        return Some(identity.to_string());
    }

    if value_string(fact, "escapeKind") != "jsdoc-any" {
        return None;
    }
    let file = value_string(fact, "file");
    let line = value_line(fact, "line")?;
    defs_by_file.get(&file).and_then(|rows| {
        rows.iter()
            .find(|row| {
                let def_line = row.line.unwrap_or(0);
                def_line >= line && def_line - line <= 3
            })
            .map(|row| row.identity.clone())
    })
}

fn build_any_annotation(facts: &[Value], owner_kind: &str) -> Option<Value> {
    if facts.is_empty() {
        return None;
    }

    let mut counts = BTreeMap::<String, usize>::new();
    for fact in facts {
        let escape_kind = value_string(fact, "escapeKind");
        if !escape_kind.is_empty() {
            *counts.entry(escape_kind).or_insert(0) += 1;
        }
    }

    let any_escape_count = counts
        .iter()
        .filter(|(kind, _)| is_any_escape_kind(kind))
        .map(|(_, count)| *count)
        .sum::<usize>();
    if any_escape_count == 0 {
        return None;
    }

    let explicit_any_count = count_escape(&counts, "explicit-any");
    let as_any_count = count_escape(&counts, "as-any") + count_escape(&counts, "angle-any");
    let laundering_count = count_escape(&counts, "as-unknown-as-T");
    let rest_any_args_count = count_escape(&counts, "rest-any-args");
    let index_signature_any_count = count_escape(&counts, "index-sig-any");
    let generic_default_any_count = count_escape(&counts, "generic-default-any");
    let jsdoc_any_count = count_escape(&counts, "jsdoc-any");
    let no_explicit_any_disable_count = count_escape(&counts, "no-explicit-any-disable");
    let is_type = is_type_owner_kind(owner_kind);
    let is_helper = is_helper_owner_kind(owner_kind);
    let mut labels = BTreeSet::<String>::from(["has-any".to_string()]);

    if is_type
        || as_any_count > 0
        || explicit_any_count > 0
        || rest_any_args_count > 0
        || laundering_count > 0
        || jsdoc_any_count > 0
        || no_explicit_any_disable_count > 0
    {
        labels.insert("any-contaminated".to_string());
    }

    if laundering_count > 0
        || rest_any_args_count > 0
        || as_any_count >= 2
        || explicit_any_count >= 3
        || index_signature_any_count > 0
        || (is_type && any_escape_count >= 3)
        || (is_helper && jsdoc_any_count >= 2)
    {
        labels.insert("severely-any-contaminated".to_string());
    }

    let mut sorted_labels = labels.into_iter().collect::<Vec<_>>();
    sorted_labels.sort_by_key(|label| severity_rank(label));
    let label = sorted_labels
        .iter()
        .max_by_key(|label| severity_rank(label))
        .cloned()
        .unwrap_or_else(|| "has-any".to_string());
    let mut lines = BTreeSet::<i64>::new();
    for fact in facts {
        if let Some(line) = value_line(fact, "line") {
            lines.insert(line);
        }
    }

    Some(json!({
        "label": label,
        "labels": sorted_labels,
        "measurements": {
            "escapeCount": facts.len(),
            "anyEscapeCount": any_escape_count,
            "escapeKindCounts": counts,
            "explicitAnyCount": explicit_any_count,
            "asAnyCount": as_any_count,
            "launderingCount": laundering_count,
            "restAnyArgsCount": rest_any_args_count,
            "indexSignatureAnyCount": index_signature_any_count,
            "genericDefaultAnyCount": generic_default_any_count,
            "jsdocAnyCount": jsdoc_any_count,
            "noExplicitAnyDisableCount": no_explicit_any_disable_count,
            "lines": lines.into_iter().collect::<Vec<_>>(),
        },
    }))
}

fn build_annotated_def_index(
    root: &str,
    def_index: &[DefinitionFile],
    annotations: &BTreeMap<String, Value>,
) -> Value {
    let mut out = Map::new();
    for file in def_index {
        let rel_file = rel_path(root, &file.file_path);
        let mut definitions = Map::new();
        for (name, definition) in &file.definitions {
            let mut definition = definition.clone();
            let kind = value_string(&definition, "kind");
            if is_any_owner_kind(&kind) {
                let identity = format!("{rel_file}::{name}");
                if let Some(annotation) = annotations.get(&identity) {
                    if let Some(object) = definition.as_object_mut() {
                        object.insert("anyContamination".to_string(), annotation.clone());
                    }
                } else if let Some(object) = definition.as_object_mut() {
                    object.remove("anyContamination");
                }
            }
            definitions.insert(name.clone(), definition);
        }
        out.insert(rel_file, Value::Object(definitions));
    }
    Value::Object(out)
}

fn is_any_owner_kind(kind: &str) -> bool {
    is_type_owner_kind(kind) || is_helper_owner_kind(kind)
}

fn is_type_owner_kind(kind: &str) -> bool {
    matches!(
        kind,
        "TSInterfaceDeclaration"
            | "TSTypeAliasDeclaration"
            | "TSEnumDeclaration"
            | "TSModuleDeclaration"
    )
}

fn is_helper_owner_kind(kind: &str) -> bool {
    matches!(
        kind,
        "FunctionDeclaration" | "const-var" | "let-var" | "var-var"
    )
}

fn is_any_escape_kind(kind: &str) -> bool {
    matches!(
        kind,
        "explicit-any"
            | "as-any"
            | "angle-any"
            | "as-unknown-as-T"
            | "rest-any-args"
            | "index-sig-any"
            | "generic-default-any"
            | "no-explicit-any-disable"
            | "jsdoc-any"
    )
}

fn count_escape(counts: &BTreeMap<String, usize>, kind: &str) -> usize {
    counts.get(kind).copied().unwrap_or(0)
}

fn severity_rank(label: &str) -> i32 {
    match label {
        "severely-any-contaminated" => 3,
        "any-contaminated" => 2,
        "has-any" => 1,
        "unknown-surface" => 0,
        _ => -1,
    }
}

fn value_line(value: &Value, field: &str) -> Option<i64> {
    let value = value.get(field)?;
    if let Some(line) = value.as_i64() {
        return Some(line);
    }
    value
        .as_u64()
        .and_then(|line| i64::try_from(line).ok())
        .or_else(|| {
            value
                .as_f64()
                .filter(|line| line.is_finite())
                .map(|line| line as i64)
        })
}
