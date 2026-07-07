use anyhow::{anyhow, bail, Result};
use lumin_rust_common::sha256_text;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    AssignmentExpression, BindingIdentifier, BindingPattern, Class, ClassElement, Comment,
    Declaration, ExportAllDeclaration, ExportDefaultDeclaration, ExportDefaultDeclarationKind,
    ExportNamedDeclaration, Expression, FormalParameterRest, IdentifierReference, IfStatement,
    ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind, LogicalExpression,
    MemberExpression, MethodDefinition, MethodDefinitionKind, ModuleExportName, Program,
    PropertyDefinition, PropertyKey, SimpleAssignmentTarget, Statement, TSAccessibility,
    TSAnyKeyword, TSAsExpression, TSIndexSignature, TSType, TSTypeAssertion, TSTypeParameter,
    UnaryExpression, UpdateExpression, VariableDeclaration, VariableDeclarationKind,
};
use oxc_ast_visit::{walk, Visit};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use oxc_syntax::operator::UnaryOperator;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const JS_TS_EXTRACT_REQUEST_SCHEMA_VERSION: &str = "lumin-js-ts-extract-request.v1";
pub const JS_TS_EXTRACT_RESPONSE_SCHEMA_VERSION: &str = "lumin-js-ts-extract-response.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsTsExtractRequest {
    pub schema_version: String,
    #[serde(default)]
    pub files: Vec<JsTsExtractInputFile>,
    #[serde(default)]
    pub source_files: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsTsExtractInputFile {
    pub file_path: String,
    pub artifact_file_path: Option<String>,
    pub source: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsTsExtractResponse {
    pub schema_version: &'static str,
    pub files: Vec<JsTsExtractFileResult>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsTsExtractFileResult {
    pub file_path: String,
    pub defs: Vec<DefinitionRecord>,
    pub uses: Vec<UseRecord>,
    pub re_exports: Vec<ReExportRecord>,
    pub class_methods: Vec<ClassMethodRecord>,
    pub local_operations: Vec<serde_json::Value>,
    pub type_escapes: Vec<TypeEscapeRecord>,
    pub loc: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionRecord {
    pub name: String,
    pub kind: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UseRecord {
    pub from_spec: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_name: Option<String>,
    pub kind: String,
    pub type_only: bool,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_name: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    pub degraded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver_stage: Option<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReExportRecord {
    pub source: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassMethodRecord {
    pub identity: String,
    pub owner_file: String,
    pub class_name: String,
    pub name: String,
    pub method_name: String,
    pub kind: &'static str,
    pub member_kind: String,
    pub visibility: String,
    pub r#static: bool,
    pub computed: bool,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeEscapeRecord {
    pub file: String,
    pub line: usize,
    pub escape_kind: &'static str,
    pub code_shape: String,
    pub normalized_code_shape: String,
    pub inside_exported_identity: Option<String>,
    pub occurrence_key: String,
}

#[derive(Debug, Clone, Copy)]
struct LocalDeclaration {
    node_kind: &'static str,
    span: Span,
}

#[derive(Debug)]
struct NamedImportSeed {
    from_spec: String,
    imported_name: String,
    local_name: String,
    type_only: bool,
    line: usize,
}

#[derive(Debug)]
struct NamedImportMemberUse {
    name: String,
    line: usize,
}

#[derive(Debug)]
struct NamedImportPrecisionRecord {
    from_spec: String,
    imported_name: String,
    local_name: String,
    type_only: bool,
    line: usize,
    members: Vec<NamedImportMemberUse>,
    degraded: bool,
}

#[derive(Debug, Clone)]
struct ExportedIdentityRange {
    start: u32,
    end: u32,
    identity: String,
}

pub fn build_js_ts_extract_response(request: JsTsExtractRequest) -> Result<JsTsExtractResponse> {
    if request.schema_version != JS_TS_EXTRACT_REQUEST_SCHEMA_VERSION {
        bail!(
            "js-ts-extract-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let relative_resolver = RelativeSourceResolver::new(request.source_files);
    let files = request
        .files
        .into_iter()
        .map(|input| extract_file_or_error(input, &relative_resolver))
        .collect();
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
    let loc = line_count(&input.source);
    match extract_file(&input, &artifact_file_path, relative_resolver) {
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
        loc,
        error,
    }
}

fn extract_file(
    input: &JsTsExtractInputFile,
    artifact_file_path: &str,
    relative_resolver: &RelativeSourceResolver,
) -> Result<JsTsExtractFileResult> {
    let allocator = Allocator::default();
    let source_type = source_type_for_path(&input.file_path);
    let parsed = parse_program(&allocator, &input.source, source_type)?;
    let line_starts = line_starts(&input.source);
    let mut defs = Vec::new();
    let mut uses = Vec::new();
    let mut re_exports = Vec::new();
    let local_declarations = collect_top_level_declaration_targets(&parsed.program);
    let named_imports = collect_named_import_seeds(&parsed.program, &line_starts);
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
    uses.extend(collect_named_import_precision_uses(
        &parsed.program,
        named_imports,
        &line_starts,
    ));
    annotate_relative_resolutions(&input.file_path, &mut uses, relative_resolver);

    let class_methods =
        collect_class_method_surface(&parsed.program, &line_starts, artifact_file_path);
    let type_escapes = collect_type_escapes(
        &parsed.program,
        &parsed.program.comments,
        &input.source,
        artifact_file_path,
        &line_starts,
        &exported_identity_ranges,
    );

    Ok(JsTsExtractFileResult {
        file_path: input.file_path.clone(),
        defs,
        uses,
        re_exports,
        class_methods,
        local_operations: Vec::new(),
        type_escapes,
        loc: line_count(&input.source),
        error: None,
    })
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(value: &bool) -> bool {
    !*value
}

fn source_type_for_path(file_path: &str) -> SourceType {
    SourceType::from_path(Path::new(file_path)).unwrap_or_else(|_| SourceType::ts())
}

fn parse_program<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
) -> Result<oxc_parser::ParserReturn<'a>> {
    let first = Parser::new(allocator, source, source_type).parse();
    if first.diagnostics.is_empty() {
        return Ok(first);
    }
    if source_type.is_javascript() && !source_type.is_jsx() {
        let jsx = Parser::new(allocator, source, source_type.with_jsx(true)).parse();
        if jsx.diagnostics.is_empty() {
            return Ok(jsx);
        }
    }
    Err(anyhow!(
        "oxc-parser: {}",
        first
            .diagnostics
            .first()
            .map(|diagnostic| format!("{diagnostic:?}"))
            .unwrap_or_else(|| "syntax error".to_string())
    ))
}

fn collect_top_level_declaration_targets(
    program: &Program<'_>,
) -> BTreeMap<String, LocalDeclaration> {
    let mut out = BTreeMap::new();
    for statement in &program.body {
        match statement {
            Statement::FunctionDeclaration(function) => {
                if let Some(id) = function.id.as_ref() {
                    out.entry(id.name.to_string()).or_insert(LocalDeclaration {
                        node_kind: "FunctionDeclaration",
                        span: function.span,
                    });
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(id) = class.id.as_ref() {
                    out.entry(id.name.to_string()).or_insert(LocalDeclaration {
                        node_kind: "ClassDeclaration",
                        span: class.span,
                    });
                }
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    if let Some(name) = binding_identifier_name(&declarator.id) {
                        out.entry(name).or_insert(LocalDeclaration {
                            node_kind: "VariableDeclarator",
                            span: declarator.span,
                        });
                    }
                }
            }
            Statement::TSTypeAliasDeclaration(alias) => {
                out.entry(alias.id.name.to_string())
                    .or_insert(LocalDeclaration {
                        node_kind: "TSTypeAliasDeclaration",
                        span: alias.span,
                    });
            }
            Statement::TSInterfaceDeclaration(interface) => {
                out.entry(interface.id.name.to_string())
                    .or_insert(LocalDeclaration {
                        node_kind: "TSInterfaceDeclaration",
                        span: interface.span,
                    });
            }
            Statement::TSEnumDeclaration(enumeration) => {
                out.entry(enumeration.id.name.to_string())
                    .or_insert(LocalDeclaration {
                        node_kind: "TSEnumDeclaration",
                        span: enumeration.span,
                    });
            }
            Statement::ExportNamedDeclaration(export) if export.source.is_none() => {
                if let Some(declaration) = export.declaration.as_ref() {
                    collect_local_declaration(declaration, &mut out);
                }
            }
            Statement::ExportDefaultDeclaration(export) => match &export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                    if let Some(id) = function.id.as_ref() {
                        out.entry(id.name.to_string()).or_insert(LocalDeclaration {
                            node_kind: "FunctionDeclaration",
                            span: function.span,
                        });
                    }
                }
                ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                    if let Some(id) = class.id.as_ref() {
                        out.entry(id.name.to_string()).or_insert(LocalDeclaration {
                            node_kind: "ClassDeclaration",
                            span: class.span,
                        });
                    }
                }
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface) => {
                    out.entry(interface.id.name.to_string())
                        .or_insert(LocalDeclaration {
                            node_kind: "TSInterfaceDeclaration",
                            span: interface.span,
                        });
                }
                _ => {}
            },
            _ => {}
        }
    }
    out
}

fn collect_local_declaration(
    declaration: &Declaration<'_>,
    out: &mut BTreeMap<String, LocalDeclaration>,
) {
    match declaration {
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = function.id.as_ref() {
                out.entry(id.name.to_string()).or_insert(LocalDeclaration {
                    node_kind: "FunctionDeclaration",
                    span: function.span,
                });
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = class.id.as_ref() {
                out.entry(id.name.to_string()).or_insert(LocalDeclaration {
                    node_kind: "ClassDeclaration",
                    span: class.span,
                });
            }
        }
        Declaration::VariableDeclaration(declaration) => {
            for declarator in &declaration.declarations {
                if let Some(name) = binding_identifier_name(&declarator.id) {
                    out.entry(name).or_insert(LocalDeclaration {
                        node_kind: "VariableDeclarator",
                        span: declarator.span,
                    });
                }
            }
        }
        Declaration::TSTypeAliasDeclaration(alias) => {
            out.entry(alias.id.name.to_string())
                .or_insert(LocalDeclaration {
                    node_kind: "TSTypeAliasDeclaration",
                    span: alias.span,
                });
        }
        Declaration::TSInterfaceDeclaration(interface) => {
            out.entry(interface.id.name.to_string())
                .or_insert(LocalDeclaration {
                    node_kind: "TSInterfaceDeclaration",
                    span: interface.span,
                });
        }
        Declaration::TSEnumDeclaration(enumeration) => {
            out.entry(enumeration.id.name.to_string())
                .or_insert(LocalDeclaration {
                    node_kind: "TSEnumDeclaration",
                    span: enumeration.span,
                });
        }
        Declaration::TSModuleDeclaration(module) => {
            if let Some(name) = ts_module_name(module) {
                out.entry(name).or_insert(LocalDeclaration {
                    node_kind: "TSModuleDeclaration",
                    span: module.span,
                });
            }
        }
        _ => {}
    }
}

fn collect_exported_identity_ranges(
    program: &Program<'_>,
    artifact_file_path: &str,
    local_declarations: &BTreeMap<String, LocalDeclaration>,
) -> Vec<ExportedIdentityRange> {
    let mut out = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::ExportDefaultDeclaration(export) => {
                let span = export.declaration.span();
                out.push(ExportedIdentityRange {
                    start: span.start,
                    end: span.end,
                    identity: format!("{artifact_file_path}::default"),
                });
            }
            Statement::ExportNamedDeclaration(export) if export.source.is_none() => {
                if let Some(declaration) = export.declaration.as_ref() {
                    collect_exported_declaration_ranges(declaration, artifact_file_path, &mut out);
                }
                for specifier in &export.specifiers {
                    let Some(exported_name) = module_export_identifier_name(&specifier.exported)
                    else {
                        continue;
                    };
                    let local_name = module_export_identifier_name(&specifier.local)
                        .unwrap_or_else(|| exported_name.clone());
                    if let Some(target) = local_declarations.get(&local_name) {
                        out.push(ExportedIdentityRange {
                            start: target.span.start,
                            end: target.span.end,
                            identity: format!("{artifact_file_path}::{exported_name}"),
                        });
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn collect_exported_declaration_ranges(
    declaration: &Declaration<'_>,
    artifact_file_path: &str,
    out: &mut Vec<ExportedIdentityRange>,
) {
    match declaration {
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = function.id.as_ref() {
                out.push(exported_identity_range(
                    function.span,
                    artifact_file_path,
                    id.name.as_str(),
                ));
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = class.id.as_ref() {
                out.push(exported_identity_range(
                    class.span,
                    artifact_file_path,
                    id.name.as_str(),
                ));
            }
        }
        Declaration::VariableDeclaration(declaration) => {
            for declarator in &declaration.declarations {
                if let Some(name) = binding_identifier_name(&declarator.id) {
                    out.push(exported_identity_range(
                        declarator.span,
                        artifact_file_path,
                        &name,
                    ));
                }
            }
        }
        Declaration::TSTypeAliasDeclaration(alias) => {
            out.push(exported_identity_range(
                alias.span,
                artifact_file_path,
                alias.id.name.as_str(),
            ));
        }
        Declaration::TSInterfaceDeclaration(interface) => {
            out.push(exported_identity_range(
                interface.span,
                artifact_file_path,
                interface.id.name.as_str(),
            ));
        }
        Declaration::TSEnumDeclaration(enumeration) => {
            out.push(exported_identity_range(
                enumeration.span,
                artifact_file_path,
                enumeration.id.name.as_str(),
            ));
        }
        Declaration::TSModuleDeclaration(module) => {
            if let Some(name) = ts_module_name(module) {
                out.push(exported_identity_range(
                    module.span,
                    artifact_file_path,
                    &name,
                ));
            }
        }
        _ => {}
    }
}

fn exported_identity_range(
    span: Span,
    artifact_file_path: &str,
    name: &str,
) -> ExportedIdentityRange {
    ExportedIdentityRange {
        start: span.start,
        end: span.end,
        identity: format!("{artifact_file_path}::{name}"),
    }
}

fn collect_export_definitions(
    statement: &Statement<'_>,
    defs: &mut Vec<DefinitionRecord>,
    line_starts: &[usize],
    artifact_file_path: &str,
    local_declarations: &BTreeMap<String, LocalDeclaration>,
) {
    match statement {
        Statement::ExportDefaultDeclaration(export) => {
            defs.push(default_definition(export, line_starts, artifact_file_path))
        }
        Statement::ExportNamedDeclaration(export) if export.source.is_none() => {
            if let Some(declaration) = export.declaration.as_ref() {
                collect_declaration_defs(declaration, defs, line_starts, artifact_file_path);
            }
            collect_export_specifier_defs(
                export,
                defs,
                line_starts,
                artifact_file_path,
                local_declarations,
            );
        }
        _ => {}
    }
}

fn default_definition(
    export: &ExportDefaultDeclaration<'_>,
    line_starts: &[usize],
    artifact_file_path: &str,
) -> DefinitionRecord {
    let (node_kind, span) = match &export.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            ("FunctionDeclaration", function.span)
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => ("ClassDeclaration", class.span),
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface) => {
            ("TSInterfaceDeclaration", interface.span)
        }
        _ => ("ExportDefaultDeclaration", export.span),
    };
    DefinitionRecord {
        name: "default".to_string(),
        kind: "default".to_string(),
        line: line_for_span(line_starts, export.span),
        local_name: None,
        definition_id: Some(definition_id(artifact_file_path, node_kind, span)),
    }
}

fn collect_declaration_defs(
    declaration: &Declaration<'_>,
    defs: &mut Vec<DefinitionRecord>,
    line_starts: &[usize],
    artifact_file_path: &str,
) {
    match declaration {
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = function.id.as_ref() {
                defs.push(DefinitionRecord {
                    name: id.name.to_string(),
                    kind: "FunctionDeclaration".to_string(),
                    line: line_for_span(line_starts, function.span),
                    local_name: None,
                    definition_id: Some(definition_id(
                        artifact_file_path,
                        "FunctionDeclaration",
                        function.span,
                    )),
                });
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = class.id.as_ref() {
                defs.push(DefinitionRecord {
                    name: id.name.to_string(),
                    kind: "ClassDeclaration".to_string(),
                    line: line_for_span(line_starts, class.span),
                    local_name: None,
                    definition_id: Some(definition_id(
                        artifact_file_path,
                        "ClassDeclaration",
                        class.span,
                    )),
                });
            }
        }
        Declaration::VariableDeclaration(declaration) => {
            collect_variable_defs(declaration, defs, line_starts, artifact_file_path);
        }
        Declaration::TSTypeAliasDeclaration(alias) => defs.push(DefinitionRecord {
            name: alias.id.name.to_string(),
            kind: "TSTypeAliasDeclaration".to_string(),
            line: line_for_span(line_starts, alias.span),
            local_name: None,
            definition_id: Some(definition_id(
                artifact_file_path,
                "TSTypeAliasDeclaration",
                alias.span,
            )),
        }),
        Declaration::TSInterfaceDeclaration(interface) => defs.push(DefinitionRecord {
            name: interface.id.name.to_string(),
            kind: "TSInterfaceDeclaration".to_string(),
            line: line_for_span(line_starts, interface.span),
            local_name: None,
            definition_id: Some(definition_id(
                artifact_file_path,
                "TSInterfaceDeclaration",
                interface.span,
            )),
        }),
        Declaration::TSEnumDeclaration(enumeration) => defs.push(DefinitionRecord {
            name: enumeration.id.name.to_string(),
            kind: "TSEnumDeclaration".to_string(),
            line: line_for_span(line_starts, enumeration.span),
            local_name: None,
            definition_id: Some(definition_id(
                artifact_file_path,
                "TSEnumDeclaration",
                enumeration.span,
            )),
        }),
        Declaration::TSModuleDeclaration(module) => {
            if let Some(name) = ts_module_name(module) {
                defs.push(DefinitionRecord {
                    name,
                    kind: "TSModuleDeclaration".to_string(),
                    line: line_for_span(line_starts, module.span),
                    local_name: None,
                    definition_id: Some(definition_id(
                        artifact_file_path,
                        "TSModuleDeclaration",
                        module.span,
                    )),
                });
            }
        }
        _ => {}
    }
}

fn collect_variable_defs(
    declaration: &VariableDeclaration<'_>,
    defs: &mut Vec<DefinitionRecord>,
    line_starts: &[usize],
    artifact_file_path: &str,
) {
    for declarator in &declaration.declarations {
        if let Some(name) = binding_identifier_name(&declarator.id) {
            defs.push(DefinitionRecord {
                name,
                kind: format!("{}-var", variable_kind_text(declaration.kind)),
                line: line_for_span(line_starts, declaration.span),
                local_name: None,
                definition_id: Some(definition_id(
                    artifact_file_path,
                    "VariableDeclarator",
                    declarator.span,
                )),
            });
        }
    }
}

fn collect_export_specifier_defs(
    export: &ExportNamedDeclaration<'_>,
    defs: &mut Vec<DefinitionRecord>,
    line_starts: &[usize],
    artifact_file_path: &str,
    local_declarations: &BTreeMap<String, LocalDeclaration>,
) {
    for specifier in &export.specifiers {
        let Some(exported_name) = module_export_identifier_name(&specifier.exported) else {
            continue;
        };
        let local_name = module_export_identifier_name(&specifier.local)
            .unwrap_or_else(|| exported_name.clone());
        let target = local_declarations
            .get(&local_name)
            .copied()
            .unwrap_or(LocalDeclaration {
                node_kind: "ExportSpecifier",
                span: specifier.span,
            });
        let local_name_field = (local_name != exported_name).then_some(local_name);
        defs.push(DefinitionRecord {
            name: exported_name,
            kind: "ExportSpecifier".to_string(),
            line: line_for_span(line_starts, specifier.span),
            local_name: local_name_field,
            definition_id: Some(definition_id(
                artifact_file_path,
                target.node_kind,
                target.span,
            )),
        });
    }
}

fn collect_re_exports(
    statement: &Statement<'_>,
    re_exports: &mut Vec<ReExportRecord>,
    uses: &mut Vec<UseRecord>,
    line_starts: &[usize],
) {
    match statement {
        Statement::ExportNamedDeclaration(export) => {
            let Some(source) = export.source.as_ref() else {
                return;
            };
            re_exports.push(ReExportRecord {
                source: source.value.to_string(),
                line: line_for_span(line_starts, export.span),
                namespace: None,
            });
            for specifier in &export.specifiers {
                let Some(name) = module_export_identifier_name(&specifier.local)
                    .or_else(|| module_export_identifier_name(&specifier.exported))
                else {
                    continue;
                };
                uses.push(UseRecord {
                    from_spec: source.value.to_string(),
                    name,
                    member_name: None,
                    kind: "reExport".to_string(),
                    type_only: is_type_only(export.export_kind)
                        || is_type_only(specifier.export_kind),
                    line: line_for_span(line_starts, specifier.span),
                    local_name: None,
                    degraded: false,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
        }
        Statement::ExportAllDeclaration(export) => {
            collect_export_all(export, re_exports, uses, line_starts)
        }
        _ => {}
    }
}

fn collect_export_all(
    export: &ExportAllDeclaration<'_>,
    re_exports: &mut Vec<ReExportRecord>,
    uses: &mut Vec<UseRecord>,
    line_starts: &[usize],
) {
    let namespace = export
        .exported
        .as_ref()
        .and_then(module_export_identifier_name);
    re_exports.push(ReExportRecord {
        source: export.source.value.to_string(),
        line: line_for_span(line_starts, export.span),
        namespace: namespace.clone(),
    });
    uses.push(UseRecord {
        from_spec: export.source.value.to_string(),
        name: namespace.unwrap_or_else(|| "*".to_string()),
        member_name: None,
        kind: if export.exported.is_some() {
            "reExportNamespace"
        } else {
            "reExportAll"
        }
        .to_string(),
        type_only: is_type_only(export.export_kind),
        line: line_for_span(line_starts, export.span),
        local_name: None,
        degraded: false,
        resolved_file: None,
        resolver_stage: None,
    });
}

fn collect_imports(statement: &Statement<'_>, uses: &mut Vec<UseRecord>, line_starts: &[usize]) {
    let Statement::ImportDeclaration(import) = statement else {
        return;
    };
    let specifiers = import
        .specifiers
        .as_ref()
        .map_or(&[][..], |items| items.as_slice());
    if specifiers.is_empty() {
        uses.push(UseRecord {
            from_spec: import.source.value.to_string(),
            name: "*".to_string(),
            member_name: None,
            kind: "import-side-effect".to_string(),
            type_only: false,
            line: line_for_span(line_starts, import.span),
            local_name: None,
            degraded: false,
            resolved_file: None,
            resolver_stage: None,
        });
        return;
    }

    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                let imported_name = module_export_identifier_name(&specifier.imported)
                    .unwrap_or_else(|| specifier.local.name.to_string());
                let local_name = specifier.local.name.to_string();
                uses.push(UseRecord {
                    from_spec: import.source.value.to_string(),
                    name: imported_name.clone(),
                    member_name: None,
                    kind: "import".to_string(),
                    type_only: is_type_only(import.import_kind)
                        || is_type_only(specifier.import_kind),
                    line: line_for_span(line_starts, specifier.span),
                    local_name: (local_name != imported_name).then_some(local_name),
                    degraded: false,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                uses.push(UseRecord {
                    from_spec: import.source.value.to_string(),
                    name: "default".to_string(),
                    member_name: None,
                    kind: "default".to_string(),
                    type_only: is_type_only(import.import_kind),
                    line: line_for_span(line_starts, specifier.span),
                    local_name: None,
                    degraded: false,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                uses.push(UseRecord {
                    from_spec: import.source.value.to_string(),
                    name: "*".to_string(),
                    member_name: None,
                    kind: "namespace".to_string(),
                    type_only: is_type_only(import.import_kind),
                    line: line_for_span(line_starts, specifier.span),
                    local_name: Some(specifier.local.name.to_string()),
                    degraded: false,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
        }
    }
}

fn collect_type_escapes(
    program: &Program<'_>,
    comments: &[Comment],
    source: &str,
    artifact_file_path: &str,
    line_starts: &[usize],
    exported_identity_ranges: &[ExportedIdentityRange],
) -> Vec<TypeEscapeRecord> {
    let mut specific = SpecificTypeEscapeVisitor::new(
        source,
        artifact_file_path,
        line_starts,
        exported_identity_ranges,
    );
    specific.visit_program(program);

    let mut explicit = ExplicitAnyVisitor::new(
        source,
        artifact_file_path,
        line_starts,
        exported_identity_ranges,
        specific.consumed_any_starts.clone(),
    );
    explicit.visit_program(program);

    let mut facts = specific.facts;
    facts.extend(explicit.facts);
    facts.extend(collect_comment_type_escapes(
        comments,
        source,
        artifact_file_path,
        line_starts,
    ));
    facts.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then_with(|| left.occurrence_key.cmp(&right.occurrence_key))
    });
    facts
}

struct SpecificTypeEscapeVisitor<'a> {
    source: &'a str,
    artifact_file_path: &'a str,
    line_starts: &'a [usize],
    exported_identity_ranges: &'a [ExportedIdentityRange],
    consumed_any_starts: BTreeSet<u32>,
    facts: Vec<TypeEscapeRecord>,
}

impl<'a> SpecificTypeEscapeVisitor<'a> {
    fn new(
        source: &'a str,
        artifact_file_path: &'a str,
        line_starts: &'a [usize],
        exported_identity_ranges: &'a [ExportedIdentityRange],
    ) -> Self {
        Self {
            source,
            artifact_file_path,
            line_starts,
            exported_identity_ranges,
            consumed_any_starts: BTreeSet::new(),
            facts: Vec::new(),
        }
    }

    fn push_fact(&mut self, span: Span, escape_kind: &'static str) {
        self.facts.push(type_escape_record(
            self.source,
            self.artifact_file_path,
            self.line_starts,
            self.exported_identity_ranges,
            span,
            escape_kind,
        ));
    }

    fn consume_any_starts(&mut self, starts: BTreeSet<u32>) {
        self.consumed_any_starts.extend(starts);
    }
}

impl<'a> Visit<'a> for SpecificTypeEscapeVisitor<'_> {
    fn visit_formal_parameter_rest(&mut self, it: &FormalParameterRest<'a>) {
        if let Some(type_annotation) = &it.type_annotation {
            let any_starts = collect_any_type_starts(&type_annotation.type_annotation);
            if !any_starts.is_empty() {
                self.consume_any_starts(any_starts);
                self.push_fact(it.span, "rest-any-args");
                return;
            }
        }
        walk::walk_formal_parameter_rest(self, it);
    }

    fn visit_ts_index_signature(&mut self, it: &TSIndexSignature<'a>) {
        let any_starts = collect_any_type_starts(&it.type_annotation.type_annotation);
        if !any_starts.is_empty() {
            self.consume_any_starts(any_starts);
            self.push_fact(it.span, "index-sig-any");
            return;
        }
        walk::walk_ts_index_signature(self, it);
    }

    fn visit_ts_type_parameter(&mut self, it: &TSTypeParameter<'a>) {
        if it.default.as_ref().is_some_and(is_any_type) {
            if let Some(default) = &it.default {
                self.consumed_any_starts.insert(default.span().start);
            }
            self.push_fact(it.span, "generic-default-any");
            return;
        }
        walk::walk_ts_type_parameter(self, it);
    }

    fn visit_ts_type_assertion(&mut self, it: &TSTypeAssertion<'a>) {
        if is_any_type(&it.type_annotation) {
            self.consumed_any_starts
                .insert(it.type_annotation.span().start);
            self.push_fact(it.span, "angle-any");
            return;
        }
        walk::walk_ts_type_assertion(self, it);
    }

    fn visit_ts_as_expression(&mut self, it: &TSAsExpression<'a>) {
        if let Expression::TSAsExpression(inner) = &it.expression {
            if is_unknown_type(&inner.type_annotation) {
                if is_any_type(&it.type_annotation) {
                    self.consumed_any_starts
                        .insert(it.type_annotation.span().start);
                }
                self.push_fact(it.span, "as-unknown-as-T");
                walk::walk_ts_as_expression(self, it);
                return;
            }
        }
        if is_any_type(&it.type_annotation) {
            self.consumed_any_starts
                .insert(it.type_annotation.span().start);
            self.push_fact(it.span, "as-any");
            return;
        }
        walk::walk_ts_as_expression(self, it);
    }
}

struct ExplicitAnyVisitor<'a> {
    source: &'a str,
    artifact_file_path: &'a str,
    line_starts: &'a [usize],
    exported_identity_ranges: &'a [ExportedIdentityRange],
    consumed_any_starts: BTreeSet<u32>,
    facts: Vec<TypeEscapeRecord>,
}

impl<'a> ExplicitAnyVisitor<'a> {
    fn new(
        source: &'a str,
        artifact_file_path: &'a str,
        line_starts: &'a [usize],
        exported_identity_ranges: &'a [ExportedIdentityRange],
        consumed_any_starts: BTreeSet<u32>,
    ) -> Self {
        Self {
            source,
            artifact_file_path,
            line_starts,
            exported_identity_ranges,
            consumed_any_starts,
            facts: Vec::new(),
        }
    }
}

impl<'a> Visit<'a> for ExplicitAnyVisitor<'_> {
    fn visit_ts_any_keyword(&mut self, it: &TSAnyKeyword) {
        if self.consumed_any_starts.contains(&it.span.start) {
            return;
        }
        self.facts.push(type_escape_record(
            self.source,
            self.artifact_file_path,
            self.line_starts,
            self.exported_identity_ranges,
            it.span,
            "explicit-any",
        ));
    }
}

struct AnyTypeStartCollector {
    starts: BTreeSet<u32>,
}

impl<'a> Visit<'a> for AnyTypeStartCollector {
    fn visit_ts_any_keyword(&mut self, it: &TSAnyKeyword) {
        self.starts.insert(it.span.start);
    }
}

fn collect_any_type_starts(ty: &TSType<'_>) -> BTreeSet<u32> {
    let mut collector = AnyTypeStartCollector {
        starts: BTreeSet::new(),
    };
    collector.visit_ts_type(ty);
    collector.starts
}

fn is_any_type(ty: &TSType<'_>) -> bool {
    matches!(ty, TSType::TSAnyKeyword(_))
}

fn is_unknown_type(ty: &TSType<'_>) -> bool {
    matches!(ty, TSType::TSUnknownKeyword(_))
}

fn collect_comment_type_escapes(
    comments: &[Comment],
    source: &str,
    artifact_file_path: &str,
    line_starts: &[usize],
) -> Vec<TypeEscapeRecord> {
    let mut facts = Vec::new();
    for comment in comments {
        let value = source_slice_span(source, comment.content_span());
        let escape_kind = if comment.is_line() {
            line_comment_escape_kind(value)
        } else {
            block_comment_escape_kind(value)
        };
        let Some(escape_kind) = escape_kind else {
            continue;
        };
        let code_shape = source_slice_span(source, comment.span).to_string();
        let normalized_code_shape = normalize_code_shape(&code_shape);
        let occurrence_key = type_escape_occurrence_key(
            artifact_file_path,
            escape_kind,
            &normalized_code_shape,
            None,
        );
        facts.push(TypeEscapeRecord {
            file: artifact_file_path.to_string(),
            line: line_for_span(line_starts, comment.span),
            escape_kind,
            code_shape,
            normalized_code_shape,
            inside_exported_identity: None,
            occurrence_key,
        });
    }
    facts
}

fn line_comment_escape_kind(value: &str) -> Option<&'static str> {
    let trimmed = value.trim_start();
    if starts_with_directive(trimmed, "@ts-ignore") {
        return Some("ts-ignore");
    }
    if starts_with_directive(trimmed, "@ts-expect-error") {
        return Some("ts-expect-error");
    }
    eslint_no_explicit_any(trimmed).then_some("no-explicit-any-disable")
}

fn block_comment_escape_kind(value: &str) -> Option<&'static str> {
    let trimmed = value.trim_start();
    if eslint_no_explicit_any(trimmed) {
        return Some("no-explicit-any-disable");
    }
    jsdoc_any(value).then_some("jsdoc-any")
}

fn starts_with_directive(value: &str, directive: &str) -> bool {
    let Some(rest) = value.strip_prefix(directive) else {
        return false;
    };
    rest.chars()
        .next()
        .is_none_or(|c| !matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-'))
}

fn eslint_no_explicit_any(value: &str) -> bool {
    value.starts_with("eslint-disable") && value.contains("no-explicit-any")
}

fn jsdoc_any(value: &str) -> bool {
    value
        .lines()
        .map(|line| line.trim_start().trim_start_matches('*').trim_start())
        .any(|line| {
            [
                "@type",
                "@param",
                "@return",
                "@returns",
                "@typedef",
                "@property",
            ]
            .iter()
            .any(|directive| starts_with_directive(line, directive) && contains_braced_any(line))
        })
}

fn contains_braced_any(value: &str) -> bool {
    let mut rest = value;
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('}') else {
            return false;
        };
        if rest[..close].trim() == "any" {
            return true;
        }
        rest = &rest[close + 1..];
    }
    false
}

fn type_escape_record(
    source: &str,
    artifact_file_path: &str,
    line_starts: &[usize],
    exported_identity_ranges: &[ExportedIdentityRange],
    span: Span,
    escape_kind: &'static str,
) -> TypeEscapeRecord {
    let code_shape = source_slice_span(source, span).to_string();
    let normalized_code_shape = normalize_code_shape(&code_shape);
    let inside_exported_identity = inside_exported_identity(exported_identity_ranges, span);
    let occurrence_key = type_escape_occurrence_key(
        artifact_file_path,
        escape_kind,
        &normalized_code_shape,
        inside_exported_identity.as_deref(),
    );
    TypeEscapeRecord {
        file: artifact_file_path.to_string(),
        line: line_for_span(line_starts, span),
        escape_kind,
        code_shape,
        normalized_code_shape,
        inside_exported_identity,
        occurrence_key,
    }
}

fn inside_exported_identity(ranges: &[ExportedIdentityRange], span: Span) -> Option<String> {
    ranges
        .iter()
        .filter(|range| range.start <= span.start && span.end <= range.end)
        .min_by_key(|range| range.end.saturating_sub(range.start))
        .map(|range| range.identity.clone())
}

fn type_escape_occurrence_key(
    file: &str,
    escape_kind: &str,
    normalized_code_shape: &str,
    inside_exported_identity: Option<&str>,
) -> String {
    sha256_text(&format!(
        "{}|{}|{}|{}",
        file,
        escape_kind,
        normalized_code_shape,
        inside_exported_identity.unwrap_or("<top-level>")
    ))
}

fn source_slice_span(source: &str, span: Span) -> &str {
    let start = span.start as usize;
    let end = span.end as usize;
    source.get(start..end).unwrap_or("")
}

fn normalize_code_shape(raw: &str) -> String {
    if raw.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let mut state = CodeShapeState::Code;
    let mut prev_space = false;
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        let next = chars.peek().copied();
        match state {
            CodeShapeState::Single => {
                out.push(c);
                if c == '\\' {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else if c == '\'' {
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::Double => {
                out.push(c);
                if c == '\\' {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else if c == '"' {
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::Template => {
                out.push(c);
                if c == '\\' {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else if c == '`' {
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::LineComment => {
                out.push(c);
                if c == '\n' {
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::BlockComment => {
                out.push(c);
                if c == '*' && next == Some('/') {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::Code => {
                if c == '\'' {
                    state = CodeShapeState::Single;
                    out.push(c);
                    prev_space = false;
                } else if c == '"' {
                    state = CodeShapeState::Double;
                    out.push(c);
                    prev_space = false;
                } else if c == '`' {
                    state = CodeShapeState::Template;
                    out.push(c);
                    prev_space = false;
                } else if c == '/' && next == Some('/') {
                    state = CodeShapeState::LineComment;
                    out.push(c);
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                    prev_space = false;
                } else if c == '/' && next == Some('*') {
                    state = CodeShapeState::BlockComment;
                    out.push(c);
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                    prev_space = false;
                } else if c.is_whitespace() {
                    if !prev_space {
                        out.push(' ');
                        prev_space = true;
                    }
                } else {
                    out.push(c);
                    prev_space = false;
                }
            }
        }
    }
    let mut normalized = out.trim().to_string();
    if normalized.ends_with(';') {
        normalized.pop();
        normalized = normalized.trim_end().to_string();
    }
    normalized
}

#[derive(Debug, Clone, Copy)]
enum CodeShapeState {
    Code,
    Single,
    Double,
    Template,
    LineComment,
    BlockComment,
}

fn collect_named_import_seeds(
    program: &Program<'_>,
    line_starts: &[usize],
) -> Vec<NamedImportSeed> {
    let mut out = Vec::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        let specifiers = import
            .specifiers
            .as_ref()
            .map_or(&[][..], |items| items.as_slice());
        for specifier in specifiers {
            let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                continue;
            };
            let imported_name = module_export_identifier_name(&specifier.imported)
                .unwrap_or_else(|| specifier.local.name.to_string());
            out.push(NamedImportSeed {
                from_spec: import.source.value.to_string(),
                imported_name,
                local_name: specifier.local.name.to_string(),
                type_only: is_type_only(import.import_kind) || is_type_only(specifier.import_kind),
                line: line_for_span(line_starts, specifier.span),
            });
        }
    }
    out
}

fn collect_named_import_precision_uses(
    program: &Program<'_>,
    seeds: Vec<NamedImportSeed>,
    line_starts: &[usize],
) -> Vec<UseRecord> {
    if seeds.is_empty() {
        return Vec::new();
    }
    let mut visitor = NamedImportPrecisionVisitor::new(seeds, line_starts);
    visitor.visit_program(program);
    visitor.into_uses()
}

struct NamedImportPrecisionVisitor<'a> {
    records: Vec<NamedImportPrecisionRecord>,
    index_by_local: BTreeMap<String, usize>,
    scopes: Vec<BTreeSet<String>>,
    line_starts: &'a [usize],
    non_escaping_identifier_depth: usize,
}

impl<'a> NamedImportPrecisionVisitor<'a> {
    fn new(seeds: Vec<NamedImportSeed>, line_starts: &'a [usize]) -> Self {
        let mut records = Vec::with_capacity(seeds.len());
        let mut index_by_local = BTreeMap::new();
        for seed in seeds {
            let index = records.len();
            index_by_local
                .entry(seed.local_name.clone())
                .or_insert(index);
            records.push(NamedImportPrecisionRecord {
                from_spec: seed.from_spec,
                imported_name: seed.imported_name,
                local_name: seed.local_name,
                type_only: seed.type_only,
                line: seed.line,
                members: Vec::new(),
                degraded: false,
            });
        }
        Self {
            records,
            index_by_local,
            scopes: vec![BTreeSet::new()],
            line_starts,
            non_escaping_identifier_depth: 0,
        }
    }

    fn into_uses(self) -> Vec<UseRecord> {
        let mut uses = Vec::new();
        for record in self.records {
            if !record.members.is_empty() && !record.degraded {
                for member in record.members {
                    uses.push(UseRecord {
                        from_spec: record.from_spec.clone(),
                        name: record.imported_name.clone(),
                        member_name: Some(member.name),
                        kind: "imported-namespace-member".to_string(),
                        type_only: record.type_only,
                        line: member.line,
                        local_name: Some(record.local_name.clone()),
                        degraded: false,
                        resolved_file: None,
                        resolver_stage: None,
                    });
                }
            } else if record.degraded {
                uses.push(UseRecord {
                    from_spec: record.from_spec,
                    name: record.imported_name,
                    member_name: None,
                    kind: "imported-namespace-escape".to_string(),
                    type_only: record.type_only,
                    line: record.line,
                    local_name: Some(record.local_name),
                    degraded: true,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
        }
        uses
    }

    fn active_record_index(&self, local_name: &str) -> Option<usize> {
        if self
            .scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(local_name))
        {
            return None;
        }
        self.index_by_local.get(local_name).copied()
    }

    fn add_binding(&mut self, binding: &BindingIdentifier<'_>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(binding.name.to_string());
        }
    }

    fn record_member(&mut self, local_name: &str, member_name: String, line: usize) {
        if let Some(index) = self.active_record_index(local_name) {
            self.records[index].members.push(NamedImportMemberUse {
                name: member_name,
                line,
            });
        }
    }

    fn degrade(&mut self, local_name: &str) {
        if let Some(index) = self.active_record_index(local_name) {
            self.records[index].degraded = true;
        }
    }

    fn with_non_escaping_identifiers(&mut self, f: impl FnOnce(&mut Self)) {
        self.non_escaping_identifier_depth += 1;
        f(self);
        self.non_escaping_identifier_depth -= 1;
    }
}

impl<'a> Visit<'a> for NamedImportPrecisionVisitor<'_> {
    fn enter_scope(
        &mut self,
        _flags: oxc_syntax::scope::ScopeFlags,
        _scope_id: &Cell<Option<oxc_syntax::scope::ScopeId>>,
    ) {
        self.scopes.push(BTreeSet::new());
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
        if self.scopes.is_empty() {
            self.scopes.push(BTreeSet::new());
        }
    }

    fn visit_import_declaration(&mut self, _it: &ImportDeclaration<'a>) {}

    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        self.add_binding(it);
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if self.non_escaping_identifier_depth == 0 {
            self.degrade(it.name.as_str());
        }
    }

    fn visit_member_expression(&mut self, it: &MemberExpression<'a>) {
        if let Some(local_name) = member_object_identifier_name(it) {
            if let Some(member_name) = static_member_property_name(it) {
                let line = line_for_span(self.line_starts, it.span());
                self.record_member(&local_name, member_name, line);
            } else {
                self.degrade(&local_name);
                if let MemberExpression::ComputedMemberExpression(member) = it {
                    self.visit_expression(&member.expression);
                }
            }
            return;
        }
        walk::walk_member_expression(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        if let Some(local_name) = assignment_target_identifier_name(&it.left) {
            self.degrade(&local_name);
            self.visit_expression(&it.right);
            return;
        }
        if let Some(local_name) = assignment_target_member_object_identifier_name(&it.left) {
            self.degrade(&local_name);
            self.visit_expression(&it.right);
            return;
        }
        walk::walk_assignment_expression(self, it);
    }

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        if let Some(local_name) = simple_assignment_target_identifier_name(&it.argument) {
            self.degrade(&local_name);
            return;
        }
        if let Some(local_name) =
            simple_assignment_target_member_object_identifier_name(&it.argument)
        {
            self.degrade(&local_name);
            return;
        }
        walk::walk_update_expression(self, it);
    }

    fn visit_unary_expression(&mut self, it: &UnaryExpression<'a>) {
        if it.operator == UnaryOperator::Typeof {
            self.with_non_escaping_identifiers(|visitor| {
                visitor.visit_expression(&it.argument);
            });
            return;
        }
        if it.operator == UnaryOperator::Delete {
            if let Some(local_name) = expression_identifier_name(&it.argument) {
                self.degrade(&local_name);
                return;
            }
            if let Some(local_name) = expression_member_object_identifier_name(&it.argument) {
                self.degrade(&local_name);
                return;
            }
        }
        walk::walk_unary_expression(self, it);
    }

    fn visit_if_statement(&mut self, it: &IfStatement<'a>) {
        self.with_non_escaping_identifiers(|visitor| {
            visitor.visit_expression(&it.test);
        });
        self.visit_statement(&it.consequent);
        if let Some(alternate) = &it.alternate {
            self.visit_statement(alternate);
        }
    }

    fn visit_logical_expression(&mut self, it: &LogicalExpression<'a>) {
        self.with_non_escaping_identifiers(|visitor| {
            visitor.visit_expression(&it.left);
        });
        self.visit_expression(&it.right);
    }
}

fn expression_identifier_name(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        Expression::ParenthesizedExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        Expression::ChainExpression(_) => None,
        Expression::TSAsExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        Expression::TSSatisfiesExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        Expression::TSNonNullExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => {
            expression_identifier_name(&expression.expression)
        }
        Expression::TSInstantiationExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        _ => None,
    }
}

fn expression_member_object_identifier_name(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::ParenthesizedExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        Expression::ChainExpression(expression) => expression
            .expression
            .as_member_expression()
            .and_then(member_object_identifier_name),
        Expression::TSAsExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        Expression::TSSatisfiesExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        Expression::TSNonNullExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        Expression::TSInstantiationExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        expression => expression
            .as_member_expression()
            .and_then(member_object_identifier_name),
    }
}

fn member_object_identifier_name(member: &MemberExpression<'_>) -> Option<String> {
    match member {
        MemberExpression::StaticMemberExpression(member) => {
            expression_identifier_name(&member.object)
        }
        MemberExpression::ComputedMemberExpression(member) => {
            expression_identifier_name(&member.object)
        }
        MemberExpression::PrivateFieldExpression(member) => {
            expression_identifier_name(&member.object)
        }
    }
}

fn static_member_property_name(member: &MemberExpression<'_>) -> Option<String> {
    match member {
        MemberExpression::StaticMemberExpression(member) => Some(member.property.name.to_string()),
        MemberExpression::ComputedMemberExpression(member) => match &member.expression {
            Expression::StringLiteral(literal) => Some(literal.value.to_string()),
            _ => None,
        },
        MemberExpression::PrivateFieldExpression(_) => None,
    }
}

fn assignment_target_identifier_name(
    target: &oxc_ast::ast::AssignmentTarget<'_>,
) -> Option<String> {
    target
        .as_simple_assignment_target()
        .and_then(simple_assignment_target_identifier_name)
}

fn assignment_target_member_object_identifier_name(
    target: &oxc_ast::ast::AssignmentTarget<'_>,
) -> Option<String> {
    target
        .as_simple_assignment_target()
        .and_then(simple_assignment_target_member_object_identifier_name)
}

fn simple_assignment_target_identifier_name(target: &SimpleAssignmentTarget<'_>) -> Option<String> {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            Some(identifier.name.to_string())
        }
        SimpleAssignmentTarget::TSAsExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        SimpleAssignmentTarget::TSSatisfiesExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        SimpleAssignmentTarget::TSNonNullExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        SimpleAssignmentTarget::TSTypeAssertion(expression) => {
            expression_identifier_name(&expression.expression)
        }
        _ => None,
    }
}

fn simple_assignment_target_member_object_identifier_name(
    target: &SimpleAssignmentTarget<'_>,
) -> Option<String> {
    match target {
        SimpleAssignmentTarget::TSAsExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        SimpleAssignmentTarget::TSSatisfiesExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        SimpleAssignmentTarget::TSNonNullExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        SimpleAssignmentTarget::TSTypeAssertion(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        target => target
            .as_member_expression()
            .and_then(member_object_identifier_name),
    }
}

const RESOLVE_FILE_EXTS: &[&str] = &[
    "", ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".mts", ".cts", ".d.ts", ".d.mts", ".d.cts",
];

const RESOLVE_INDEX_EXTS: &[&str] = &[
    "/index.ts",
    "/index.tsx",
    "/index.js",
    "/index.jsx",
    "/index.mjs",
    "/index.cjs",
    "/index.mts",
    "/index.cts",
    "/index.d.ts",
    "/index.d.mts",
    "/index.d.cts",
];

#[derive(Debug)]
struct RelativeSourceResolver {
    source_files: BTreeMap<String, String>,
}

impl RelativeSourceResolver {
    fn new(source_files: Vec<String>) -> Self {
        let mut out = BTreeMap::new();
        for source_file in source_files {
            out.entry(normalize_path_text(&source_file))
                .or_insert(source_file);
        }
        Self { source_files: out }
    }

    fn resolve(&self, from_file: &str, spec: &str) -> Option<String> {
        if !spec.starts_with("./") && !spec.starts_with("../") {
            return None;
        }
        let base = join_relative_spec(dirname_text(from_file), spec);
        for ext in RESOLVE_FILE_EXTS {
            if let Some(resolved) = self.source_file(&format!("{base}{ext}")) {
                return Some(resolved);
            }
        }
        for ext in RESOLVE_INDEX_EXTS {
            if let Some(resolved) = self.source_file(&format!("{base}{ext}")) {
                return Some(resolved);
            }
        }
        if js_output_extension(spec) {
            for alt in [".ts", ".tsx", ".mts", ".cts"] {
                if let Some(swapped) = replace_js_output_extension(spec, alt) {
                    let candidate = join_relative_spec(dirname_text(from_file), &swapped);
                    if let Some(resolved) = self.source_file(&candidate) {
                        return Some(resolved);
                    }
                }
            }
            if let Some(stripped) = strip_js_output_extension(&base) {
                for ext in RESOLVE_INDEX_EXTS {
                    if let Some(resolved) = self.source_file(&format!("{stripped}{ext}")) {
                        return Some(resolved);
                    }
                }
            }
        }
        None
    }

    fn source_file(&self, candidate: &str) -> Option<String> {
        self.source_files
            .get(&normalize_path_text(candidate))
            .cloned()
    }
}

fn annotate_relative_resolutions(
    from_file: &str,
    uses: &mut [UseRecord],
    resolver: &RelativeSourceResolver,
) {
    for use_record in uses {
        if let Some(resolved) = resolver.resolve(from_file, &use_record.from_spec) {
            use_record.resolved_file = Some(resolved);
            use_record.resolver_stage = Some("relative");
        }
    }
}

fn dirname_text(path: &str) -> &str {
    let normalized = path.rfind(['/', '\\']);
    normalized.map_or("", |index| &path[..index])
}

fn join_relative_spec(base: &str, spec: &str) -> String {
    let joined = if base.is_empty() {
        spec.to_string()
    } else {
        format!("{base}/{spec}")
    };
    normalize_path_text(&joined)
}

fn normalize_path_text(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let (prefix, rest) = split_path_prefix(&normalized);
    let absolute = rest.starts_with('/');
    let mut parts = Vec::new();
    for part in rest.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if let Some(last) = parts.last() {
                if last != &".." {
                    parts.pop();
                    continue;
                }
            }
            if !absolute {
                parts.push(part);
            }
            continue;
        }
        parts.push(part);
    }

    let body = parts.join("/");
    match (prefix.is_empty(), absolute, body.is_empty()) {
        (false, _, false) => format!("{prefix}/{body}"),
        (false, _, true) => prefix.to_string(),
        (true, true, false) => format!("/{body}"),
        (true, true, true) => "/".to_string(),
        (true, false, false) => body,
        (true, false, true) => ".".to_string(),
    }
}

fn split_path_prefix(path: &str) -> (&str, &str) {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        let prefix = &path[..2];
        let rest = path.get(2..).unwrap_or_default();
        return (prefix, rest);
    }
    ("", path)
}

fn js_output_extension(spec: &str) -> bool {
    [".mjs", ".cjs", ".js", ".jsx"]
        .iter()
        .any(|ext| spec.ends_with(ext))
}

fn replace_js_output_extension(spec: &str, alt: &str) -> Option<String> {
    for ext in [".mjs", ".cjs", ".js", ".jsx"] {
        if let Some(prefix) = spec.strip_suffix(ext) {
            return Some(format!("{prefix}{alt}"));
        }
    }
    None
}

fn strip_js_output_extension(spec: &str) -> Option<&str> {
    for ext in [".mjs", ".cjs", ".js", ".jsx"] {
        if let Some(prefix) = spec.strip_suffix(ext) {
            return Some(prefix);
        }
    }
    None
}

fn collect_class_method_surface(
    program: &Program<'_>,
    line_starts: &[usize],
    artifact_file_path: &str,
) -> Vec<ClassMethodRecord> {
    let mut out = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::ClassDeclaration(class) => {
                if let Some(name) = class.id.as_ref().map(|id| id.name.to_string()) {
                    collect_class_methods_from_class(
                        class,
                        &name,
                        line_starts,
                        artifact_file_path,
                        &mut out,
                    );
                }
            }
            Statement::ExportNamedDeclaration(export) => match export.declaration.as_ref() {
                Some(Declaration::ClassDeclaration(class)) => {
                    if let Some(name) = class.id.as_ref().map(|id| id.name.to_string()) {
                        collect_class_methods_from_class(
                            class,
                            &name,
                            line_starts,
                            artifact_file_path,
                            &mut out,
                        );
                    }
                }
                Some(Declaration::VariableDeclaration(declaration)) => {
                    for declarator in &declaration.declarations {
                        if let (Some(name), Some(Expression::ClassExpression(class))) = (
                            binding_identifier_name(&declarator.id),
                            declarator.init.as_ref(),
                        ) {
                            collect_class_methods_from_class(
                                class,
                                &name,
                                line_starts,
                                artifact_file_path,
                                &mut out,
                            );
                        }
                    }
                }
                _ => {}
            },
            Statement::ExportDefaultDeclaration(export) => {
                if let ExportDefaultDeclarationKind::ClassDeclaration(class) = &export.declaration {
                    if let Some(name) = class.id.as_ref().map(|id| id.name.to_string()) {
                        collect_class_methods_from_class(
                            class,
                            &name,
                            line_starts,
                            artifact_file_path,
                            &mut out,
                        );
                    }
                }
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    if let (Some(name), Some(Expression::ClassExpression(class))) = (
                        binding_identifier_name(&declarator.id),
                        declarator.init.as_ref(),
                    ) {
                        collect_class_methods_from_class(
                            class,
                            &name,
                            line_starts,
                            artifact_file_path,
                            &mut out,
                        );
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn collect_class_methods_from_class(
    class: &Class<'_>,
    class_name: &str,
    line_starts: &[usize],
    artifact_file_path: &str,
    out: &mut Vec<ClassMethodRecord>,
) {
    for element in &class.body.body {
        match element {
            ClassElement::MethodDefinition(method) => {
                if let Some(record) =
                    class_method_record(method, class_name, line_starts, artifact_file_path)
                {
                    out.push(record);
                }
            }
            ClassElement::PropertyDefinition(property) => {
                if let Some(record) = class_field_function_record(
                    property,
                    class_name,
                    line_starts,
                    artifact_file_path,
                ) {
                    out.push(record);
                }
            }
            _ => {}
        }
    }
}

fn class_method_record(
    method: &MethodDefinition<'_>,
    class_name: &str,
    line_starts: &[usize],
    artifact_file_path: &str,
) -> Option<ClassMethodRecord> {
    let method_name = property_key_name(&method.key, method.computed)?;
    if method_name == "constructor" {
        return None;
    }
    let member_kind = method_kind_text(method.kind).to_string();
    if member_kind == "constructor" {
        return None;
    }
    let line = line_for_span(line_starts, method.key.span());
    let end_line = line_for_span(line_starts, method.value.span);
    Some(ClassMethodRecord {
        identity: format!("{artifact_file_path}::{class_name}#{method_name}"),
        owner_file: artifact_file_path.to_string(),
        class_name: class_name.to_string(),
        name: method_name.clone(),
        method_name,
        kind: "ClassMethod",
        member_kind,
        visibility: visibility_text(method.accessibility, &method.key),
        r#static: method.r#static,
        computed: method.computed,
        line,
        end_line: (end_line != line).then_some(end_line),
    })
}

fn class_field_function_record(
    property: &PropertyDefinition<'_>,
    class_name: &str,
    line_starts: &[usize],
    artifact_file_path: &str,
) -> Option<ClassMethodRecord> {
    if !matches!(
        property.value.as_ref(),
        Some(Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_))
    ) {
        return None;
    }
    let method_name = property_key_name(&property.key, property.computed)?;
    if method_name == "constructor" {
        return None;
    }
    let line = line_for_span(line_starts, property.key.span());
    let end_line = property
        .value
        .as_ref()
        .map(|value| line_for_span(line_starts, value.span()))
        .unwrap_or(line);
    Some(ClassMethodRecord {
        identity: format!("{artifact_file_path}::{class_name}#{method_name}"),
        owner_file: artifact_file_path.to_string(),
        class_name: class_name.to_string(),
        name: method_name.clone(),
        method_name,
        kind: "ClassMethod",
        member_kind: "class-field-function".to_string(),
        visibility: visibility_text(property.accessibility, &property.key),
        r#static: property.r#static,
        computed: property.computed,
        line,
        end_line: (end_line != line).then_some(end_line),
    })
}

fn binding_identifier_name(pattern: &BindingPattern<'_>) -> Option<String> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.to_string()),
        _ => None,
    }
}

fn module_export_identifier_name(name: &ModuleExportName<'_>) -> Option<String> {
    match name {
        ModuleExportName::IdentifierName(identifier) => Some(identifier.name.to_string()),
        ModuleExportName::IdentifierReference(identifier) => Some(identifier.name.to_string()),
        ModuleExportName::StringLiteral(_) => None,
    }
}

fn property_key_name(key: &PropertyKey<'_>, computed: bool) -> Option<String> {
    match key {
        PropertyKey::PrivateIdentifier(identifier) => Some(format!("#{}", identifier.name)),
        PropertyKey::StaticIdentifier(identifier) if !computed => Some(identifier.name.to_string()),
        PropertyKey::StringLiteral(literal) if !computed => Some(literal.value.to_string()),
        PropertyKey::Identifier(identifier) if !computed => Some(identifier.name.to_string()),
        _ => None,
    }
}

fn visibility_text(accessibility: Option<TSAccessibility>, key: &PropertyKey<'_>) -> String {
    if matches!(key, PropertyKey::PrivateIdentifier(_)) {
        return "private".to_string();
    }
    match accessibility {
        Some(TSAccessibility::Private) => "private",
        Some(TSAccessibility::Protected) => "protected",
        Some(TSAccessibility::Public) | None => "public",
    }
    .to_string()
}

fn variable_kind_text(kind: VariableDeclarationKind) -> &'static str {
    match kind {
        VariableDeclarationKind::Var => "var",
        VariableDeclarationKind::Let => "let",
        VariableDeclarationKind::Const => "const",
        VariableDeclarationKind::Using => "using",
        VariableDeclarationKind::AwaitUsing => "await using",
    }
}

fn method_kind_text(kind: MethodDefinitionKind) -> &'static str {
    match kind {
        MethodDefinitionKind::Constructor => "constructor",
        MethodDefinitionKind::Method => "method",
        MethodDefinitionKind::Get => "get",
        MethodDefinitionKind::Set => "set",
    }
}

fn is_type_only(kind: ImportOrExportKind) -> bool {
    matches!(kind, ImportOrExportKind::Type)
}

fn ts_module_name(module: &oxc_ast::ast::TSModuleDeclaration<'_>) -> Option<String> {
    match &module.id {
        oxc_ast::ast::TSModuleDeclarationName::Identifier(identifier) => {
            Some(identifier.name.to_string())
        }
        oxc_ast::ast::TSModuleDeclarationName::StringLiteral(literal) => {
            Some(literal.value.to_string())
        }
    }
}

fn definition_id(file: &str, node_kind: &str, span: Span) -> String {
    format!(
        "{}#{}:{}-{}",
        file.replace('\\', "/"),
        node_kind,
        span.start,
        span.end
    )
}

fn line_count(source: &str) -> usize {
    source.split('\n').count()
}

fn line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

fn line_for_span(line_starts: &[usize], span: Span) -> usize {
    let offset = span.start as usize;
    match line_starts.binary_search(&offset) {
        Ok(index) => index + 1,
        Err(index) => index,
    }
}
