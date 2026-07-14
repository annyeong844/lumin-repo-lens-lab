mod ast_support;
mod cjs;
mod code_shape;
mod definitions;
mod dynamic_imports;
mod function_signature;
mod inline_patterns;
mod module_uses;
mod named_imports;
mod parser_support;
mod protocol;
mod shape_hash;
mod surfaces;
mod type_escape;
mod vue_global_components;

use anyhow::{bail, Result};
use oxc_allocator::Allocator;
use oxc_span::SourceType;
use rayon::{prelude::*, ThreadPoolBuilder};
use std::fs;

use crate::relative_source_resolver::RelativeSourceResolver;
use ast_support::{
    assignment_target_identifier_name, assignment_target_member_object_identifier_name,
    binding_identifier_name, binding_identifier_name_ref, definition_id,
    expression_identifier_name, expression_member_object_identifier_name, is_type_only,
    member_object_identifier_name, method_kind_text, module_export_identifier_name,
    property_key_name, simple_assignment_target_identifier_name,
    simple_assignment_target_member_object_identifier_name, static_member_property_name,
    ts_module_name, variable_kind_text, visibility_text,
};
use cjs::{collect_cjs_export_surface, collect_cjs_require_uses};
pub(crate) use code_shape::normalize_code_shape;
use definitions::{
    collect_export_definitions, collect_exported_identity_ranges,
    collect_top_level_declaration_targets, ExportedIdentityRange,
};
use dynamic_imports::{collect_dynamic_import_uses, collect_import_meta_glob_uses};
use function_signature::collect_function_signature_facts;
pub(crate) use function_signature::FunctionSignatureFact;
use inline_patterns::collect_inline_pattern_facts;
pub(crate) use inline_patterns::{
    InlinePatternOccurrence, MAX_CATCH_STATEMENTS, NORMALIZER_VERSION as INLINE_NORMALIZER_VERSION,
};
use module_uses::{annotate_relative_resolutions, collect_imports, collect_re_exports};
use named_imports::collect_named_import_precision_uses;
use parser_support::{line_count, line_for_span, line_starts, parse_program, source_type_for_path};
pub use protocol::{
    CjsExportExactRecord, CjsExportOpaqueRecord, CjsExportSurface, CjsRequireOpacityRecord,
    ClassMethodRecord, DefinitionRecord, DynamicImportOpacityRecord, JsTsExtractFileResult,
    JsTsExtractInputFile, JsTsExtractRequest, JsTsExtractResponse, ReExportRecord,
    TypeEscapeRecord, UseRecord, VueGlobalComponentRegistration,
};
use shape_hash::collect_shape_hash_facts;
use surfaces::{collect_class_method_surface, collect_pre_write_local_operation_surface};
use type_escape::collect_type_escapes;
use vue_global_components::collect_vue_global_component_registrations;

pub const JS_TS_EXTRACT_REQUEST_SCHEMA_VERSION: &str = "lumin-js-ts-extract-request.v1";
pub const JS_TS_EXTRACT_RESPONSE_SCHEMA_VERSION: &str = "lumin-js-ts-extract-response.v1";
const JS_TS_EXTRACT_WORKER_STACK_BYTES: usize = 4 * 1024 * 1024;

pub(crate) fn normalize_shape_type_literal(type_literal: &str) -> serde_json::Value {
    if function_signature::looks_like_type_literal(type_literal) {
        return function_signature::normalize_type_literal(type_literal);
    }
    let literal = type_literal.trim().trim_end_matches(';').trim_end();
    if literal.is_empty() {
        return serde_json::json!({
            "typeLiteral": type_literal,
            "ok": false,
            "reason": "empty-shape-type-literal",
        });
    }
    let source = format!("export type __IntentShape = {literal};\n");
    let allocator = Allocator::default();
    let parsed = match parse_program(&allocator, &source, SourceType::ts()) {
        Ok(parsed) => parsed,
        Err(error) => {
            return serde_json::json!({
                "typeLiteral": type_literal,
                "ok": false,
                "reason": "parse-error",
                "message": error.to_string(),
            });
        }
    };
    let (facts, diagnostics) = collect_shape_hash_facts(
        &parsed.program,
        &source,
        "__intent_shape.ts",
        &line_starts(&source),
    );
    if facts.len() != 1 {
        return serde_json::json!({
            "typeLiteral": type_literal,
            "ok": false,
            "reason": diagnostics.first()
                .and_then(|value| value.get("code"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unsupported-intent-shape"),
        });
    }
    let fact = &facts[0];
    let shape_kind = fact
        .get("shapeKind")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("object");
    let evidence_count = if shape_kind == "literal-union" {
        fact.get("literals")
    } else {
        fact.get("fields")
    }
    .and_then(serde_json::Value::as_array)
    .map_or(0, Vec::len);
    serde_json::json!({
        "typeLiteral": type_literal,
        "ok": true,
        "hash": fact["hash"],
        "shapeKind": shape_kind,
        "evidenceCount": evidence_count,
    })
}

pub fn build_js_ts_extract_response(request: JsTsExtractRequest) -> Result<JsTsExtractResponse> {
    if request.schema_version != JS_TS_EXTRACT_REQUEST_SCHEMA_VERSION {
        bail!(
            "js-ts-extract-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let thread_count = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1);
    let pool = ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .stack_size(JS_TS_EXTRACT_WORKER_STACK_BYTES)
        .build()?;
    let relative_resolver = RelativeSourceResolver::from_paths(request.source_files);
    let files = pool.install(|| {
        request
            .files
            .into_par_iter()
            .map(|input| extract_file_or_error(input, &relative_resolver))
            .collect()
    });
    Ok(JsTsExtractResponse {
        schema_version: JS_TS_EXTRACT_RESPONSE_SCHEMA_VERSION,
        files,
    })
}

fn extract_file_or_error(
    input: JsTsExtractInputFile,
    relative_resolver: &RelativeSourceResolver,
) -> JsTsExtractFileResult {
    let artifact_file_path = input
        .artifact_file_path
        .clone()
        .unwrap_or_else(|| input.file_path.clone());
    let source = match input.source {
        Some(source) => source,
        None => match fs::read_to_string(&input.file_path) {
            Ok(source) => source,
            Err(error) => {
                return empty_file_result(
                    input.file_path,
                    0,
                    Some(format!("failed to read source: {error}")),
                );
            }
        },
    };
    let loc = line_count(&source);
    match extract_file(
        &input.file_path,
        &source,
        &artifact_file_path,
        relative_resolver,
    ) {
        Ok(mut result) => {
            result.loc = loc;
            result
        }
        Err(error) => empty_file_result(input.file_path, loc, Some(error.to_string())),
    }
}

fn empty_file_result(
    file_path: String,
    loc: usize,
    error: Option<String>,
) -> JsTsExtractFileResult {
    JsTsExtractFileResult {
        file_path,
        defs: Vec::new(),
        uses: Vec::new(),
        re_exports: Vec::new(),
        class_methods: Vec::new(),
        local_operations: Vec::new(),
        type_escapes: Vec::new(),
        global_component_registrations: Vec::new(),
        function_signature_facts: Vec::new(),
        inline_pattern_occurrences: Vec::new(),
        inline_pattern_diagnostics: Vec::new(),
        shape_facts: Vec::new(),
        shape_diagnostics: Vec::new(),
        dynamic_import_opacity: Vec::new(),
        cjs_require_opacity: Vec::new(),
        cjs_export_surface: None,
        loc,
        error,
    }
}

fn extract_file(
    file_path: &str,
    source: &str,
    artifact_file_path: &str,
    relative_resolver: &RelativeSourceResolver,
) -> Result<JsTsExtractFileResult> {
    let allocator = Allocator::default();
    let source_type = source_type_for_path(file_path);
    let parsed = parse_program(&allocator, source, source_type)?;
    let line_starts = line_starts(source);
    let mut defs = Vec::new();
    let mut uses = Vec::new();
    let mut re_exports = Vec::new();
    let local_declarations = collect_top_level_declaration_targets(&parsed.program);
    let exported_identity_ranges =
        collect_exported_identity_ranges(&parsed.program, artifact_file_path, &local_declarations);

    for statement in &parsed.program.body {
        collect_export_definitions(
            statement,
            &mut defs,
            &line_starts,
            artifact_file_path,
            &local_declarations,
        );
        collect_re_exports(statement, &mut re_exports, &mut uses, &line_starts);
        collect_imports(statement, &mut uses, &line_starts);
    }
    uses.extend(collect_import_meta_glob_uses(&parsed.program, &line_starts));
    let dynamic_imports = collect_dynamic_import_uses(&parsed.program, &line_starts);
    uses.extend(dynamic_imports.uses);
    let cjs_requires = collect_cjs_require_uses(&parsed.program, &line_starts);
    let cjs_export_surface = collect_cjs_export_surface(&parsed.program, &line_starts);
    uses.extend(cjs_requires.uses);
    uses.extend(collect_named_import_precision_uses(
        &parsed.program,
        &line_starts,
    ));
    annotate_relative_resolutions(file_path, &mut uses, relative_resolver);

    let class_methods =
        collect_class_method_surface(&parsed.program, &line_starts, artifact_file_path);
    let local_operations = collect_pre_write_local_operation_surface(
        &parsed.program,
        &line_starts,
        artifact_file_path,
    );
    let type_escapes = collect_type_escapes(
        &parsed.program,
        &parsed.program.comments,
        source,
        artifact_file_path,
        &line_starts,
        &exported_identity_ranges,
    );
    let global_component_registrations =
        collect_vue_global_component_registrations(&parsed.program, file_path, &line_starts);
    let (shape_facts, shape_diagnostics) =
        collect_shape_hash_facts(&parsed.program, source, artifact_file_path, &line_starts);
    let function_signature_facts =
        collect_function_signature_facts(&parsed.program, source, artifact_file_path, &line_starts);
    let inline_patterns =
        collect_inline_pattern_facts(&parsed.program, source, artifact_file_path, &line_starts);

    Ok(JsTsExtractFileResult {
        file_path: file_path.to_string(),
        defs,
        uses,
        re_exports,
        class_methods,
        local_operations,
        type_escapes,
        global_component_registrations,
        function_signature_facts,
        inline_pattern_occurrences: inline_patterns.occurrences,
        inline_pattern_diagnostics: inline_patterns.diagnostics,
        shape_facts,
        shape_diagnostics,
        dynamic_import_opacity: dynamic_imports.opacity,
        cjs_require_opacity: cjs_requires.opacity,
        cjs_export_surface,
        loc: line_count(source),
        error: None,
    })
}

#[cfg(test)]
mod tests;
