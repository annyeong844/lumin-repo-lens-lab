use anyhow::{bail, Result};
use serde_json::Value;

use super::protocol::SourceUseAssemblyRecordInput;

#[derive(Debug)]
pub(super) struct SourceUseAssemblyRecord {
    pub(super) record_id: String,
    pub(super) consumer_file: String,
    pub(super) resolved_file: Option<String>,
    pub(super) from_spec: Option<String>,
    pub(super) name: Option<String>,
    pub(super) member_name: Option<String>,
    pub(super) kind: Option<String>,
    pub(super) type_only: bool,
    pub(super) type_only_present: bool,
    pub(super) line: Option<u64>,
    pub(super) sfc_language: Option<String>,
    pub(super) resolver_stage: Option<String>,
    pub(super) consumer_source: Option<String>,
    pub(super) unresolved_evidence: Option<Value>,
    pub(super) generated_virtual_surface: Option<Value>,
}

#[derive(Debug, Default)]
struct SourceUseAssemblyRecordInputBuilder {
    record_id: Option<String>,
    consumer_file: Option<String>,
    consumer_file_id: Option<usize>,
    resolved_file: Option<String>,
    resolved_file_id: Option<usize>,
    from_spec: Option<String>,
    from_spec_id: Option<usize>,
    name: Option<String>,
    name_id: Option<usize>,
    member_name: Option<String>,
    member_name_id: Option<usize>,
    kind: Option<String>,
    kind_id: Option<usize>,
    type_only: bool,
    type_only_present: bool,
    line: Option<u64>,
    sfc_language: Option<String>,
    resolver_stage: Option<String>,
    resolver_stage_id: Option<usize>,
    consumer_source: Option<String>,
    consumer_source_id: Option<usize>,
    unresolved_evidence: Option<Value>,
    generated_virtual_surface: Option<Value>,
}

impl SourceUseAssemblyRecordInputBuilder {
    fn build(self) -> SourceUseAssemblyRecordInput {
        SourceUseAssemblyRecordInput {
            record_id: self.record_id,
            consumer_file: self.consumer_file,
            consumer_file_id: self.consumer_file_id,
            resolved_file: self.resolved_file,
            resolved_file_id: self.resolved_file_id,
            from_spec: self.from_spec,
            from_spec_id: self.from_spec_id,
            name: self.name,
            name_id: self.name_id,
            member_name: self.member_name,
            member_name_id: self.member_name_id,
            kind: self.kind,
            kind_id: self.kind_id,
            type_only: self.type_only,
            type_only_present: self.type_only_present,
            line: self.line,
            sfc_language: self.sfc_language,
            resolver_stage: self.resolver_stage,
            resolver_stage_id: self.resolver_stage_id,
            consumer_source: self.consumer_source,
            consumer_source_id: self.consumer_source_id,
            unresolved_evidence: self.unresolved_evidence,
            generated_virtual_surface: self.generated_virtual_surface,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct SourceUseAssemblyTables<'a> {
    pub(super) path_table: &'a [String],
    pub(super) kind_table: &'a [String],
    pub(super) resolver_stage_table: &'a [String],
    pub(super) consumer_source_table: &'a [String],
    pub(super) specifier_table: &'a [String],
    pub(super) name_table: &'a [String],
}

fn path_from_table(path_table: &[String], id: usize, field: &str) -> Result<String> {
    path_table
        .get(id)
        .filter(|path| !path.is_empty())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("source-use-assembly-artifact: invalid {field} {id}"))
}

fn string_from_table(table: &[String], id: usize, field: &str) -> Result<String> {
    table
        .get(id)
        .filter(|value| !value.is_empty())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("source-use-assembly-artifact: invalid {field} {id}"))
}

pub(super) fn source_files_from_request(
    path_table: &[String],
    source_files: Vec<String>,
    source_file_ids: Vec<usize>,
) -> Result<Vec<String>> {
    let mut files = source_files;
    for id in source_file_ids {
        files.push(path_from_table(path_table, id, "sourceFileIds")?);
    }
    Ok(files)
}

fn row_string(value: &Value, field: &str) -> Result<Option<String>> {
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_str()
        .map(|text| (!text.is_empty()).then(|| text.to_string()))
        .ok_or_else(|| anyhow::anyhow!("source-use-assembly-artifact: invalid recordRows {field}"))
}

fn row_usize(value: &Value, field: &str) -> Result<Option<usize>> {
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_u64()
        .map(|number| Some(number as usize))
        .ok_or_else(|| anyhow::anyhow!("source-use-assembly-artifact: invalid recordRows {field}"))
}

fn row_u64(value: &Value, field: &str) -> Result<Option<u64>> {
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_u64()
        .map(Some)
        .ok_or_else(|| anyhow::anyhow!("source-use-assembly-artifact: invalid recordRows {field}"))
}

fn row_bool(value: &Value, field: &str) -> Result<Option<bool>> {
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_bool()
        .map(Some)
        .ok_or_else(|| anyhow::anyhow!("source-use-assembly-artifact: invalid recordRows {field}"))
}

fn record_from_row(fields: &[String], row: Vec<Value>) -> Result<SourceUseAssemblyRecordInput> {
    let mut builder = SourceUseAssemblyRecordInputBuilder::default();
    for (index, value) in row.iter().enumerate() {
        if value.is_null() {
            continue;
        }
        let field = fields.get(index).ok_or_else(|| {
            anyhow::anyhow!("source-use-assembly-artifact: recordRows has too many columns")
        })?;
        match field.as_str() {
            "recordId" => builder.record_id = row_string(value, field)?,
            "consumerFile" => builder.consumer_file = row_string(value, field)?,
            "consumerFileId" => builder.consumer_file_id = row_usize(value, field)?,
            "resolvedFile" => builder.resolved_file = row_string(value, field)?,
            "resolvedFileId" => builder.resolved_file_id = row_usize(value, field)?,
            "fromSpec" => builder.from_spec = row_string(value, field)?,
            "fromSpecId" => builder.from_spec_id = row_usize(value, field)?,
            "name" => builder.name = row_string(value, field)?,
            "nameId" => builder.name_id = row_usize(value, field)?,
            "memberName" => builder.member_name = row_string(value, field)?,
            "memberNameId" => builder.member_name_id = row_usize(value, field)?,
            "kind" => builder.kind = row_string(value, field)?,
            "kindId" => builder.kind_id = row_usize(value, field)?,
            "typeOnly" => {
                if let Some(type_only) = row_bool(value, field)? {
                    builder.type_only = type_only;
                }
            }
            "typeOnlyPresent" => {
                if let Some(type_only_present) = row_bool(value, field)? {
                    builder.type_only_present = type_only_present;
                }
            }
            "typeOnlyState" => match row_usize(value, field)? {
                Some(1) => {
                    builder.type_only = false;
                    builder.type_only_present = true;
                }
                Some(2) => {
                    builder.type_only = true;
                    builder.type_only_present = true;
                }
                Some(0) | None => {
                    builder.type_only = false;
                    builder.type_only_present = false;
                }
                Some(_) => bail!("source-use-assembly-artifact: invalid recordRows {field}"),
            },
            "line" => builder.line = row_u64(value, field)?,
            "sfcLanguage" => builder.sfc_language = row_string(value, field)?,
            "resolverStage" => builder.resolver_stage = row_string(value, field)?,
            "resolverStageId" => builder.resolver_stage_id = row_usize(value, field)?,
            "consumerSource" => builder.consumer_source = row_string(value, field)?,
            "consumerSourceId" => builder.consumer_source_id = row_usize(value, field)?,
            "unresolvedEvidence" => builder.unresolved_evidence = Some(value.clone()),
            "generatedVirtualSurface" => builder.generated_virtual_surface = Some(value.clone()),
            _ => bail!("source-use-assembly-artifact: unsupported recordRows field '{field}'"),
        }
    }
    Ok(builder.build())
}

pub(super) fn record_inputs_from_request(
    records: Vec<SourceUseAssemblyRecordInput>,
    record_row_fields: Vec<String>,
    record_rows: Vec<Vec<Value>>,
) -> Result<Vec<SourceUseAssemblyRecordInput>> {
    if record_rows.is_empty() {
        return Ok(records);
    }
    if record_row_fields.is_empty() {
        bail!("source-use-assembly-artifact: recordRows requires recordRowFields");
    }
    let mut inputs = records;
    for row in record_rows {
        inputs.push(record_from_row(&record_row_fields, row)?);
    }
    Ok(inputs)
}

pub(super) fn normalize_record(
    input: SourceUseAssemblyRecordInput,
    index: usize,
    tables: SourceUseAssemblyTables<'_>,
) -> Result<SourceUseAssemblyRecord> {
    let consumer_file = match (input.consumer_file, input.consumer_file_id) {
        (Some(path), _) if !path.is_empty() => path,
        (_, Some(id)) => path_from_table(tables.path_table, id, "consumerFileId")?,
        _ => bail!(
            "source-use-assembly-artifact: record '{}' missing consumerFile",
            input
                .record_id
                .as_deref()
                .filter(|record_id| !record_id.is_empty())
                .unwrap_or("<synthetic>")
        ),
    };
    let resolved_file = match (input.resolved_file, input.resolved_file_id) {
        (Some(path), _) if !path.is_empty() => Some(path),
        (_, Some(id)) => Some(path_from_table(tables.path_table, id, "resolvedFileId")?),
        _ => None,
    };
    let kind = match (input.kind, input.kind_id) {
        (Some(kind), _) if !kind.is_empty() => Some(kind),
        (_, Some(id)) => Some(string_from_table(tables.kind_table, id, "kindId")?),
        _ => None,
    };
    let resolver_stage = match (input.resolver_stage, input.resolver_stage_id) {
        (Some(stage), _) if !stage.is_empty() => Some(stage),
        (_, Some(id)) => Some(string_from_table(
            tables.resolver_stage_table,
            id,
            "resolverStageId",
        )?),
        _ => None,
    };
    let consumer_source = match (input.consumer_source, input.consumer_source_id) {
        (Some(source), _) if !source.is_empty() => Some(source),
        (_, Some(id)) => Some(string_from_table(
            tables.consumer_source_table,
            id,
            "consumerSourceId",
        )?),
        _ => None,
    };
    let from_spec = match (input.from_spec, input.from_spec_id) {
        (Some(spec), _) if !spec.is_empty() => Some(spec),
        (_, Some(id)) => Some(string_from_table(tables.specifier_table, id, "fromSpecId")?),
        _ => None,
    };
    let name = match (input.name, input.name_id) {
        (Some(name), _) if !name.is_empty() => Some(name),
        (_, Some(id)) => Some(string_from_table(tables.name_table, id, "nameId")?),
        _ => None,
    };
    let member_name = match (input.member_name, input.member_name_id) {
        (Some(name), _) if !name.is_empty() => Some(name),
        (_, Some(id)) => Some(string_from_table(tables.name_table, id, "memberNameId")?),
        _ => None,
    };

    Ok(SourceUseAssemblyRecord {
        record_id: input
            .record_id
            .filter(|record_id| !record_id.is_empty())
            .unwrap_or_else(|| format!("r{index}")),
        consumer_file,
        resolved_file,
        from_spec,
        name,
        member_name,
        kind,
        type_only: input.type_only,
        type_only_present: input.type_only_present,
        line: input.line,
        sfc_language: input.sfc_language,
        resolver_stage,
        consumer_source,
        unresolved_evidence: input.unresolved_evidence,
        generated_virtual_surface: input.generated_virtual_surface,
    })
}
