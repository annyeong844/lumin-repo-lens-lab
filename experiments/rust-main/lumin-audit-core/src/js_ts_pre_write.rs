mod cache;

use crate::js_ts_extract::{ClassMethodRecord, JsTsExtractFileResult, TypeEscapeRecord, UseRecord};
use crate::scan_scope::{collect_source_files, to_repo_relative, ScanScopeOptions};
use crate::symbol_graph::any_contamination::{
    annotate_projected_def_index, ProjectedAnyContamination,
};
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use cache::{extract_with_cache, JsTsPreWriteIncrementalRequest};

pub const JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-js-ts-pre-write-evidence-request.v1";
pub const JS_TS_PRE_WRITE_EVIDENCE_RESPONSE_SCHEMA_VERSION: &str =
    "lumin-js-ts-pre-write-evidence-response.v1";
const TYPE_ESCAPE_KINDS: &[&str] = &[
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JsTsPreWriteEvidenceRequest {
    pub schema_version: String,
    pub root: PathBuf,
    pub evidence_artifact: String,
    pub any_inventory_artifact: String,
    pub generated: String,
    pub include_tests: bool,
    #[serde(default)]
    pub excludes: Vec<String>,
    #[serde(default)]
    pub dependency_roots: Vec<String>,
    #[serde(default)]
    pub discover_files: bool,
    #[serde(default)]
    pub files: Vec<JsTsPreWriteSourceFile>,
    #[serde(default)]
    pub incremental: JsTsPreWriteIncrementalRequest,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JsTsPreWriteSourceFile {
    pub file_path: PathBuf,
    pub artifact_file_path: String,
}

#[derive(Debug)]
struct SourceRow {
    relative_path: String,
    extracted: JsTsExtractFileResult,
}

struct ProjectionContext<'a> {
    root: &'a Path,
    evidence_artifact: &'a str,
    any_inventory_artifact: &'a str,
    generated: &'a str,
    include_tests: bool,
    excludes: Vec<String>,
    dependency_roots: &'a BTreeSet<String>,
    incremental: Value,
}

#[derive(Debug, Default)]
struct FanInSpaceConsumers {
    value: BTreeSet<String>,
    type_only: BTreeSet<String>,
    broad: BTreeSet<String>,
}

pub fn build_js_ts_pre_write_evidence(request: JsTsPreWriteEvidenceRequest) -> Result<Value> {
    validate_request(&request)?;

    let JsTsPreWriteEvidenceRequest {
        schema_version: _,
        root,
        evidence_artifact,
        any_inventory_artifact,
        generated,
        include_tests,
        excludes,
        dependency_roots,
        discover_files,
        mut files,
        incremental,
    } = request;
    let dependency_roots = dependency_roots.into_iter().collect::<BTreeSet<_>>();
    if discover_files {
        files = discover_js_ts_source_files(&root, include_tests, &excludes)?;
    }

    let path_map = files
        .iter()
        .map(|file| {
            (
                normalized_path(&file.file_path),
                normalize_slashes(&file.artifact_file_path),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let source_files = files
        .iter()
        .map(|file| file.file_path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let (extracted, incremental) = extract_with_cache(
        &root,
        files,
        source_files,
        include_tests,
        &excludes,
        &incremental,
    )?;
    if extracted.len() != path_map.len() {
        bail!(
            "js-ts-pre-write-evidence: extractor returned {} rows for {} files",
            extracted.len(),
            path_map.len()
        );
    }

    let mut rows = Vec::with_capacity(extracted.len());
    let mut seen = BTreeSet::new();
    for file in extracted {
        let normalized = normalize_slashes(&file.file_path);
        let relative_path = path_map.get(&normalized).with_context(|| {
            format!(
                "js-ts-pre-write-evidence: extractor returned out-of-scope file {}",
                file.file_path
            )
        })?;
        if !seen.insert(normalized) {
            bail!(
                "js-ts-pre-write-evidence: extractor returned duplicate file {}",
                file.file_path
            );
        }
        if file
            .error
            .as_deref()
            .is_some_and(|error| error.starts_with("failed to read source:"))
        {
            bail!(
                "js-ts-pre-write-evidence: failed to read required source {}: {}",
                relative_path,
                file.error.as_deref().unwrap_or("unknown read error")
            );
        }
        rows.push(SourceRow {
            relative_path: relative_path.clone(),
            extracted: file,
        });
    }
    rows.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    project_evidence(
        ProjectionContext {
            root: &root,
            evidence_artifact: &evidence_artifact,
            any_inventory_artifact: &any_inventory_artifact,
            generated: &generated,
            include_tests,
            excludes,
            dependency_roots: &dependency_roots,
            incremental,
        },
        rows,
        &path_map,
    )
}

fn validate_request(request: &JsTsPreWriteEvidenceRequest) -> Result<()> {
    if request.schema_version != JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION {
        bail!(
            "js-ts-pre-write-evidence: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if request.root.as_os_str().is_empty() || !request.root.is_absolute() {
        bail!("js-ts-pre-write-evidence: root must be an absolute path");
    }
    validate_artifact_path(&request.evidence_artifact)?;
    validate_artifact_path(&request.any_inventory_artifact)?;
    if request.generated.trim().is_empty() {
        bail!("js-ts-pre-write-evidence: generated must be a non-empty string");
    }
    if request.discover_files && !request.files.is_empty() {
        bail!("js-ts-pre-write-evidence: discoverFiles and explicit files are mutually exclusive");
    }
    let mut previous_dependency = None::<&str>;
    for dependency in &request.dependency_roots {
        if package_root(dependency).as_deref() != Some(dependency.as_str()) {
            bail!(
                "js-ts-pre-write-evidence: dependencyRoots must contain normalized package roots"
            );
        }
        if previous_dependency.is_some_and(|previous| previous >= dependency.as_str()) {
            bail!("js-ts-pre-write-evidence: dependencyRoots must be strictly sorted");
        }
        previous_dependency = Some(dependency);
    }
    let mut previous = None::<&str>;
    let mut absolute_paths = BTreeSet::new();
    for file in &request.files {
        validate_artifact_path(&file.artifact_file_path)?;
        if !file.file_path.is_absolute() || !file.file_path.starts_with(&request.root) {
            bail!(
                "js-ts-pre-write-evidence: filePath must stay inside root: {}",
                file.file_path.display()
            );
        }
        if let Some(previous) = previous {
            if previous >= file.artifact_file_path.as_str() {
                bail!(
                    "js-ts-pre-write-evidence: files must be strictly sorted by artifactFilePath"
                );
            }
        }
        previous = Some(&file.artifact_file_path);
        if !absolute_paths.insert(normalized_path(&file.file_path)) {
            bail!(
                "js-ts-pre-write-evidence: duplicate filePath {}",
                file.file_path.display()
            );
        }
    }
    Ok(())
}

fn discover_js_ts_source_files(
    root: &Path,
    include_tests: bool,
    excludes: &[String],
) -> Result<Vec<JsTsPreWriteSourceFile>> {
    let files = collect_source_files(
        root,
        &ScanScopeOptions {
            include_tests,
            exclude: excludes.to_vec(),
            languages: ["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            directory: false,
        },
    )?;
    files
        .into_iter()
        .map(|file_path| {
            let artifact_file_path = to_repo_relative(root, &file_path.to_string_lossy())
                .with_context(|| {
                    format!(
                        "js-ts-pre-write-evidence: discovered file escaped root: {}",
                        file_path.display()
                    )
                })?;
            Ok(JsTsPreWriteSourceFile {
                file_path,
                artifact_file_path,
            })
        })
        .collect()
}

fn validate_artifact_path(value: &str) -> Result<()> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        bail!("js-ts-pre-write-evidence: artifactFilePath must be a safe repo-relative path");
    }
    Ok(())
}

fn project_evidence(
    context: ProjectionContext<'_>,
    rows: Vec<SourceRow>,
    path_map: &BTreeMap<String, String>,
) -> Result<Value> {
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
                .filter(|dep_root| context.dependency_roots.contains(dep_root))
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
                .and_then(|file| resolved_relative_path(file, path_map))
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
    let scan_scope = if context.include_tests {
        "TS/JS including tests"
    } else {
        "TS/JS production files"
    };
    let files = rows
        .iter()
        .map(|row| row.relative_path.clone())
        .collect::<Vec<_>>();
    Ok(json!({
        "schemaVersion": JS_TS_PRE_WRITE_EVIDENCE_RESPONSE_SCHEMA_VERSION,
        "root": normalize_slashes(&context.root.to_string_lossy()),
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
                "evidenceArtifact": context.evidence_artifact,
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
                "evidenceArtifact": context.evidence_artifact,
            },
            "nodes": topology_nodes,
            "edges": topology_edges,
        },
        "anyInventory": {
            "meta": {
                "tool": "lumin-audit-core js-ts-pre-write-evidence",
                "generated": context.generated,
                "root": normalize_slashes(&context.root.to_string_lossy()),
                "artifact": context.any_inventory_artifact,
                "complete": complete,
                "scope": scan_scope,
                "includeTests": context.include_tests,
                "exclude": context.excludes,
                "fileCount": rows.len(),
                "filesWithParseErrors": parse_errors,
                "incremental": context.incremental,
                "supports": {
                    "typeEscapes": true,
                    "escapeKinds": TYPE_ESCAPE_KINDS,
                },
            },
            "typeEscapes": type_escapes,
        },
    }))
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

fn package_root(specifier: &str) -> Option<String> {
    if specifier.is_empty()
        || specifier.starts_with('.')
        || specifier.starts_with('/')
        || is_windows_absolute_like(specifier)
    {
        return None;
    }
    if specifier.starts_with('@') {
        let mut parts = specifier.split('/');
        let scope = parts.next()?;
        let package = parts.next()?;
        if package.is_empty() {
            return None;
        }
        return Some(format!("{scope}/{package}"));
    }
    specifier.split('/').next().map(str::to_string)
}

fn is_relative_specifier(specifier: &str) -> bool {
    specifier == "."
        || specifier == ".."
        || specifier.starts_with("./")
        || specifier.starts_with("../")
}

fn is_windows_absolute_like(value: &str) -> bool {
    value.len() >= 3 && value.as_bytes()[1] == b':' && matches!(value.as_bytes()[2], b'/' | b'\\')
}

fn normalized_path(path: &Path) -> String {
    normalize_slashes(&path.to_string_lossy())
}

fn normalize_slashes(value: &str) -> String {
    value.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn builds_compact_pre_write_evidence_without_repository_artifacts() -> Result<()> {
        let temp = tempdir()?;
        let root = temp.path();
        fs::create_dir_all(root.join("src"))?;
        fs::write(
            root.join("src/dep.ts"),
            "export function live() { return 1; }\nexport const unused = 2;\n",
        )?;
        fs::write(
            root.join("src/app.ts"),
            "import { live } from './dep';\nimport react from 'react';\nimport aliasValue from 'lib/alias';\nexport const app = 1 as any;\n",
        )?;
        fs::write(root.join("src/side.ts"), "import './dep';\n")?;
        let request = JsTsPreWriteEvidenceRequest {
            schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
            root: root.to_path_buf(),
            evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
            any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
            generated: "2026-07-11T00:00:00.000Z".to_string(),
            include_tests: true,
            excludes: vec!["vendor".to_string()],
            dependency_roots: vec!["react".to_string()],
            discover_files: false,
            files: vec![
                source_file(root, "src/app.ts"),
                source_file(root, "src/dep.ts"),
                source_file(root, "src/side.ts"),
            ],
            incremental: Default::default(),
        };

        let evidence = build_js_ts_pre_write_evidence(request)?;
        assert_eq!(
            evidence["schemaVersion"],
            JS_TS_PRE_WRITE_EVIDENCE_RESPONSE_SCHEMA_VERSION
        );
        assert_eq!(
            evidence["symbols"]["defIndex"]["src/dep.ts"]["live"]["line"],
            1
        );
        assert_eq!(
            evidence["symbols"]["fanInByIdentity"]["src/dep.ts::live"],
            1
        );
        assert_eq!(
            evidence["symbols"]["fanInByIdentity"]["src/dep.ts::unused"],
            0
        );
        assert_eq!(
            evidence["symbols"]["fanInByIdentitySpace"]["src/dep.ts::unused"]["broad"],
            1
        );
        assert_eq!(
            evidence["symbols"]["defIndex"]["src/app.ts"]["app"]["anyContamination"]["label"],
            "any-contaminated"
        );
        assert_eq!(
            evidence["symbols"]["helperOwnersByIdentity"]["src/app.ts::app"]["anyContamination"]
                ["label"],
            "any-contaminated"
        );
        assert_eq!(
            evidence["symbols"]["dependencyImportConsumers"][0]["depRoot"],
            "react"
        );
        assert_eq!(
            evidence["symbols"]["dependencyImportConsumers"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(evidence["topology"]["edges"][0]["to"], "src/dep.ts");
        assert_eq!(evidence["anyInventory"]["meta"]["complete"], true);
        assert_eq!(
            evidence["anyInventory"]["meta"]["artifact"],
            "any-inventory.pre.PROBE.json"
        );
        assert_eq!(
            evidence["anyInventory"]["meta"]["supports"]["escapeKinds"],
            json!(TYPE_ESCAPE_KINDS)
        );
        assert_eq!(
            evidence["anyInventory"]["typeEscapes"][0]["escapeKind"],
            "as-any"
        );
        Ok(())
    }

    #[test]
    fn parse_errors_degrade_every_shared_projection_without_claiming_absence() -> Result<()> {
        let temp = tempdir()?;
        let root = temp.path();
        fs::create_dir_all(root.join("src"))?;
        fs::write(root.join("src/broken.ts"), "export const = ;\n")?;
        let evidence = build_js_ts_pre_write_evidence(JsTsPreWriteEvidenceRequest {
            schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
            root: root.to_path_buf(),
            evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
            any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
            generated: "2026-07-11T00:00:00.000Z".to_string(),
            include_tests: false,
            excludes: Vec::new(),
            dependency_roots: Vec::new(),
            discover_files: false,
            files: vec![source_file(root, "src/broken.ts")],
            incremental: Default::default(),
        })?;

        assert_eq!(evidence["symbols"]["meta"]["complete"], false);
        assert_eq!(evidence["topology"]["meta"]["complete"], false);
        assert_eq!(evidence["anyInventory"]["meta"]["complete"], false);
        assert_eq!(
            evidence["anyInventory"]["meta"]["filesWithParseErrors"][0]["file"],
            "src/broken.ts"
        );
        Ok(())
    }

    #[test]
    fn unreadable_required_sources_hard_stop() -> Result<()> {
        let temp = tempdir()?;
        let root = temp.path();
        let result = build_js_ts_pre_write_evidence(JsTsPreWriteEvidenceRequest {
            schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
            root: root.to_path_buf(),
            evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
            any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
            generated: "2026-07-11T00:00:00.000Z".to_string(),
            include_tests: true,
            excludes: Vec::new(),
            dependency_roots: Vec::new(),
            discover_files: false,
            files: vec![source_file(root, "src/missing.ts")],
            incremental: Default::default(),
        });
        let Err(error) = result else {
            bail!("missing required source did not hard-stop");
        };

        assert!(error.to_string().contains("failed to read required source"));
        Ok(())
    }

    #[test]
    fn rejects_paths_outside_the_declared_root() -> Result<()> {
        let root = tempdir()?;
        let outside = tempdir()?;
        let request = JsTsPreWriteEvidenceRequest {
            schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
            root: root.path().to_path_buf(),
            evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
            any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
            generated: "2026-07-11T00:00:00.000Z".to_string(),
            include_tests: true,
            excludes: Vec::new(),
            dependency_roots: Vec::new(),
            discover_files: false,
            files: vec![JsTsPreWriteSourceFile {
                file_path: outside.path().join("outside.ts"),
                artifact_file_path: "outside.ts".to_string(),
            }],
            incremental: Default::default(),
        };
        let result = build_js_ts_pre_write_evidence(request);
        let Err(error) = result else {
            bail!("outside path did not fail");
        };
        assert!(error.to_string().contains("inside root"));
        Ok(())
    }

    #[test]
    fn discovers_the_checked_production_scope_before_parsing() -> Result<()> {
        let temp = tempdir()?;
        let root = temp.path();
        fs::create_dir_all(root.join("src"))?;
        fs::write(root.join("src/app.ts"), "export const app = true;\n")?;
        fs::write(
            root.join("src/app.test.ts"),
            "export const testOnly = true;\n",
        )?;

        let evidence = build_js_ts_pre_write_evidence(JsTsPreWriteEvidenceRequest {
            schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
            root: root.to_path_buf(),
            evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
            any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
            generated: "2026-07-11T00:00:00.000Z".to_string(),
            include_tests: false,
            excludes: Vec::new(),
            dependency_roots: Vec::new(),
            discover_files: true,
            files: Vec::new(),
            incremental: Default::default(),
        })?;

        assert_eq!(evidence["files"], json!(["src/app.ts"]));
        assert_eq!(evidence["summary"]["fileCount"], 1);
        assert!(evidence["symbols"]["defIndex"]
            .get("src/app.test.ts")
            .is_none());
        Ok(())
    }

    #[test]
    fn strict_cache_reuses_only_byte_identical_files_and_rebuilds_current_evidence() -> Result<()> {
        let temp = tempdir()?;
        let root = temp.path();
        let cache_root = root.join(".cache");
        fs::create_dir_all(root.join("src"))?;
        fs::write(root.join("src/a.ts"), "export const a = 1;\n")?;
        fs::write(root.join("src/b.ts"), "export const b = 1;\n")?;

        let build = || {
            build_js_ts_pre_write_evidence(JsTsPreWriteEvidenceRequest {
                schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
                root: root.to_path_buf(),
                evidence_artifact: "pre-write-evidence.PROBE.json".to_string(),
                any_inventory_artifact: "any-inventory.pre.PROBE.json".to_string(),
                generated: "2026-07-11T00:00:00.000Z".to_string(),
                include_tests: true,
                excludes: Vec::new(),
                dependency_roots: Vec::new(),
                discover_files: true,
                files: Vec::new(),
                incremental: JsTsPreWriteIncrementalRequest {
                    enabled: true,
                    cache_root: Some(cache_root.clone()),
                    clear: false,
                },
            })
        };

        let cold = build()?;
        assert_eq!(
            cold["anyInventory"]["meta"]["incremental"]["changedFiles"],
            2
        );
        assert_eq!(
            cold["anyInventory"]["meta"]["incremental"]["reusedFiles"],
            0
        );

        let warm = build()?;
        assert_eq!(
            warm["anyInventory"]["meta"]["incremental"]["changedFiles"],
            0
        );
        assert_eq!(
            warm["anyInventory"]["meta"]["incremental"]["reusedFiles"],
            2
        );
        assert_eq!(
            warm["anyInventory"]["meta"]["incremental"]["writeStatus"],
            "unchanged"
        );
        assert_eq!(warm["symbols"]["defIndex"]["src/a.ts"]["a"]["line"], 1);

        fs::write(root.join("src/a.ts"), "export const a = 2;\n")?;
        let changed = build()?;
        assert_eq!(
            changed["anyInventory"]["meta"]["incremental"]["changedFiles"],
            1
        );
        assert_eq!(
            changed["anyInventory"]["meta"]["incremental"]["reusedFiles"],
            1
        );
        assert_eq!(changed["summary"]["fileCount"], 2);

        fs::write(root.join("src/c.ts"), "export const c = 1;\n")?;
        let expanded = build()?;
        assert_eq!(
            expanded["anyInventory"]["meta"]["incremental"]["changedFiles"],
            3
        );
        assert_eq!(
            expanded["anyInventory"]["meta"]["incremental"]["reusedFiles"],
            0
        );
        assert_eq!(expanded["summary"]["fileCount"], 3);
        Ok(())
    }

    fn source_file(root: &Path, relative: &str) -> JsTsPreWriteSourceFile {
        JsTsPreWriteSourceFile {
            file_path: root.join(relative),
            artifact_file_path: relative.to_string(),
        }
    }
}
