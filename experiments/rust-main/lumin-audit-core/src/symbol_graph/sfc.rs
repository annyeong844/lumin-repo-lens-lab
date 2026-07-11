use super::protocol::{
    SfcFrameworkConventionComponentInput, SfcGeneratedComponentManifestInput,
    SfcGlobalComponentRegistrationInput, SfcStyleAssetReferenceInput, SfcTemplateComponentRefInput,
};
use super::{is_absolute_like_path, normalize_path_segments, normalize_slashes, rel_path};
use crate::source_use_assembly::SourceUseAssemblyResponse;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(super) struct SfcStyleAssetProjection {
    pub(super) references: Vec<Value>,
    pub(super) resolved_count: usize,
}

#[derive(Debug)]
pub(super) struct SfcTemplateComponentProjection {
    pub(super) refs: Vec<Value>,
    pub(super) count: usize,
}

#[derive(Debug)]
pub(super) struct SfcGlobalComponentRegistrationProjection {
    pub(super) registrations: Vec<Value>,
    pub(super) count: usize,
}

#[derive(Debug)]
pub(super) struct SfcGeneratedComponentManifestProjection {
    pub(super) manifests: Vec<Value>,
    pub(super) count: usize,
}

#[derive(Debug, Default)]
pub(super) struct SfcFrameworkConventionComponentProjection {
    pub(super) components: Vec<Value>,
    pub(super) count: usize,
}

pub(super) fn project_sfc_style_asset_references(
    root: &str,
    inputs: Vec<SfcStyleAssetReferenceInput>,
) -> SfcStyleAssetProjection {
    let mut resolved_count = 0;
    let mut references = Vec::with_capacity(inputs.len());
    for input in inputs {
        let mut object = Map::new();
        let resolved_file = input
            .resolved_file
            .filter(|path| !path.is_empty())
            .or_else(|| resolve_sfc_style_asset_target(&input.consumer_file, &input.from_spec));
        object.insert(
            "consumerFile".to_string(),
            json!(rel_path(root, &input.consumer_file)),
        );
        object.insert("fromSpec".to_string(), json!(input.from_spec));
        insert_optional_string(&mut object, "source", input.source);
        insert_optional_string(&mut object, "kind", input.kind);
        insert_optional_string(&mut object, "styleKind", input.style_kind);
        insert_optional_string(&mut object, "confidence", input.confidence);
        if let Some(resolved_file) = resolved_file {
            resolved_count += 1;
            object.insert("status".to_string(), json!("resolved"));
            object.insert(
                "resolvedFile".to_string(),
                json!(rel_path(root, &resolved_file)),
            );
        } else {
            object.insert("status".to_string(), json!("unresolved"));
            object.insert("reason".to_string(), json!("sfc-style-asset-unresolved"));
        }
        insert_optional_string(&mut object, "importSyntax", input.import_syntax);
        if let Some(line) = input.line {
            object.insert("line".to_string(), json!(line));
        }
        insert_optional_string(&mut object, "sfcBlockKind", input.sfc_block_kind);
        insert_optional_string(&mut object, "sfcLanguage", input.sfc_language);
        references.push(Value::Object(object));
    }

    SfcStyleAssetProjection {
        references,
        resolved_count,
    }
}

pub(super) fn resolve_sfc_style_asset_target(
    consumer_file: &str,
    from_spec: &str,
) -> Option<String> {
    if !is_relative_spec_text(from_spec) {
        return None;
    }
    let stripped = strip_style_asset_resource_query(from_spec);
    let parent = Path::new(consumer_file).parent()?;
    let target = parent.join(stripped);
    if target.is_file() {
        Some(path_to_string(target))
    } else {
        None
    }
}

fn is_relative_spec_text(spec: &str) -> bool {
    spec.starts_with("./") || spec.starts_with("../")
}

fn strip_style_asset_resource_query(spec: &str) -> &str {
    let query = spec.find('?');
    let hash = spec.find('#').filter(|index| *index > 0);
    match (query, hash) {
        (Some(query), Some(hash)) => &spec[..query.min(hash)],
        (Some(index), None) | (None, Some(index)) => &spec[..index],
        (None, None) => spec,
    }
}

fn path_to_string(path: PathBuf) -> String {
    normalize_path_segments(&path.to_string_lossy())
}

pub(super) fn source_use_resolved_target_map(
    source_use_assembly: &SourceUseAssemblyResponse,
) -> BTreeMap<String, String> {
    source_use_assembly
        .resolved_record_targets
        .iter()
        .map(|target| (target.record_id.clone(), target.resolved_file.clone()))
        .collect()
}

pub(super) fn source_use_external_record_set(
    source_use_assembly: &SourceUseAssemblyResponse,
) -> BTreeSet<String> {
    source_use_assembly
        .external_record_ids
        .iter()
        .cloned()
        .collect()
}

fn source_use_target_for_record(
    targets: &BTreeMap<String, String>,
    record_id: Option<&str>,
) -> Option<String> {
    record_id
        .filter(|record_id| !record_id.is_empty())
        .and_then(|record_id| targets.get(record_id))
        .filter(|target| !target.is_empty())
        .cloned()
}

fn source_use_record_is_external(targets: &BTreeSet<String>, record_id: Option<&str>) -> bool {
    record_id
        .filter(|record_id| !record_id.is_empty())
        .is_some_and(|record_id| targets.contains(record_id))
}

fn sfc_generated_manifest_status_and_reason(
    status: Option<String>,
    reason: Option<String>,
    source_use_record_id: Option<&str>,
    resolved_file: Option<&str>,
) -> (String, Option<String>) {
    if let Some(status) = status {
        return (status, reason);
    }
    if source_use_record_id.is_some() {
        return match resolved_file {
            Some(target) if is_js_family_target(target) => ("resolved".to_string(), reason),
            Some(_) => (
                "muted".to_string(),
                reason.or_else(|| {
                    Some("sfc-framework-generated-manifest-non-source-binding".to_string())
                }),
            ),
            None => (
                "unresolved".to_string(),
                reason.or_else(|| Some("sfc-framework-generated-manifest-unresolved".to_string())),
            ),
        };
    }
    (
        "unresolved".to_string(),
        reason.or_else(|| Some("sfc-framework-generated-manifest-unresolved".to_string())),
    )
}

fn is_js_family_target(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".d.ts")
        || lower.ends_with(".d.mts")
        || lower.ends_with(".d.cts")
        || matches!(
            Path::new(&lower)
                .extension()
                .and_then(|value| value.to_str()),
            Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "mts" | "cts")
        )
}

pub(super) fn project_sfc_template_component_refs(
    root: &str,
    inputs: Vec<SfcTemplateComponentRefInput>,
    source_use_resolved_targets: &BTreeMap<String, String>,
    source_use_external_record_ids: &BTreeSet<String>,
) -> SfcTemplateComponentProjection {
    let count = inputs.len();
    let mut refs = Vec::with_capacity(count);
    for input in inputs {
        let source_use_record_id = input.source_use_record_id;
        let has_source_use_record = source_use_record_id.is_some();
        let has_external_source_use_record = source_use_record_is_external(
            source_use_external_record_ids,
            source_use_record_id.as_deref(),
        );
        let resolved_file = input.resolved_file.or_else(|| {
            source_use_target_for_record(
                source_use_resolved_targets,
                source_use_record_id.as_deref(),
            )
        });
        let status = input.status.unwrap_or_else(|| {
            if has_external_source_use_record {
                "external".to_string()
            } else if has_source_use_record && resolved_file.is_some() {
                "resolved".to_string()
            } else {
                "unresolved".to_string()
            }
        });
        let reason = input.reason.or_else(|| {
            if status == "external" {
                Some("sfc-template-component-external-binding".to_string())
            } else {
                (status == "unresolved").then(|| "sfc-template-component-unresolved".to_string())
            }
        });
        let mut object = Map::new();
        object.insert(
            "consumerFile".to_string(),
            json!(rel_path(root, &input.consumer_file)),
        );
        insert_optional_string(&mut object, "tagName", input.tag_name);
        insert_optional_string(&mut object, "normalizedTagName", input.normalized_tag_name);
        insert_optional_string(&mut object, "bindingName", input.binding_name);
        insert_optional_string(&mut object, "bindingSource", input.binding_source);
        insert_optional_string(&mut object, "source", input.source);
        insert_optional_string(&mut object, "language", input.language);
        insert_optional_string(&mut object, "templateKind", input.template_kind);
        insert_optional_string(&mut object, "confidence", input.confidence);
        object.insert("eligibleForFanIn".to_string(), json!(false));
        object.insert("eligibleForSafeFix".to_string(), json!(false));
        object.insert("status".to_string(), json!(status));
        if let Some(resolved_file) = resolved_file.filter(|path| !path.is_empty()) {
            object.insert(
                "resolvedFile".to_string(),
                json!(rel_path(root, &resolved_file)),
            );
        }
        insert_optional_string(&mut object, "reason", reason);
        insert_optional_string(&mut object, "bindingKind", input.binding_kind);
        insert_optional_string(&mut object, "importedName", input.imported_name);
        insert_optional_string(&mut object, "memberName", input.member_name);
        if let Some(line) = input.line {
            object.insert("line".to_string(), json!(line));
        }
        insert_optional_string(&mut object, "sfcBlockKind", input.sfc_block_kind);
        refs.push(Value::Object(object));
    }

    SfcTemplateComponentProjection { refs, count }
}

pub(super) fn project_sfc_global_component_registrations(
    root: &str,
    inputs: Vec<SfcGlobalComponentRegistrationInput>,
    source_use_resolved_targets: &BTreeMap<String, String>,
    source_use_external_record_ids: &BTreeSet<String>,
) -> SfcGlobalComponentRegistrationProjection {
    let count = inputs.len();
    let mut registrations = Vec::with_capacity(count);
    for input in inputs {
        let source_use_record_id = input.source_use_record_id;
        let has_source_use_record = source_use_record_id.is_some();
        let has_external_source_use_record = source_use_record_is_external(
            source_use_external_record_ids,
            source_use_record_id.as_deref(),
        );
        let resolved_file = input.resolved_file.or_else(|| {
            source_use_target_for_record(
                source_use_resolved_targets,
                source_use_record_id.as_deref(),
            )
        });
        let status = input.status.unwrap_or_else(|| {
            if has_external_source_use_record {
                "external".to_string()
            } else if has_source_use_record && resolved_file.is_some() {
                "resolved".to_string()
            } else {
                "unresolved".to_string()
            }
        });
        let reason = input.reason.or_else(|| {
            if status == "external" {
                Some("sfc-global-component-external-binding".to_string())
            } else {
                (status == "unresolved").then(|| "sfc-global-component-unresolved".to_string())
            }
        });
        let mut object = Map::new();
        object.insert(
            "registrationFile".to_string(),
            json!(rel_path(root, &input.registration_file)),
        );
        insert_optional_string(&mut object, "framework", input.framework);
        insert_optional_string(&mut object, "api", input.api);
        insert_optional_string(&mut object, "componentName", input.component_name);
        if let Some(mut normalized_tag_names) = input.normalized_tag_names {
            normalized_tag_names.sort();
            object.insert(
                "normalizedTagNames".to_string(),
                json!(normalized_tag_names),
            );
        }
        insert_optional_string(&mut object, "bindingName", input.binding_name);
        if let Some(binding_source) = input.binding_source.filter(|value| !value.is_empty()) {
            object.insert("bindingSource".to_string(), json!(binding_source.clone()));
            object.insert("fromSpec".to_string(), json!(binding_source));
        } else {
            insert_optional_string(&mut object, "fromSpec", input.from_spec);
        }
        insert_optional_string(&mut object, "source", input.source);
        object.insert(
            "confidence".to_string(),
            json!(if status == "muted" {
                "muted-review"
            } else {
                "registration-review"
            }),
        );
        object.insert("eligibleForFanIn".to_string(), json!(false));
        object.insert("eligibleForSafeFix".to_string(), json!(false));
        object.insert("status".to_string(), json!(status));
        if let Some(resolved_file) = resolved_file.filter(|path| !path.is_empty()) {
            object.insert(
                "resolvedFile".to_string(),
                json!(rel_path(root, &resolved_file)),
            );
        }
        insert_optional_string(&mut object, "reason", reason);
        insert_optional_string(&mut object, "bindingKind", input.binding_kind);
        insert_optional_string(&mut object, "importedName", input.imported_name);
        insert_optional_string(&mut object, "factoryKind", input.factory_kind);
        insert_optional_string(&mut object, "ambiguityKey", input.ambiguity_key);
        if let Some(line) = input.line {
            object.insert("line".to_string(), json!(line));
        }
        registrations.push(Value::Object(object));
    }

    SfcGlobalComponentRegistrationProjection {
        registrations,
        count,
    }
}

pub(super) fn project_sfc_generated_component_manifests(
    root: &str,
    inputs: Vec<SfcGeneratedComponentManifestInput>,
    source_use_resolved_targets: &BTreeMap<String, String>,
    source_use_external_record_ids: &BTreeSet<String>,
) -> SfcGeneratedComponentManifestProjection {
    let count = inputs.len();
    let mut manifests = Vec::with_capacity(count);
    for input in inputs {
        let source_use_record_id = input.source_use_record_id;
        if source_use_record_is_external(
            source_use_external_record_ids,
            source_use_record_id.as_deref(),
        ) {
            continue;
        }
        let source_use_target = source_use_target_for_record(
            source_use_resolved_targets,
            source_use_record_id.as_deref(),
        );
        let resolved_file = input.resolved_file.or(source_use_target);
        let (status, reason) = sfc_generated_manifest_status_and_reason(
            input.status,
            input.reason,
            source_use_record_id.as_deref(),
            resolved_file.as_deref(),
        );
        let mut normalized_tag_names = input.normalized_tag_names;
        normalized_tag_names.sort();
        let mut object = Map::new();
        object.insert(
            "manifestFile".to_string(),
            json!(rel_path(root, &input.manifest_file)),
        );
        insert_optional_string(&mut object, "manifestKind", input.manifest_kind);
        insert_optional_string(&mut object, "componentName", input.component_name);
        object.insert(
            "normalizedTagNames".to_string(),
            json!(normalized_tag_names),
        );
        insert_optional_string(&mut object, "bindingSource", input.binding_source);
        insert_optional_string(&mut object, "fromSpec", input.from_spec);
        insert_optional_string(&mut object, "computedKeySource", input.computed_key_source);
        insert_optional_string(&mut object, "source", input.source);
        insert_optional_string(&mut object, "confidence", input.confidence);
        object.insert("eligibleForFanIn".to_string(), json!(false));
        object.insert("eligibleForSafeFix".to_string(), json!(false));
        object.insert("status".to_string(), json!(status));
        if let Some(resolved_file) = resolved_file.filter(|path| !path.is_empty()) {
            object.insert(
                "resolvedFile".to_string(),
                json!(rel_path(root, &resolved_file)),
            );
        }
        insert_optional_string(&mut object, "reason", reason);
        if let Some(line) = input.line {
            object.insert("line".to_string(), json!(line));
        }
        manifests.push(Value::Object(object));
    }

    SfcGeneratedComponentManifestProjection { manifests, count }
}

pub(super) fn project_sfc_framework_convention_components(
    root: &str,
    inputs: Vec<SfcFrameworkConventionComponentInput>,
) -> SfcFrameworkConventionComponentProjection {
    let count = inputs.len();
    let mut components = Vec::with_capacity(count);
    for input in inputs {
        let binding_source = input
            .binding_source
            .filter(|value| !value.is_empty())
            .map(|value| rel_path_if_absolute(root, &value));
        let from_spec = input
            .from_spec
            .filter(|value| !value.is_empty())
            .map(|value| rel_path_if_absolute(root, &value));
        let mut object = Map::new();
        insert_optional_string(&mut object, "framework", input.framework);
        insert_optional_string(&mut object, "conventionKind", input.convention_kind);
        if let Some(consumer_file) = input.consumer_file.filter(|value| !value.is_empty()) {
            object.insert(
                "consumerFile".to_string(),
                json!(rel_path(root, &consumer_file)),
            );
        }
        insert_optional_string(&mut object, "componentName", input.component_name);
        if let Some(mut normalized_tag_names) = input.normalized_tag_names {
            normalized_tag_names.sort();
            object.insert(
                "normalizedTagNames".to_string(),
                json!(normalized_tag_names),
            );
        }
        insert_optional_string(&mut object, "tagName", input.tag_name);
        insert_optional_string(&mut object, "normalizedTagName", input.normalized_tag_name);
        insert_optional_string(&mut object, "directiveName", input.directive_name);
        insert_optional_string(&mut object, "actionName", input.action_name);
        insert_optional_string(&mut object, "subscriptionName", input.subscription_name);
        insert_optional_string(&mut object, "storeName", input.store_name);
        insert_optional_string(&mut object, "macroName", input.macro_name);
        insert_optional_string(&mut object, "optionName", input.option_name);
        insert_optional_string(&mut object, "hookName", input.hook_name);
        insert_optional_string(&mut object, "configShape", input.config_shape);
        insert_optional_string(&mut object, "configProperty", input.config_property);
        insert_optional_string(&mut object, "extendsSource", input.extends_source);
        insert_optional_string(&mut object, "extendsSourceKind", input.extends_source_kind);
        insert_optional_string(&mut object, "moduleSource", input.module_source);
        insert_optional_string(&mut object, "moduleSourceKind", input.module_source_kind);
        if let Some(source_file) = input.source_file.filter(|value| !value.is_empty()) {
            object.insert(
                "sourceFile".to_string(),
                json!(rel_path(root, &source_file)),
            );
        }
        if let Some(config_file) = input.config_file.filter(|value| !value.is_empty()) {
            object.insert(
                "configFile".to_string(),
                json!(rel_path(root, &config_file)),
            );
        }
        insert_optional_string(&mut object, "componentDir", input.component_dir);
        if let Some(resolved_dir) = input.resolved_dir.filter(|value| !value.is_empty()) {
            object.insert(
                "resolvedDir".to_string(),
                json!(rel_path(root, &resolved_dir)),
            );
        }
        insert_optional_string(&mut object, "prefix", input.prefix);
        if let Some(path_prefix) = input
            .path_prefix
            .filter(|value| value.is_boolean() || value.is_string())
        {
            object.insert("pathPrefix".to_string(), path_prefix);
        }
        if let Some(global) = input.global {
            object.insert("global".to_string(), json!(global));
        }
        if let Some(manifest_file) = input.manifest_file.filter(|value| !value.is_empty()) {
            object.insert(
                "manifestFile".to_string(),
                json!(rel_path(root, &manifest_file)),
            );
        }
        insert_optional_string(&mut object, "manifestKind", input.manifest_kind);
        if let Some(resolved_file) = input.resolved_file.filter(|value| !value.is_empty()) {
            object.insert(
                "resolvedFile".to_string(),
                json!(rel_path(root, &resolved_file)),
            );
        }
        insert_optional_string(&mut object, "pluginName", input.plugin_name);
        insert_optional_string(&mut object, "bindingName", input.binding_name);
        if let Some(binding_source) = binding_source {
            object.insert("bindingSource".to_string(), json!(binding_source.clone()));
            object.insert("fromSpec".to_string(), json!(binding_source));
        }
        if let Some(from_spec) = from_spec {
            object.insert("fromSpec".to_string(), json!(from_spec));
        }
        insert_optional_string(&mut object, "source", input.source);
        insert_optional_string(&mut object, "confidence", input.confidence);
        object.insert("eligibleForFanIn".to_string(), json!(false));
        object.insert("eligibleForSafeFix".to_string(), json!(false));
        object.insert(
            "status".to_string(),
            json!(input.status.unwrap_or_else(|| "muted".to_string())),
        );
        insert_optional_string(&mut object, "reason", input.reason);
        insert_optional_string(&mut object, "bindingKind", input.binding_kind);
        insert_optional_string(&mut object, "importedName", input.imported_name);
        if let Some(component_path_segments) = input.component_path_segments {
            object.insert(
                "componentPathSegments".to_string(),
                json!(component_path_segments),
            );
        }
        insert_optional_string(&mut object, "sfcBlockKind", input.sfc_block_kind);
        if let Some(line) = input.line {
            object.insert("line".to_string(), json!(line));
        }
        components.push(Value::Object(object));
    }

    SfcFrameworkConventionComponentProjection { components, count }
}

fn insert_optional_string(object: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        object.insert(key.to_string(), json!(value));
    }
}

fn rel_path_if_absolute(root: &str, value: &str) -> String {
    let normalized = normalize_slashes(value);
    if is_absolute_like_path(&normalized) {
        rel_path(root, &normalized)
    } else {
        normalized
    }
}
