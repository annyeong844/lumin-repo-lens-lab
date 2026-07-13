use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::js_ts_extract::{ClassMethodRecord, TypeEscapeRecord, UseRecord};
use crate::shape_index::{build_shape_index_artifact, ShapeIndexRequest};
use crate::symbol_graph::any_contamination::{
    annotate_projected_def_index, ProjectedAnyContamination,
};

use super::input::{normalize_slashes, package_root, PreparedEvidenceInput};
use super::protocol::JS_TS_PRE_WRITE_EVIDENCE_RESPONSE_SCHEMA_VERSION;

pub(super) const TYPE_ESCAPE_KINDS: &[&str] = &[
    "explicit-any",
    "as-any",
    "angle-any",
    "as-unknown-as-T",
    "rest-any-args",
    "index-sig-any",
    "generic-default-any",
    "ts-ignore",
    "ts-expect-error",
    "no-explicit-any-disable",
    "jsdoc-any",
];

#[derive(Debug, Default)]
struct FanInSpaceConsumers {
    value: BTreeSet<String>,
    type_only: BTreeSet<String>,
    broad: BTreeSet<String>,
}

pub(super) fn project(input: PreparedEvidenceInput) -> Result<Value> {
    let PreparedEvidenceInput {
        root,
        evidence_artifact,
        any_inventory_artifact,
        generated,
        include_tests,
        excludes,
        dependency_roots,
        shape_intent_normalizations,
        incremental,
        rows,
        path_map,
        discovery_ms: _,
    } = input;
    let mut def_index = Map::new();
    let mut definitions_by_file = BTreeMap::<String, BTreeSet<String>>::new();
    let mut class_method_index = Map::new();
    let mut local_operations_by_file = Map::new();
    let mut parse_error_files = Vec::new();
    let mut parse_errors = Vec::new();
    let mut type_escapes = Vec::<TypeEscapeRecord>::new();
    let mut topology_nodes = Map::new();

    for row in &rows {
        topology_nodes.insert(
            row.relative_path.clone(),
            json!({ "loc": row.extracted.loc }),
        );
        if let Some(error) = row.extracted.error.as_deref() {
            parse_error_files.push(row.relative_path.clone());
            parse_errors.push(json!({
                "file": row.relative_path,
                "message": error.chars().take(200).collect::<String>(),
                "line": 0,
            }));
            continue;
        }
        type_escapes.extend(row.extracted.type_escapes.iter().cloned());
        let mut names = Map::new();
        let mut definition_names = BTreeSet::new();
        for definition in &row.extracted.defs {
            definition_names.insert(definition.name.clone());
            names.insert(
                definition.name.clone(),
                json!({
                    "name": definition.name,
                    "kind": definition.kind,
                    "line": definition.line,
                    "localName": definition.local_name,
                    "definitionId": definition.definition_id,
                }),
            );
        }
        definitions_by_file.insert(row.relative_path.clone(), definition_names);
        if !names.is_empty() {
            def_index.insert(row.relative_path.clone(), Value::Object(names));
        }
        if !row.extracted.class_methods.is_empty() {
            class_method_index.insert(
                row.relative_path.clone(),
                build_class_method_file_index(&row.relative_path, &row.extracted.class_methods)?,
            );
        }
        if !row.extracted.local_operations.is_empty() {
            local_operations_by_file.insert(
                row.relative_path.clone(),
                Value::Array(project_local_operations(
                    &row.relative_path,
                    &row.extracted.local_operations,
                )),
            );
        }
    }

    let mut fan_in_consumers = BTreeMap::<String, BTreeSet<String>>::new();
    let mut fan_in_space = BTreeMap::<String, FanInSpaceConsumers>::new();
    for (file, names) in &definitions_by_file {
        for name in names {
            let identity = format!("{file}::{name}");
            fan_in_consumers.insert(identity.clone(), BTreeSet::new());
            fan_in_space.insert(identity, FanInSpaceConsumers::default());
        }
    }

    let mut topology_edges = BTreeSet::<(String, String, bool)>::new();
    let mut dependency_consumers = BTreeMap::<String, Value>::new();
    let mut unresolved_internal = BTreeSet::new();
    for row in &rows {
        if row.extracted.error.is_some() {
            continue;
        }
        for usage in &row.extracted.uses {
            if let Some(dep_root) = package_root(&usage.from_spec)
                .filter(|dep_root| dependency_roots.contains(dep_root))
            {
                let key = format!(
                    "{}|{}|{}|{}",
                    dep_root, usage.from_spec, row.relative_path, usage.kind
                );
                dependency_consumers.entry(key).or_insert_with(|| {
                    json!({
                        "depRoot": dep_root,
                        "fromSpec": usage.from_spec,
                        "file": row.relative_path,
                        "kind": usage.kind,
                    })
                });
            }

            let Some(target) = usage
                .resolved_file
                .as_deref()
                .and_then(|file| resolved_relative_path(file, &path_map))
            else {
                if is_relative_specifier(&usage.from_spec) {
                    unresolved_internal.insert(usage.from_spec.clone());
                }
                continue;
            };
            topology_edges.insert((row.relative_path.clone(), target.clone(), usage.type_only));
            let Some(target_definitions) = definitions_by_file.get(&target) else {
                continue;
            };
            let (names, broad) = referenced_names(usage, target_definitions);
            for name in names {
                let identity = format!("{target}::{name}");
                if !broad {
                    if let Some(consumers) = fan_in_consumers.get_mut(&identity) {
                        consumers.insert(row.relative_path.clone());
                    }
                }
                if let Some(spaces) = fan_in_space.get_mut(&identity) {
                    if broad {
                        spaces.broad.insert(row.relative_path.clone());
                    } else if usage.type_only {
                        spaces.type_only.insert(row.relative_path.clone());
                    } else {
                        spaces.value.insert(row.relative_path.clone());
                    }
                }
            }
        }
    }

    let fan_in_by_identity = fan_in_consumers
        .into_iter()
        .map(|(identity, consumers)| (identity, json!(consumers.len())))
        .collect::<Map<_, _>>();
    let fan_in_by_identity_space = fan_in_space
        .into_iter()
        .map(|(identity, spaces)| {
            (
                identity,
                json!({
                    "value": spaces.value.len(),
                    "type": spaces.type_only.len(),
                    "broad": spaces.broad.len(),
                }),
            )
        })
        .collect::<Map<_, _>>();
    let topology_edges = topology_edges
        .into_iter()
        .map(|(from, to, type_only)| json!({ "from": from, "to": to, "typeOnly": type_only }))
        .collect::<Vec<_>>();
    let local_operation_count = local_operations_by_file
        .values()
        .filter_map(Value::as_array)
        .map(Vec::len)
        .sum::<usize>();
    let local_operation_owner_count = local_operations_by_file.len();
    let definition_count = definitions_by_file
        .values()
        .map(BTreeSet::len)
        .sum::<usize>();
    let type_escape_values = type_escapes
        .iter()
        .map(serde_json::to_value)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let ProjectedAnyContamination {
        helper_owners_by_identity,
        type_owners_by_identity,
    } = annotate_projected_def_index(&mut def_index, &type_escape_values);
    let dependency_consumer_count = dependency_consumers.len();
    let internal_edge_count = topology_edges.len();
    let complete = parse_error_files.is_empty();
    type_escapes.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.escape_kind.cmp(&right.escape_kind))
            .then_with(|| left.occurrence_key.cmp(&right.occurrence_key))
    });
    let type_escape_count = type_escapes.len();
    let scan_scope = if include_tests {
        "TS/JS including tests"
    } else {
        "TS/JS production files"
    };
    let files = rows
        .iter()
        .map(|row| row.relative_path.clone())
        .collect::<Vec<_>>();
    let shape_index = build_embedded_shape_index(
        &rows,
        &root,
        &generated,
        include_tests,
        &excludes,
        &incremental,
    )?;
    Ok(json!({
        "schemaVersion": JS_TS_PRE_WRITE_EVIDENCE_RESPONSE_SCHEMA_VERSION,
        "root": normalize_slashes(&root.to_string_lossy()),
        "files": files,
        "summary": {
            "fileCount": rows.len(),
            "parseErrorFileCount": parse_error_files.len(),
            "definitionCount": definition_count,
            "dependencyConsumerCount": dependency_consumer_count,
            "internalEdgeCount": internal_edge_count,
            "typeEscapeCount": type_escape_count,
        },
        "symbols": {
            "meta": {
                "tool": "lumin-audit-core js-ts-pre-write-evidence",
                "complete": complete,
                "evidenceArtifact": evidence_artifact,
                "supports": {
                    "identityFanIn": true,
                    "identityFanInSpace": true,
                    "dependencyImportConsumers": true,
                    "classMethodIndex": true,
                    "preWriteLocalOperationIndex": true,
                    "anyContamination": true,
                },
            },
            "defIndex": def_index,
            "helperOwnersByIdentity": helper_owners_by_identity,
            "typeOwnersByIdentity": type_owners_by_identity,
            "classMethodIndex": class_method_index,
            "preWriteLocalOperationIndex": {
                "schemaVersion": "pre-write-local-operations.v1",
                "status": if complete { "complete" } else { "incomplete" },
                "reason": if complete { Value::Null } else { json!("parse-errors-present") },
                "meta": { "supports": { "nestedLocalOperationIndex": true } },
                "byOwnerFile": local_operations_by_file,
                "summary": {
                    "ownerFileCount": local_operation_owner_count,
                    "operationCount": local_operation_count,
                },
            },
            "fanInByIdentity": fan_in_by_identity,
            "fanInByIdentitySpace": fan_in_by_identity_space,
            "dependencyImportConsumers": dependency_consumers.into_values().collect::<Vec<_>>(),
            "unresolvedInternalSpecifiers": unresolved_internal.into_iter().collect::<Vec<_>>(),
            "filesWithParseErrors": parse_error_files,
        },
        "topology": {
            "meta": {
                "tool": "lumin-audit-core js-ts-pre-write-evidence",
                "complete": complete,
                "evidenceArtifact": evidence_artifact,
            },
            "nodes": topology_nodes,
            "edges": topology_edges,
        },
        "shapeIndex": shape_index,
        "shapeIntentNormalizations": shape_intent_normalizations,
        "anyInventory": {
            "meta": {
                "tool": "lumin-audit-core js-ts-pre-write-evidence",
                "generated": generated,
                "root": normalize_slashes(&root.to_string_lossy()),
                "artifact": any_inventory_artifact,
                "complete": complete,
                "scope": scan_scope,
                "includeTests": include_tests,
                "exclude": excludes,
                "fileCount": rows.len(),
                "filesWithParseErrors": parse_errors,
                "incremental": incremental,
                "supports": {
                    "typeEscapes": true,
                    "escapeKinds": TYPE_ESCAPE_KINDS,
                },
            },
            "typeEscapes": type_escapes,
        },
    }))
}

fn build_embedded_shape_index(
    rows: &[super::input::SourceRow],
    root: &std::path::Path,
    generated: &str,
    include_tests: bool,
    excludes: &[String],
    incremental: &Value,
) -> Result<Value> {
    let scope = if include_tests {
        "TS/JS including tests, exported types only"
    } else {
        "TS/JS production files, exported types only"
    };
    let mut facts = Vec::new();
    let mut diagnostics = Vec::new();
    let mut files_with_parse_errors = Vec::new();
    for row in rows {
        if let Some(error) = row.extracted.error.as_deref() {
            files_with_parse_errors.push(json!({
                "file": row.relative_path,
                "message": error,
            }));
            diagnostics.push(json!({
                "kind": "shape-hash-diagnostic",
                "code": "parse-error",
                "severity": "error",
                "file": row.relative_path,
                "message": error,
            }));
            continue;
        }
        facts.extend(row.extracted.shape_facts.iter().cloned().map(|mut fact| {
            if let Some(object) = fact.as_object_mut() {
                object.insert("scope".to_string(), json!(scope));
            }
            fact
        }));
        diagnostics.extend(row.extracted.shape_diagnostics.iter().cloned());
    }
    build_shape_index_artifact(ShapeIndexRequest {
        schema_version: crate::shape_index::SHAPE_INDEX_REQUEST_SCHEMA_VERSION.to_string(),
        generated: generated.to_string(),
        root: json!(normalize_slashes(&root.to_string_lossy())),
        include_tests,
        exclude: excludes.iter().map(|value| json!(value)).collect(),
        scope: scope.to_string(),
        observed_at: generated.to_string(),
        file_count: rows.len(),
        facts,
        diagnostics,
        files_with_parse_errors,
        files_with_read_errors: Vec::new(),
        incremental: Some(incremental.clone()),
    })
}

fn build_class_method_file_index(
    relative_path: &str,
    methods: &[ClassMethodRecord],
) -> Result<Value> {
    let mut by_name = Map::<String, Value>::new();
    for method in methods {
        let value = serde_json::to_value(method)?;
        let name = if method.name.is_empty() {
            method.method_name.clone()
        } else {
            method.name.clone()
        };
        let record = json!({
            "identity": method.identity,
            "ownerFile": relative_path,
            "className": method.class_name,
            "name": name,
            "methodName": method.method_name,
            "kind": method.kind,
            "memberKind": method.member_kind,
            "visibility": method.visibility,
            "static": method.r#static,
            "computed": method.computed,
            "line": method.line,
            "endLine": value.get("endLine").cloned().unwrap_or(Value::Null),
        });
        by_name
            .entry(name)
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .context("class method group must be an array")?
            .push(record);
    }
    Ok(Value::Object(by_name))
}

fn project_local_operations(relative_path: &str, operations: &[Value]) -> Vec<Value> {
    operations
        .iter()
        .map(|operation| {
            json!({
                "identity": operation.get("identity").cloned().unwrap_or(Value::Null),
                "name": operation.get("name").cloned().unwrap_or(Value::Null),
                "ownerFile": relative_path,
                "containerName": operation.get("containerName").cloned().unwrap_or(Value::Null),
                "containerKind": operation.get("containerKind").cloned().unwrap_or(Value::Null),
                "scopeKind": operation.get("scopeKind").cloned().unwrap_or_else(|| json!("nested-function")),
                "matchedField": "preWriteLocalOperationIndex",
                "line": operation.get("line").cloned().unwrap_or(Value::Null),
                "operationFamily": operation.get("operationFamily").cloned().unwrap_or(Value::Null),
                "domainTokens": operation.get("domainTokens").cloned().unwrap_or_else(|| json!([])),
                "visibility": operation.get("visibility").cloned().unwrap_or_else(|| json!("local-only")),
                "eligibleForDeadExportRanking": false,
                "eligibleForSafeFix": false,
            })
        })
        .collect()
}

fn referenced_names(usage: &UseRecord, definitions: &BTreeSet<String>) -> (Vec<String>, bool) {
    let broad = matches!(
        usage.kind.as_str(),
        "namespace"
            | "imported-namespace-escape"
            | "cjs-namespace-escape"
            | "cjs-reexport-broad"
            | "dynamic"
            | "import-meta-glob"
    ) || usage.name == "*";
    if broad {
        return (definitions.iter().cloned().collect(), true);
    }
    let name = usage.member_name.as_deref().unwrap_or(&usage.name);
    if definitions.contains(name) {
        (vec![name.to_string()], false)
    } else {
        (Vec::new(), false)
    }
}

fn resolved_relative_path(file: &str, path_map: &BTreeMap<String, String>) -> Option<String> {
    let normalized = normalize_slashes(file);
    path_map.get(&normalized).cloned().or_else(|| {
        path_map
            .values()
            .find(|candidate| candidate.as_str() == normalized)
            .cloned()
    })
}

fn is_relative_specifier(specifier: &str) -> bool {
    specifier == "."
        || specifier == ".."
        || specifier.starts_with("./")
        || specifier.starts_with("../")
}
