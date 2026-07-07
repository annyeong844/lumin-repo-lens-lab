use anyhow::{anyhow, bail, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Class, ClassElement, Declaration, ExportAllDeclaration,
    ExportDefaultDeclaration, ExportDefaultDeclarationKind, ExportNamedDeclaration, Expression,
    ImportDeclarationSpecifier, ImportOrExportKind, MethodDefinition, MethodDefinitionKind,
    ModuleExportName, Program, PropertyDefinition, PropertyKey, Statement, TSAccessibility,
    VariableDeclaration, VariableDeclarationKind,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    pub type_escapes: Vec<serde_json::Value>,
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
    pub kind: String,
    pub type_only: bool,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_name: Option<String>,
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

#[derive(Debug, Clone, Copy)]
struct LocalDeclaration {
    node_kind: &'static str,
    span: Span,
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
    annotate_relative_resolutions(&input.file_path, &mut uses, relative_resolver);

    let class_methods =
        collect_class_method_surface(&parsed.program, &line_starts, artifact_file_path);

    Ok(JsTsExtractFileResult {
        file_path: input.file_path.clone(),
        defs,
        uses,
        re_exports,
        class_methods,
        local_operations: Vec::new(),
        type_escapes: Vec::new(),
        loc: line_count(&input.source),
        error: None,
    })
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
                    kind: "reExport".to_string(),
                    type_only: is_type_only(export.export_kind)
                        || is_type_only(specifier.export_kind),
                    line: line_for_span(line_starts, specifier.span),
                    local_name: None,
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
        kind: if export.exported.is_some() {
            "reExportNamespace"
        } else {
            "reExportAll"
        }
        .to_string(),
        type_only: is_type_only(export.export_kind),
        line: line_for_span(line_starts, export.span),
        local_name: None,
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
            kind: "import-side-effect".to_string(),
            type_only: false,
            line: line_for_span(line_starts, import.span),
            local_name: None,
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
                    kind: "import".to_string(),
                    type_only: is_type_only(import.import_kind)
                        || is_type_only(specifier.import_kind),
                    line: line_for_span(line_starts, specifier.span),
                    local_name: (local_name != imported_name).then_some(local_name),
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                uses.push(UseRecord {
                    from_spec: import.source.value.to_string(),
                    name: "default".to_string(),
                    kind: "default".to_string(),
                    type_only: is_type_only(import.import_kind),
                    line: line_for_span(line_starts, specifier.span),
                    local_name: None,
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                uses.push(UseRecord {
                    from_spec: import.source.value.to_string(),
                    name: "*".to_string(),
                    kind: "namespace".to_string(),
                    type_only: is_type_only(import.import_kind),
                    line: line_for_span(line_starts, specifier.span),
                    local_name: Some(specifier.local.name.to_string()),
                    resolved_file: None,
                    resolver_stage: None,
                });
            }
        }
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
