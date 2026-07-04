use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::cmp::Reverse;
use std::collections::BTreeMap;

pub const SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION: &str = "lumin-symbol-graph-producer-request.v1";

const TOOL_NAME: &str = "build-symbol-graph.mjs";
const SYMBOL_META_SCHEMA_VERSION: i64 = 3;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolGraphRequest {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub def_index: Vec<DefinitionFile>,
    #[serde(default)]
    pub file_data: Vec<FileDataRecord>,
    #[serde(default)]
    pub parse_errors: usize,
    #[serde(default)]
    pub warnings: Vec<Value>,
    #[serde(default)]
    pub next_cache_entries: BTreeMap<String, Value>,
    #[serde(default)]
    pub unresolved_internal_by_prefix: Vec<CountEntry>,
    #[serde(default)]
    pub prefix_examples: BTreeMap<String, String>,
    #[serde(default)]
    pub unresolved_internal_specifiers: Vec<String>,
    #[serde(default)]
    pub unresolved_internal_specifier_records: Vec<Value>,
    #[serde(default)]
    pub language_support: Value,
    #[serde(default)]
    pub total_uses: usize,
    #[serde(default)]
    pub unresolved_uses: usize,
    #[serde(default)]
    pub resolved_internal_uses: usize,
    #[serde(default)]
    pub resolved_generated_virtual_uses: usize,
    #[serde(default)]
    pub non_source_asset_uses: usize,
    #[serde(default)]
    pub external_uses: usize,
    #[serde(default)]
    pub dependency_import_consumers: Vec<Value>,
    #[serde(default)]
    pub resolved_internal_edges: Vec<Value>,
    #[serde(default)]
    pub generated_consumer_blind_zones: Vec<Value>,
    #[serde(default)]
    pub generated_virtual_surfaces: Vec<Value>,
    #[serde(default)]
    pub generated_virtual_import_consumers: Vec<Value>,
    #[serde(default)]
    pub unresolved_internal_uses: usize,
    #[serde(default)]
    pub mdx_consumer_uses: usize,
    #[serde(default)]
    pub sfc_script_consumer_uses: usize,
    #[serde(default)]
    pub sfc_script_src_reachability_uses: usize,
    #[serde(default)]
    pub sfc_style_asset_reference_uses: usize,
    #[serde(default)]
    pub sfc_template_component_ref_uses: usize,
    #[serde(default)]
    pub sfc_global_component_registration_uses: usize,
    #[serde(default)]
    pub sfc_generated_component_manifest_uses: usize,
    #[serde(default)]
    pub sfc_framework_convention_component_uses: usize,
    #[serde(default)]
    pub sfc_style_asset_references: Vec<Value>,
    #[serde(default)]
    pub sfc_template_component_refs: Vec<Value>,
    #[serde(default)]
    pub sfc_global_component_registrations: Vec<Value>,
    #[serde(default)]
    pub sfc_generated_component_manifests: Vec<Value>,
    #[serde(default)]
    pub sfc_framework_convention_components: Vec<Value>,
    #[serde(default)]
    pub dead: Vec<Value>,
    #[serde(default)]
    pub truly_dead: Vec<Value>,
    #[serde(default)]
    pub dead_in_prod: Vec<Value>,
    #[serde(default)]
    pub dead_in_test: Vec<Value>,
    #[serde(default)]
    pub symbol_fan_in: Vec<Value>,
    #[serde(default)]
    pub fan_in_by_identity: Value,
    #[serde(default)]
    pub fan_in_by_identity_space: Value,
    #[serde(default)]
    pub namespace_re_export_diagnostics: Vec<Value>,
    #[serde(default)]
    pub any_contamination_facts: Value,
    #[serde(default)]
    pub incremental: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionFile {
    pub file_path: String,
    #[serde(default)]
    pub definitions: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDataRecord {
    pub file_path: String,
    #[serde(default)]
    pub re_exports: Vec<Value>,
    #[serde(default)]
    pub class_methods: Vec<Value>,
    #[serde(default)]
    pub local_operations: Vec<Value>,
    #[serde(default)]
    pub dynamic_import_opacity: Vec<Value>,
    #[serde(default)]
    pub cjs_export_surface: Option<Value>,
    #[serde(default)]
    pub cjs_require_opacity: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountEntry {
    pub key: String,
    pub count: usize,
}

pub fn build_symbol_graph_artifact(request: SymbolGraphRequest) -> Result<Value> {
    if request.schema_version != SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION {
        bail!(
            "symbol-graph-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let mut warnings = request.warnings.clone();
    if request.parse_errors > 0 {
        warnings.push(json!({
            "code": "parse-errors",
            "count": request.parse_errors,
            "message": format!("{} file(s) failed to parse; their defs/uses are missing from the graph", request.parse_errors),
        }));
    }

    let supports = json!({
        "anyContamination": true,
        "identityFanIn": true,
        "identityFanInSpace": true,
        "reExportRecords": "file-level",
        "mdxImportConsumers": true,
        "sfcScriptImportConsumers": true,
        "sfcScriptSrcReachability": true,
        "sfcStyleAssetReferences": true,
        "sfcTemplateComponentRefs": true,
        "sfcGlobalComponentRegistrations": true,
        "sfcGeneratedComponentManifests": true,
        "sfcFrameworkConventionComponents": true,
        "dependencyImportConsumers": true,
        "resolvedInternalEdges": true,
        "definitionIds": true,
        "unresolvedInternalSummaryByReason": true,
        "cjsExportSurface": true,
        "cjsRequireOpacity": true,
        "generatedConsumerBlindZones": true,
        "generatedVirtualSurfaces": true,
        "nonSourceAssetImports": true,
        "namespaceReExportDiagnostics": true,
        "classMethodIndex": true,
        "nestedLocalOperationIndex": true,
    });

    let mut meta = Map::new();
    meta.insert("tool".to_string(), json!(TOOL_NAME));
    meta.insert("generated".to_string(), json!(request.generated));
    meta.insert("root".to_string(), json!(request.root));
    meta.insert(
        "schemaVersion".to_string(),
        json!(SYMBOL_META_SCHEMA_VERSION),
    );
    meta.insert("supports".to_string(), supports);
    meta.insert("languageSupport".to_string(), request.language_support);
    meta.insert("warnings".to_string(), Value::Array(warnings));
    if let Some(incremental) = request.incremental {
        meta.insert("incremental".to_string(), incremental);
    }

    let total_defs = request
        .def_index
        .iter()
        .map(|file| file.definitions.len())
        .sum::<usize>();
    let total_class_methods = request
        .file_data
        .iter()
        .map(|file| file.class_methods.len())
        .sum::<usize>();
    let total_local_operations = request
        .file_data
        .iter()
        .map(|file| file.local_operations.len())
        .sum::<usize>();
    let unresolved_ratio = if request.resolved_internal_uses + request.unresolved_internal_uses > 0
    {
        round4(
            request.unresolved_internal_uses as f64
                / (request.resolved_internal_uses + request.unresolved_internal_uses) as f64,
        )
    } else {
        0.0
    };

    let any_contamination = request
        .any_contamination_facts
        .as_object()
        .cloned()
        .unwrap_or_default();

    Ok(json!({
        "meta": Value::Object(meta),
        "files": request.files.len(),
        "totalDefs": total_defs,
        "totalClassMethods": total_class_methods,
        "totalPreWriteLocalOperations": total_local_operations,
        "totalUsesResolved": request.total_uses,
        "unresolvedUses": request.unresolved_uses,
        "uses": {
            "resolvedInternal": request.resolved_internal_uses,
            "resolvedGeneratedVirtual": request.resolved_generated_virtual_uses,
            "nonSourceAsset": request.non_source_asset_uses,
            "external": request.external_uses,
            "unresolvedInternal": request.unresolved_internal_uses,
            "mdxConsumers": request.mdx_consumer_uses,
            "sfcScriptConsumers": request.sfc_script_consumer_uses,
            "sfcScriptSrcReachability": request.sfc_script_src_reachability_uses,
            "sfcStyleAssetReferences": request.sfc_style_asset_reference_uses,
            "sfcTemplateComponentRefs": request.sfc_template_component_ref_uses,
            "sfcGlobalComponentRegistrations": request.sfc_global_component_registration_uses,
            "sfcGeneratedComponentManifests": request.sfc_generated_component_manifest_uses,
            "sfcFrameworkConventionComponents": request.sfc_framework_convention_component_uses,
            "unresolvedInternalRatio": unresolved_ratio,
        },
        "dependencyImportConsumers": sort_values_by_key(request.dependency_import_consumers, dependency_consumer_key),
        "resolvedInternalEdges": sort_values_by_key(request.resolved_internal_edges, resolved_internal_edge_key),
        "sfcStyleAssetReferences": sort_values_by_key(request.sfc_style_asset_references, sfc_style_asset_key),
        "sfcTemplateComponentRefs": sort_values_by_key(request.sfc_template_component_refs, sfc_template_ref_key),
        "sfcGlobalComponentRegistrations": sort_values_by_key(request.sfc_global_component_registrations, sfc_global_registration_key),
        "sfcGeneratedComponentManifests": sort_values_by_key(request.sfc_generated_component_manifests, sfc_generated_manifest_key),
        "sfcFrameworkConventionComponents": sort_values_by_key(request.sfc_framework_convention_components, sfc_framework_convention_key),
        "generatedConsumerBlindZones": sort_values_by_key(request.generated_consumer_blind_zones, generated_blind_zone_key),
        "generatedVirtualSurfaces": sort_generated_virtual_surfaces(request.generated_virtual_surfaces),
        "generatedVirtualImportConsumers": sort_values_by_key(request.generated_virtual_import_consumers, generated_import_consumer_key),
        "topUnresolvedSpecifiers": top_unresolved_specifiers(&request.unresolved_internal_by_prefix, &request.prefix_examples),
        "dynamicImportOpacity": build_dynamic_import_opacity(&request.root, &request.file_data),
        "cjsExportSurfaceByFile": build_cjs_export_surface_by_file(&request.root, &request.file_data),
        "cjsRequireOpacity": build_cjs_require_opacity(&request.root, &request.file_data),
        "unresolvedInternalSpecifiers": sorted_strings(request.unresolved_internal_specifiers),
        "unresolvedInternalSpecifierRecords": sort_values_by_key(request.unresolved_internal_specifier_records.clone(), unresolved_record_key),
        "unresolvedInternalSummaryByReason": unresolved_summary_by_reason(&request.unresolved_internal_specifier_records),
        "filesWithParseErrors": files_with_parse_errors(&request.root, &request.next_cache_entries),
        "deadTotal": request.dead.len(),
        "trulyDead": request.truly_dead.len(),
        "deadInProd": request.dead_in_prod.len(),
        "deadInTest": request.dead_in_test.len(),
        "topSymbolFanIn": top_symbol_fan_in(request.symbol_fan_in),
        "fanInByIdentity": object_or_empty(request.fan_in_by_identity),
        "fanInByIdentitySpace": object_or_empty(request.fan_in_by_identity_space),
        "namespaceReExportDiagnostics": sort_values_by_key(request.namespace_re_export_diagnostics, namespace_re_export_key),
        "helperOwnersByIdentity": any_contamination.get("helperOwnersByIdentity").cloned().unwrap_or_else(|| json!({})),
        "typeOwnersByIdentity": any_contamination.get("typeOwnersByIdentity").cloned().unwrap_or_else(|| json!({})),
        "defIndex": build_plain_def_index(&request.root, &request.def_index),
        "classMethodIndex": build_class_method_index(&request.root, &request.file_data),
        "preWriteLocalOperationIndex": build_pre_write_local_operation_index(&request.root, &request.file_data),
        "deadProdList": request.dead_in_prod,
        "reExportsByFile": build_re_exports_by_file(&request.root, &request.file_data),
    }))
}

fn sort_values_by_key(mut values: Vec<Value>, key_fn: fn(&Value) -> String) -> Vec<Value> {
    values.sort_by_key(key_fn);
    values
}

fn sorted_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

fn value_string(value: &Value, field: &str) -> String {
    value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn value_bool_key(value: &Value, field: &str) -> &'static str {
    if value.get(field).and_then(Value::as_bool) == Some(true) {
        "1"
    } else {
        "0"
    }
}

fn padded_line(value: &Value) -> String {
    let raw = value
        .get("line")
        .cloned()
        .unwrap_or(Value::String(String::new()));
    match raw {
        Value::Number(number) => format!("{number:0>6}"),
        Value::String(text) => format!("{text:0>6}"),
        _ => String::from("000000"),
    }
}

fn dependency_consumer_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}",
        value_string(value, "depRoot"),
        value_string(value, "fromSpec"),
        value_string(value, "file"),
        value_string(value, "kind")
    )
}

fn resolved_internal_edge_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "from"),
        value_string(value, "to"),
        value_string(value, "kind"),
        value_string(value, "source"),
        value_bool_key(value, "typeOnly")
    )
}

fn sfc_style_asset_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "fromSpec"),
        value_string(value, "source"),
        value_string(value, "status")
    )
}

fn sfc_template_ref_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "tagName"),
        value_string(value, "bindingName"),
        value_string(value, "status"),
        value_string(value, "reason")
    )
}

fn sfc_global_registration_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "registrationFile"),
        value_string(value, "componentName"),
        value_string(value, "bindingName"),
        value_string(value, "status"),
        value_string(value, "reason")
    )
}

fn sfc_generated_manifest_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "manifestFile"),
        value_string(value, "componentName"),
        value_string(value, "fromSpec"),
        value_string(value, "status"),
        value_string(value, "reason")
    )
}

fn sfc_framework_convention_key(value: &Value) -> String {
    [
        "framework",
        "conventionKind",
        "consumerFile",
        "sourceFile",
        "configFile",
        "componentName",
        "tagName",
        "directiveName",
        "actionName",
        "subscriptionName",
        "storeName",
        "macroName",
        "fromSpec",
    ]
    .iter()
    .map(|field| value_string(value, field))
    .collect::<Vec<_>>()
    .join("|")
}

fn generated_blind_zone_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}",
        value_string(value, "scopePackageRoot"),
        value_string(value, "candidatePath"),
        value_string(value, "specifier"),
        value_string(value, "consumerFile")
    )
}

fn generated_import_consumer_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "specifier"),
        value_string(value, "name"),
        value_string(value, "kind"),
        value_string(value, "surfaceId")
    )
}

fn unresolved_record_key(value: &Value) -> String {
    format!(
        "{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "specifier"),
        value_string(value, "kind")
    )
}

fn namespace_re_export_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "exportedName"),
        value_string(value, "targetFile"),
        value_string(value, "kind"),
        value.get("line").map(Value::to_string).unwrap_or_default()
    )
}

fn top_unresolved_specifiers(
    counters: &[CountEntry],
    examples: &BTreeMap<String, String>,
) -> Vec<Value> {
    let mut entries = counters.to_vec();
    entries.sort_by_key(|entry| Reverse(entry.count));
    entries
        .into_iter()
        .take(20)
        .map(|entry| {
            let example = examples
                .get(&entry.key)
                .cloned()
                .unwrap_or_else(|| entry.key.clone());
            let mut object = Map::new();
            object.insert("specifierPrefix".to_string(), json!(entry.key));
            object.insert("count".to_string(), json!(entry.count));
            object.insert("example".to_string(), json!(example));
            if likely_alias_prefix(object.get("specifierPrefix").and_then(Value::as_str).unwrap_or_default()) {
                object.insert(
                    "likelyCause".to_string(),
                    json!("possible unresolved tsconfig paths alias. Check per-app tsconfig.json for a compilerOptions.paths entry matching this prefix. See FP-36 in references/false-positive-index.md."),
                );
            }
            Value::Object(object)
        })
        .collect()
}

fn likely_alias_prefix(prefix: &str) -> bool {
    prefix.starts_with("@/")
        || prefix.starts_with("~/")
        || prefix.starts_with("#/")
        || (prefix.starts_with('@') && prefix.get(1..).is_some_and(|rest| rest.contains('/')))
}

fn compact_unresolved_example(record: &Value) -> Value {
    let mut object = Map::new();
    for field in ["specifier", "consumerFile", "kind"] {
        if let Some(value) = record.get(field) {
            object.insert(field.to_string(), value.clone());
        }
    }
    if let Some(value) = record.get("typeOnly").filter(|value| value.is_boolean()) {
        object.insert("typeOnly".to_string(), value.clone());
    }
    for field in ["resolverStage", "matchedPattern", "hint"] {
        if let Some(value) = record.get(field).filter(|value| value.is_string()) {
            object.insert(field.to_string(), value.clone());
        }
    }
    if let Some(candidates) = record.get("targetCandidates").and_then(Value::as_array) {
        if !candidates.is_empty() {
            object.insert(
                "targetCandidates".to_string(),
                Value::Array(candidates.iter().take(3).cloned().collect()),
            );
        }
    }
    Value::Object(object)
}

fn unresolved_space(record: &Value) -> &'static str {
    match record.get("typeOnly").and_then(Value::as_bool) {
        Some(true) => "type",
        Some(false) => "value",
        None => "unknown",
    }
}

#[derive(Default)]
struct UnresolvedGroup {
    count: usize,
    spaces_type: usize,
    spaces_value: usize,
    spaces_unknown: usize,
    resolver_stages: BTreeMap<String, usize>,
    hints: BTreeMap<String, usize>,
    examples: Vec<Value>,
}

fn unresolved_summary_by_reason(records: &[Value]) -> Value {
    let mut groups = BTreeMap::<String, UnresolvedGroup>::new();
    for record in records {
        let reason = value_string(record, "reason");
        let reason = if reason.is_empty() {
            "unknown-internal-resolution".to_string()
        } else {
            reason
        };
        let group = groups.entry(reason).or_default();
        group.count += 1;
        match unresolved_space(record) {
            "type" => group.spaces_type += 1,
            "value" => group.spaces_value += 1,
            _ => group.spaces_unknown += 1,
        }
        let resolver_stage = value_string(record, "resolverStage");
        if !resolver_stage.is_empty() {
            *group.resolver_stages.entry(resolver_stage).or_insert(0) += 1;
        }
        let hint = value_string(record, "hint");
        if !hint.is_empty() {
            *group.hints.entry(hint).or_insert(0) += 1;
        }
        group.examples.push(compact_unresolved_example(record));
    }

    let mut entries = groups.into_iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .1
            .count
            .cmp(&left.1.count)
            .then_with(|| left.0.cmp(&right.0))
    });
    let mut out = Map::new();
    for (reason, mut group) in entries {
        group.examples.sort_by_key(unresolved_record_key);
        let mut object = Map::new();
        object.insert("count".to_string(), json!(group.count));
        object.insert(
            "spaces".to_string(),
            json!({
                "type": group.spaces_type,
                "value": group.spaces_value,
                "unknown": group.spaces_unknown,
            }),
        );
        if !group.resolver_stages.is_empty() {
            object.insert("resolverStages".to_string(), json!(group.resolver_stages));
        }
        if !group.hints.is_empty() {
            object.insert("hints".to_string(), json!(group.hints));
        }
        object.insert(
            "examples".to_string(),
            Value::Array(group.examples.into_iter().take(5).collect()),
        );
        out.insert(reason, Value::Object(object));
    }
    Value::Object(out)
}

fn build_dynamic_import_opacity(root: &str, file_data: &[FileDataRecord]) -> Vec<Value> {
    let mut records = Vec::new();
    for file in file_data {
        let consumer_file = rel_path(root, &file.file_path);
        for item in &file.dynamic_import_opacity {
            let mut object = Map::new();
            object.insert("consumerFile".to_string(), json!(consumer_file));
            if let Some(line) = item.get("line") {
                object.insert("line".to_string(), line.clone());
            }
            if let Some(kind) = item.get("kind") {
                object.insert("kind".to_string(), kind.clone());
            }
            if let Some(prefix) = item.get("prefix").and_then(Value::as_str) {
                let target = resolve_prefix_target(&file.file_path, prefix);
                object.insert("prefix".to_string(), json!(prefix));
                object.insert(
                    "targetDir".to_string(),
                    json!(format!(
                        "{}/",
                        rel_path(root, &target).trim_end_matches('/')
                    )),
                );
            }
            records.push(Value::Object(object));
        }
    }
    sort_values_by_key(records, dynamic_opacity_key)
}

fn dynamic_opacity_key(value: &Value) -> String {
    format!(
        "{}|{}|{}",
        value_string(value, "consumerFile"),
        padded_line(value),
        value_string(value, "prefix")
    )
}

fn build_cjs_export_surface_by_file(root: &str, file_data: &[FileDataRecord]) -> Value {
    let mut out = Map::new();
    for file in file_data {
        let Some(surface) = file.cjs_export_surface.as_ref().and_then(Value::as_object) else {
            continue;
        };
        let exact = surface
            .get("exact")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let opaque = surface
            .get("opaque")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if exact.is_empty() && opaque.is_empty() {
            continue;
        }
        out.insert(
            rel_path(root, &file.file_path),
            json!({
                "exact": sort_cjs_surface_list(exact),
                "opaque": sort_cjs_surface_list(opaque),
            }),
        );
    }
    Value::Object(out)
}

fn sort_cjs_surface_list(values: Vec<Value>) -> Vec<Value> {
    sort_values_by_key(values, |value| {
        format!(
            "{}|{}|{}",
            value_string(value, "name"),
            value_string(value, "kind"),
            padded_line(value)
        )
    })
}

fn build_cjs_require_opacity(root: &str, file_data: &[FileDataRecord]) -> Vec<Value> {
    let mut records = Vec::new();
    for file in file_data {
        for item in &file.cjs_require_opacity {
            records.push(json!({
                "consumerFile": rel_path(root, &file.file_path),
                "line": item.get("line").cloned().unwrap_or(Value::Null),
                "kind": item.get("kind").cloned().unwrap_or(Value::Null),
            }));
        }
    }
    sort_values_by_key(records, |value| {
        format!(
            "{}|{}|{}",
            value_string(value, "consumerFile"),
            padded_line(value),
            value_string(value, "kind")
        )
    })
}

fn files_with_parse_errors(root: &str, entries: &BTreeMap<String, Value>) -> Vec<String> {
    let mut files = entries
        .iter()
        .filter_map(|(file, entry)| {
            if entry.get("parseError").and_then(Value::as_bool) == Some(true) {
                Some(rel_path(root, file))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn top_symbol_fan_in(mut values: Vec<Value>) -> Vec<Value> {
    values.sort_by(|left, right| {
        let left_count = left.get("count").and_then(Value::as_i64).unwrap_or(0);
        let right_count = right.get("count").and_then(Value::as_i64).unwrap_or(0);
        right_count.cmp(&left_count)
    });
    values.truncate(50);
    values
}

fn object_or_empty(value: Value) -> Value {
    if value.is_object() {
        value
    } else {
        json!({})
    }
}

fn build_plain_def_index(root: &str, def_index: &[DefinitionFile]) -> Value {
    let mut out = Map::new();
    for file in def_index {
        out.insert(rel_path(root, &file.file_path), json!(file.definitions));
    }
    Value::Object(out)
}

fn sort_class_method_records(values: &[Value]) -> Vec<Value> {
    let mut records = values.to_vec();
    records.sort_by_key(|value| {
        format!(
            "{}|{}|{}|{}",
            value_string(value, "className"),
            value_string(value, "name"),
            padded_line(value),
            value_string(value, "identity")
        )
    });
    records
}

fn build_class_method_index(root: &str, file_data: &[FileDataRecord]) -> Value {
    let mut out = Map::new();
    for file in file_data {
        if file.class_methods.is_empty() {
            continue;
        }
        let rel = rel_path(root, &file.file_path);
        let mut by_name = Map::<String, Value>::new();
        for method in sort_class_method_records(&file.class_methods) {
            let name = value_string(&method, "name");
            let name = if name.is_empty() {
                value_string(&method, "methodName")
            } else {
                name
            };
            if name.is_empty() {
                continue;
            }
            let class_name = value_string(&method, "className");
            let record = json!({
                "identity": method.get("identity").cloned().unwrap_or_else(|| json!(format!("{rel}::{class_name}#{name}"))),
                "ownerFile": method.get("ownerFile").cloned().unwrap_or_else(|| json!(rel)),
                "className": method.get("className").cloned().unwrap_or(Value::Null),
                "name": name,
                "methodName": method.get("methodName").cloned().unwrap_or_else(|| json!(name)),
                "kind": method.get("kind").cloned().unwrap_or_else(|| json!("ClassMethod")),
                "memberKind": method.get("memberKind").cloned().unwrap_or_else(|| json!("method")),
                "visibility": method.get("visibility").cloned().unwrap_or_else(|| json!("public")),
                "static": method.get("static").and_then(Value::as_bool).unwrap_or(false),
                "computed": method.get("computed").and_then(Value::as_bool).unwrap_or(false),
                "line": method.get("line").cloned().unwrap_or(Value::Null),
            });
            let mut record = record.as_object().cloned().unwrap_or_default();
            if let Some(end_line) = method.get("endLine") {
                record.insert("endLine".to_string(), end_line.clone());
            }
            let method_group = by_name
                .entry(name)
                .or_insert_with(|| Value::Array(Vec::new()));
            if let Value::Array(methods) = method_group {
                methods.push(Value::Object(record));
            }
        }
        if !by_name.is_empty() {
            out.insert(rel, Value::Object(by_name));
        }
    }
    Value::Object(out)
}

fn sort_local_operation_records(values: &[Value]) -> Vec<Value> {
    let mut records = values.to_vec();
    records.sort_by_key(|value| {
        format!(
            "{}|{}|{}|{}",
            value_string(value, "containerName"),
            value_string(value, "name"),
            padded_line(value),
            value_string(value, "identity")
        )
    });
    records
}

fn build_pre_write_local_operation_index(root: &str, file_data: &[FileDataRecord]) -> Value {
    let mut by_owner_file = Map::new();
    let mut operation_count = 0usize;
    for file in file_data {
        let operations = sort_local_operation_records(&file.local_operations);
        if operations.is_empty() {
            continue;
        }
        let rel = rel_path(root, &file.file_path);
        let projected = operations
            .into_iter()
            .map(|operation| {
                json!({
                    "identity": operation.get("identity").cloned().unwrap_or(Value::Null),
                    "name": operation.get("name").cloned().unwrap_or(Value::Null),
                    "ownerFile": operation.get("ownerFile").cloned().unwrap_or_else(|| json!(rel)),
                    "containerName": operation.get("containerName").cloned().unwrap_or(Value::Null),
                    "containerKind": operation.get("containerKind").cloned().unwrap_or(Value::Null),
                    "scopeKind": operation.get("scopeKind").cloned().unwrap_or_else(|| json!("nested-function")),
                    "matchedField": operation.get("matchedField").cloned().unwrap_or_else(|| json!("preWriteLocalOperationIndex")),
                    "line": operation.get("line").cloned().unwrap_or(Value::Null),
                    "operationFamily": operation.get("operationFamily").cloned().unwrap_or(Value::Null),
                    "domainTokens": sorted_value_strings(operation.get("domainTokens")),
                    "visibility": operation.get("visibility").cloned().unwrap_or_else(|| json!("local-only")),
                    "eligibleForDeadExportRanking": false,
                    "eligibleForSafeFix": false,
                })
            })
            .collect::<Vec<_>>();
        operation_count += projected.len();
        by_owner_file.insert(rel, Value::Array(projected));
    }
    json!({
        "schemaVersion": "pre-write-local-operations.v1",
        "status": "complete",
        "meta": {
            "supports": {
                "nestedLocalOperationIndex": true,
            },
        },
        "byOwnerFile": by_owner_file,
        "summary": {
            "ownerFileCount": by_owner_file.len(),
            "operationCount": operation_count,
        },
    })
}

fn sorted_value_strings(value: Option<&Value>) -> Vec<String> {
    let mut strings = value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    strings.sort();
    strings
}

fn build_re_exports_by_file(root: &str, file_data: &[FileDataRecord]) -> Value {
    let mut out = Map::new();
    for file in file_data {
        if file.re_exports.is_empty() {
            continue;
        }
        let records = file
            .re_exports
            .iter()
            .map(|item| {
                let mut object = Map::new();
                if let Some(source) = item.get("source") {
                    object.insert("source".to_string(), source.clone());
                }
                if let Some(line) = item.get("line") {
                    object.insert("line".to_string(), line.clone());
                }
                if let Some(namespace) = item.get("namespace") {
                    object.insert("namespace".to_string(), namespace.clone());
                }
                Value::Object(object)
            })
            .collect::<Vec<_>>();
        out.insert(rel_path(root, &file.file_path), Value::Array(records));
    }
    Value::Object(out)
}

fn sort_generated_virtual_surfaces(values: Vec<Value>) -> Vec<Value> {
    let mut surfaces = values
        .into_iter()
        .map(|surface| {
            let mut object = surface.as_object().cloned().unwrap_or_default();
            let mut exports = object
                .get("exports")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            exports.sort_by_key(|entry| {
                format!(
                    "{}|{}",
                    value_string(entry, "name"),
                    value_string(entry, "kind")
                )
            });
            object.insert("exports".to_string(), Value::Array(exports));
            Value::Object(object)
        })
        .collect::<Vec<_>>();
    surfaces.sort_by_key(|surface| value_string(surface, "id"));
    surfaces
}

fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

fn rel_path(root: &str, file: &str) -> String {
    let root = normalize_slashes(root).trim_end_matches('/').to_string();
    let file = normalize_slashes(file);
    let prefix = format!("{root}/");
    if let Some(stripped) = file.strip_prefix(&prefix) {
        stripped.to_string()
    } else {
        file
    }
}

fn normalize_slashes(value: &str) -> String {
    value.replace('\\', "/")
}

fn resolve_prefix_target(file: &str, prefix: &str) -> String {
    let normalized_file = normalize_slashes(file);
    let base = normalized_file
        .rsplit_once('/')
        .map(|(base, _)| base)
        .unwrap_or("");
    normalize_path_segments(&format!("{base}/{prefix}"))
}

fn normalize_path_segments(path: &str) -> String {
    let mut prefix = String::new();
    let mut rest = normalize_slashes(path);
    if rest.len() >= 3 && rest.as_bytes()[1] == b':' && rest.as_bytes()[2] == b'/' {
        prefix = rest[..3].to_string();
        rest = rest[3..].to_string();
    } else if rest.starts_with('/') {
        prefix = "/".to_string();
        rest = rest.trim_start_matches('/').to_string();
    }

    let mut parts = Vec::new();
    for part in rest.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    format!("{prefix}{}", parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_symbols_artifact_from_js_facts() -> Result<()> {
        let artifact = build_symbol_graph_artifact(SymbolGraphRequest {
            schema_version: SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            files: vec![
                "C:/repo/src/a.ts".to_string(),
                "C:/repo/src/b.ts".to_string(),
            ],
            def_index: vec![DefinitionFile {
                file_path: "C:/repo/src/a.ts".to_string(),
                definitions: BTreeMap::from([(
                    "alpha".to_string(),
                    json!({"name": "alpha", "kind": "FunctionDeclaration", "line": 1}),
                )]),
            }],
            file_data: vec![FileDataRecord {
                file_path: "C:/repo/src/a.ts".to_string(),
                re_exports: vec![json!({"source": "./b", "line": 2})],
                class_methods: vec![json!({
                    "className": "Thing",
                    "name": "run",
                    "line": 3,
                })],
                local_operations: vec![json!({
                    "identity": "src/a.ts::outer#inner",
                    "name": "inner",
                    "containerName": "outer",
                    "line": 4,
                    "operationFamily": "format",
                    "domainTokens": ["z", "a"],
                })],
                dynamic_import_opacity: vec![
                    json!({"line": 5, "kind": "template", "prefix": "../routes"}),
                ],
                cjs_export_surface: Some(
                    json!({"exact": [{"name": "foo", "kind": "property", "line": 6}], "opaque": []}),
                ),
                cjs_require_opacity: vec![json!({"line": 7, "kind": "dynamic-require"})],
            }],
            parse_errors: 0,
            warnings: vec![],
            next_cache_entries: BTreeMap::new(),
            unresolved_internal_by_prefix: vec![CountEntry {
                key: "@/missing".to_string(),
                count: 2,
            }],
            prefix_examples: BTreeMap::from([(
                "@/missing".to_string(),
                "@/missing/foo".to_string(),
            )]),
            unresolved_internal_specifiers: vec!["@/missing/foo".to_string()],
            unresolved_internal_specifier_records: vec![json!({
                "specifier": "@/missing/foo",
                "consumerFile": "src/b.ts",
                "kind": "import",
                "typeOnly": false,
                "reason": "alias-miss",
            })],
            language_support: json!({"ts": {"enabled": true}}),
            total_uses: 1,
            unresolved_uses: 1,
            resolved_internal_uses: 1,
            resolved_generated_virtual_uses: 0,
            non_source_asset_uses: 0,
            external_uses: 0,
            dependency_import_consumers: vec![
                json!({"depRoot": "react", "fromSpec": "react", "file": "src/a.ts", "kind": "import"}),
            ],
            resolved_internal_edges: vec![
                json!({"from": "src/b.ts", "to": "src/a.ts", "kind": "import", "source": "./a", "typeOnly": false}),
            ],
            generated_consumer_blind_zones: vec![],
            generated_virtual_surfaces: vec![],
            generated_virtual_import_consumers: vec![],
            unresolved_internal_uses: 1,
            mdx_consumer_uses: 0,
            sfc_script_consumer_uses: 0,
            sfc_script_src_reachability_uses: 0,
            sfc_style_asset_reference_uses: 0,
            sfc_template_component_ref_uses: 0,
            sfc_global_component_registration_uses: 0,
            sfc_generated_component_manifest_uses: 0,
            sfc_framework_convention_component_uses: 0,
            sfc_style_asset_references: vec![],
            sfc_template_component_refs: vec![],
            sfc_global_component_registrations: vec![],
            sfc_generated_component_manifests: vec![],
            sfc_framework_convention_components: vec![],
            dead: vec![json!({"file": "src/a.ts", "symbol": "alpha", "line": 1})],
            truly_dead: vec![json!({"file": "src/a.ts", "symbol": "alpha", "line": 1})],
            dead_in_prod: vec![json!({"file": "src/a.ts", "symbol": "alpha", "line": 1})],
            dead_in_test: vec![],
            symbol_fan_in: vec![
                json!({"defFile": "src/a.ts", "symbol": "alpha", "count": 0, "kind": "FunctionDeclaration"}),
            ],
            fan_in_by_identity: json!({"src/a.ts::alpha": 0}),
            fan_in_by_identity_space: json!({"src/a.ts::alpha": {"value": 0, "type": 0, "broad": 0}}),
            namespace_re_export_diagnostics: vec![],
            any_contamination_facts: json!({
                "helperOwnersByIdentity": {"src/a.ts::alpha": []},
                "typeOwnersByIdentity": {},
            }),
            incremental: None,
        })?;

        assert_eq!(artifact["meta"]["tool"], TOOL_NAME);
        assert_eq!(artifact["meta"]["schemaVersion"], 3);
        assert_eq!(artifact["files"], 2);
        assert_eq!(artifact["totalDefs"], 1);
        assert_eq!(artifact["uses"]["unresolvedInternalRatio"], 0.5);
        assert_eq!(artifact["defIndex"]["src/a.ts"]["alpha"]["name"], "alpha");
        assert_eq!(artifact["deadProdList"][0]["symbol"], "alpha");
        assert_eq!(
            artifact["preWriteLocalOperationIndex"]["byOwnerFile"]["src/a.ts"][0]["domainTokens"]
                [0],
            "a"
        );
        Ok(())
    }

    #[test]
    fn parse_errors_are_visible() -> Result<()> {
        let artifact = build_symbol_graph_artifact(SymbolGraphRequest {
            schema_version: SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            files: vec!["C:/repo/src/bad.ts".to_string()],
            def_index: vec![],
            file_data: vec![],
            parse_errors: 1,
            warnings: vec![],
            next_cache_entries: BTreeMap::from([(
                "C:/repo/src/bad.ts".to_string(),
                json!({"parseError": true}),
            )]),
            unresolved_internal_by_prefix: vec![],
            prefix_examples: BTreeMap::new(),
            unresolved_internal_specifiers: vec![],
            unresolved_internal_specifier_records: vec![],
            language_support: json!({}),
            total_uses: 0,
            unresolved_uses: 0,
            resolved_internal_uses: 0,
            resolved_generated_virtual_uses: 0,
            non_source_asset_uses: 0,
            external_uses: 0,
            dependency_import_consumers: vec![],
            resolved_internal_edges: vec![],
            generated_consumer_blind_zones: vec![],
            generated_virtual_surfaces: vec![],
            generated_virtual_import_consumers: vec![],
            unresolved_internal_uses: 0,
            mdx_consumer_uses: 0,
            sfc_script_consumer_uses: 0,
            sfc_script_src_reachability_uses: 0,
            sfc_style_asset_reference_uses: 0,
            sfc_template_component_ref_uses: 0,
            sfc_global_component_registration_uses: 0,
            sfc_generated_component_manifest_uses: 0,
            sfc_framework_convention_component_uses: 0,
            sfc_style_asset_references: vec![],
            sfc_template_component_refs: vec![],
            sfc_global_component_registrations: vec![],
            sfc_generated_component_manifests: vec![],
            sfc_framework_convention_components: vec![],
            dead: vec![],
            truly_dead: vec![],
            dead_in_prod: vec![],
            dead_in_test: vec![],
            symbol_fan_in: vec![],
            fan_in_by_identity: json!({}),
            fan_in_by_identity_space: json!({}),
            namespace_re_export_diagnostics: vec![],
            any_contamination_facts: json!({}),
            incremental: None,
        })?;

        assert_eq!(artifact["meta"]["warnings"][0]["code"], "parse-errors");
        assert_eq!(artifact["filesWithParseErrors"][0], "src/bad.ts");
        Ok(())
    }

    #[test]
    fn rejects_unknown_schema() {
        let error = match build_symbol_graph_artifact(SymbolGraphRequest {
            schema_version: "future".to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            files: vec![],
            def_index: vec![],
            file_data: vec![],
            parse_errors: 0,
            warnings: vec![],
            next_cache_entries: BTreeMap::new(),
            unresolved_internal_by_prefix: vec![],
            prefix_examples: BTreeMap::new(),
            unresolved_internal_specifiers: vec![],
            unresolved_internal_specifier_records: vec![],
            language_support: json!({}),
            total_uses: 0,
            unresolved_uses: 0,
            resolved_internal_uses: 0,
            resolved_generated_virtual_uses: 0,
            non_source_asset_uses: 0,
            external_uses: 0,
            dependency_import_consumers: vec![],
            resolved_internal_edges: vec![],
            generated_consumer_blind_zones: vec![],
            generated_virtual_surfaces: vec![],
            generated_virtual_import_consumers: vec![],
            unresolved_internal_uses: 0,
            mdx_consumer_uses: 0,
            sfc_script_consumer_uses: 0,
            sfc_script_src_reachability_uses: 0,
            sfc_style_asset_reference_uses: 0,
            sfc_template_component_ref_uses: 0,
            sfc_global_component_registration_uses: 0,
            sfc_generated_component_manifest_uses: 0,
            sfc_framework_convention_component_uses: 0,
            sfc_style_asset_references: vec![],
            sfc_template_component_refs: vec![],
            sfc_global_component_registrations: vec![],
            sfc_generated_component_manifests: vec![],
            sfc_framework_convention_components: vec![],
            dead: vec![],
            truly_dead: vec![],
            dead_in_prod: vec![],
            dead_in_test: vec![],
            symbol_fan_in: vec![],
            fan_in_by_identity: json!({}),
            fan_in_by_identity_space: json!({}),
            namespace_re_export_diagnostics: vec![],
            any_contamination_facts: json!({}),
            incremental: None,
        }) {
            Ok(_) => panic!("schema should reject"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("unsupported schemaVersion"));
    }
}
