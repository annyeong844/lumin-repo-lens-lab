use super::protocol::{
    DeadCandidateInputs, DefinitionFileInput, FanInInputs, FileDataInput, SymbolGraphContext,
    SymbolGraphExtraction, SymbolGraphInputs, SymbolGraphSfcInputs,
};
use super::{normalize_slashes, SymbolGraphRequest, SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION};
use crate::source_use_assembly::SourceUseAssemblyRequest;
use anyhow::{bail, Result};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct DefinitionFile {
    pub(super) file_path: String,
    pub(super) definitions: BTreeMap<String, Value>,
}

#[derive(Debug)]
pub(super) struct FileDataRecord {
    pub(super) file_path: String,
    pub(super) py_dunder_all: Option<Vec<String>>,
    pub(super) re_exports: Vec<Value>,
    pub(super) class_methods: Vec<Value>,
    pub(super) local_operations: Vec<Value>,
    pub(super) type_escapes: Vec<Value>,
    pub(super) dynamic_import_opacity: Vec<Value>,
    pub(super) cjs_export_surface: Option<Value>,
    pub(super) cjs_require_opacity: Vec<Value>,
}

#[derive(Debug)]
pub(super) struct PreparedSymbolGraphRequest {
    pub(super) generated: String,
    pub(super) root: String,
    pub(super) include_tests: bool,
    pub(super) exclude: Vec<String>,
    pub(super) generated_artifacts_mode: String,
    pub(super) language_support: Value,
    pub(super) warnings: Vec<Value>,
    pub(super) incremental: Value,
    pub(super) path_table: Vec<String>,
    pub(super) files: Vec<String>,
    pub(super) def_index: Vec<DefinitionFile>,
    pub(super) file_data: Vec<FileDataRecord>,
    pub(super) parse_error_files: Vec<String>,
    pub(super) source_use_assembly: SourceUseAssemblyRequest,
    pub(super) fan_in_inputs: FanInInputs,
    pub(super) dead_candidate_inputs: DeadCandidateInputs,
    pub(super) sfc: SymbolGraphSfcInputs,
}

fn symbol_path_from_table(path_table: &[String], id: usize, field: &str) -> Result<String> {
    path_table
        .get(id)
        .filter(|path| !path.is_empty())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("symbol-graph-artifact: invalid {field} {id}"))
}

fn require_object_values(label: &str, values: &[Value]) -> Result<()> {
    if let Some((index, _)) = values
        .iter()
        .enumerate()
        .find(|(_, value)| !value.is_object())
    {
        bail!("symbol-graph-artifact: {label}[{index}] must be an object");
    }
    Ok(())
}

fn validate_extraction_facts(extraction: &SymbolGraphExtraction) -> Result<()> {
    for (file_index, file) in extraction.def_index.iter().enumerate() {
        for (name, definition) in &file.definitions {
            if !definition.is_object() {
                bail!(
                    "symbol-graph-artifact: extraction.defIndex[{file_index}].definitions.{name} must be an object"
                );
            }
        }
    }
    for (file_index, file) in extraction.file_data.iter().enumerate() {
        for (field, values) in [
            ("reExports", file.re_exports.as_slice()),
            ("classMethods", file.class_methods.as_slice()),
            ("localOperations", file.local_operations.as_slice()),
            ("typeEscapes", file.type_escapes.as_slice()),
            (
                "dynamicImportOpacity",
                file.dynamic_import_opacity.as_slice(),
            ),
            ("cjsRequireOpacity", file.cjs_require_opacity.as_slice()),
        ] {
            require_object_values(
                &format!("extraction.fileData[{file_index}].{field}"),
                values,
            )?;
        }
        if let Some(surface) = &file.cjs_export_surface {
            let object = surface.as_object().ok_or_else(|| {
                anyhow::anyhow!(
                    "symbol-graph-artifact: extraction.fileData[{file_index}].cjsExportSurface must be an object"
                )
            })?;
            for field in ["exact", "opaque"] {
                if let Some(values) = object.get(field) {
                    let values = values.as_array().ok_or_else(|| {
                        anyhow::anyhow!(
                            "symbol-graph-artifact: extraction.fileData[{file_index}].cjsExportSurface.{field} must be an array"
                        )
                    })?;
                    require_object_values(
                        &format!("extraction.fileData[{file_index}].cjsExportSurface.{field}"),
                        values,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn validate_source_use_facts(request: &SourceUseAssemblyRequest) -> Result<()> {
    for (record_index, record) in request.records.iter().enumerate() {
        if record
            .unresolved_evidence
            .as_ref()
            .is_some_and(|value| !value.is_object())
        {
            bail!(
                "symbol-graph-artifact: sourceUseAssembly.records[{record_index}].unresolvedEvidence must be an object"
            );
        }
        let Some(surface) = record.generated_virtual_surface.as_ref() else {
            continue;
        };
        let object = surface.as_object().ok_or_else(|| {
            anyhow::anyhow!(
                "symbol-graph-artifact: sourceUseAssembly.records[{record_index}].generatedVirtualSurface must be an object"
            )
        })?;
        let id = object.get("id").and_then(Value::as_str).unwrap_or_default();
        if id.is_empty() {
            bail!(
                "symbol-graph-artifact: sourceUseAssembly.records[{record_index}].generatedVirtualSurface.id must be non-empty"
            );
        }
        if let Some(exports) = object.get("exports") {
            let exports = exports.as_array().ok_or_else(|| {
                anyhow::anyhow!(
                    "symbol-graph-artifact: sourceUseAssembly.records[{record_index}].generatedVirtualSurface.exports must be an array"
                )
            })?;
            require_object_values(
                &format!(
                    "sourceUseAssembly.records[{record_index}].generatedVirtualSurface.exports"
                ),
                exports,
            )?;
        }
    }
    Ok(())
}

fn validate_sfc_facts(graph: &SymbolGraphInputs) -> Result<()> {
    for (index, component) in graph.sfc.framework_convention_components.iter().enumerate() {
        if component
            .path_prefix
            .as_ref()
            .is_some_and(|value| !value.is_boolean() && !value.is_string())
        {
            bail!(
                "symbol-graph-artifact: graph.sfc.frameworkConventionComponents[{index}].pathPrefix must be a boolean or string"
            );
        }
    }
    Ok(())
}

pub(super) fn prepare_symbol_graph_request(
    request: SymbolGraphRequest,
) -> Result<PreparedSymbolGraphRequest> {
    let SymbolGraphRequest {
        schema_version,
        context,
        extraction,
        source_use_assembly,
        graph,
    } = request;
    if schema_version != SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION {
        bail!("symbol-graph-artifact: unsupported schemaVersion '{schema_version}'");
    }

    let SymbolGraphContext {
        generated,
        root,
        include_tests,
        exclude,
        generated_artifacts_mode,
        language_support,
        warnings,
        incremental,
    } = context;
    if generated.is_empty() {
        bail!("symbol-graph-artifact: context.generated must be non-empty");
    }
    if root.is_empty() {
        bail!("symbol-graph-artifact: context.root must be non-empty");
    }
    if !matches!(
        generated_artifacts_mode.as_str(),
        "default" | "present" | "prepared"
    ) {
        bail!(
            "symbol-graph-artifact: unsupported generatedArtifactsMode '{generated_artifacts_mode}'"
        );
    }
    if !language_support.is_object() {
        bail!("symbol-graph-artifact: context.languageSupport must be an object");
    }
    if !incremental.is_null() && !incremental.is_object() {
        bail!("symbol-graph-artifact: context.incremental must be an object or null");
    }
    require_object_values("context.warnings", &warnings)?;
    validate_extraction_facts(&extraction)?;
    validate_source_use_facts(&source_use_assembly)?;
    validate_sfc_facts(&graph)?;
    if normalize_slashes(&source_use_assembly.root).trim_end_matches('/')
        != normalize_slashes(&root).trim_end_matches('/')
    {
        bail!("symbol-graph-artifact: sourceUseAssembly.root must match context.root");
    }

    let SymbolGraphExtraction {
        path_table,
        file_ids,
        def_index,
        file_data,
        parse_error_file_ids,
    } = extraction;
    let files = file_ids
        .into_iter()
        .map(|id| symbol_path_from_table(&path_table, id, "fileIds"))
        .collect::<Result<Vec<_>>>()?;
    let def_index = def_index
        .into_iter()
        .map(|input: DefinitionFileInput| {
            Ok(DefinitionFile {
                file_path: symbol_path_from_table(
                    &path_table,
                    input.file_path_id,
                    "defIndex.filePathId",
                )?,
                definitions: input.definitions,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let file_data = file_data
        .into_iter()
        .map(|input: FileDataInput| {
            Ok(FileDataRecord {
                file_path: symbol_path_from_table(
                    &path_table,
                    input.file_path_id,
                    "fileData.filePathId",
                )?,
                py_dunder_all: input.py_dunder_all,
                re_exports: input.re_exports,
                class_methods: input.class_methods,
                local_operations: input.local_operations,
                type_escapes: input.type_escapes,
                dynamic_import_opacity: input.dynamic_import_opacity,
                cjs_export_surface: input.cjs_export_surface,
                cjs_require_opacity: input.cjs_require_opacity,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let parse_error_files = parse_error_file_ids
        .into_iter()
        .map(|id| symbol_path_from_table(&path_table, id, "parseErrorFileIds"))
        .collect::<Result<Vec<_>>>()?;
    let SymbolGraphInputs {
        fan_in,
        dead_candidates,
        sfc,
    } = graph;

    Ok(PreparedSymbolGraphRequest {
        generated,
        root,
        include_tests,
        exclude,
        generated_artifacts_mode,
        language_support,
        warnings,
        incremental,
        path_table,
        files,
        def_index,
        file_data,
        parse_error_files,
        source_use_assembly,
        fan_in_inputs: fan_in,
        dead_candidate_inputs: dead_candidates,
        sfc,
    })
}
