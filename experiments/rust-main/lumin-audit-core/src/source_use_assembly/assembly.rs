use anyhow::{bail, Result};

use crate::relative_source_resolver::{normalize_path_text, RelativeSourceResolver};

use super::input::{
    normalize_record, record_inputs_from_request, source_files_from_request,
    SourceUseAssemblyTables,
};
use super::namespace::NamespaceReExportResolver;
use super::protocol::{
    SourceUseAssemblyRequest, SourceUseAssemblyResponse, SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
};

mod generated;
mod glob_record;
mod internal;
mod record;
mod support;
mod terminal;

use record::assemble_record;
use support::{AssemblyState, EMBEDDED_BUILD_OPTIONS, STANDALONE_BUILD_OPTIONS};

pub fn build_source_use_assembly_response(
    request: SourceUseAssemblyRequest,
) -> Result<SourceUseAssemblyResponse> {
    build_source_use_assembly_response_with_options(request, STANDALONE_BUILD_OPTIONS, None)
}

pub fn build_embedded_source_use_assembly_response(
    request: SourceUseAssemblyRequest,
) -> Result<SourceUseAssemblyResponse> {
    build_source_use_assembly_response_with_options(request, EMBEDDED_BUILD_OPTIONS, None)
}

pub(crate) fn build_embedded_source_use_assembly_response_with_path_table(
    request: SourceUseAssemblyRequest,
    inherited_path_table: &[String],
) -> Result<SourceUseAssemblyResponse> {
    build_source_use_assembly_response_with_options(
        request,
        EMBEDDED_BUILD_OPTIONS,
        Some(inherited_path_table),
    )
}

fn build_source_use_assembly_response_with_options(
    request: SourceUseAssemblyRequest,
    options: support::SourceUseAssemblyBuildOptions,
    inherited_path_table: Option<&[String]>,
) -> Result<SourceUseAssemblyResponse> {
    if request.schema_version != SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION {
        bail!(
            "source-use-assembly-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    let records = record_inputs_from_request(
        request.records,
        request.record_row_fields,
        request.record_rows,
    )?;
    let record_count = records.len();
    let root = normalize_path_text(&request.root);
    let path_table = if request.path_table.is_empty() {
        inherited_path_table.unwrap_or(&request.path_table)
    } else {
        &request.path_table
    };
    let source_files =
        source_files_from_request(path_table, request.source_files, request.source_file_ids)?;
    let resolver = RelativeSourceResolver::from_rooted_paths(&root, source_files);
    let tables = SourceUseAssemblyTables {
        path_table,
        kind_table: &request.kind_table,
        resolver_stage_table: &request.resolver_stage_table,
        consumer_source_table: &request.consumer_source_table,
        specifier_table: &request.specifier_table,
        name_table: &request.name_table,
    };
    let records = records
        .into_iter()
        .enumerate()
        .map(|(index, record)| normalize_record(record, index, tables))
        .collect::<Result<Vec<_>>>()?;
    let mut namespace_resolver =
        NamespaceReExportResolver::new(request.namespace_re_exports, request.named_re_exports);
    namespace_resolver.extend_from_records(&resolver, &records);
    let mut state = AssemblyState::new(
        request.root,
        root,
        resolver,
        namespace_resolver,
        request.import_meta_glob_cap,
        options,
        record_count,
    );

    for record in records {
        assemble_record(&mut state, record);
    }

    Ok(state.into_response())
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
