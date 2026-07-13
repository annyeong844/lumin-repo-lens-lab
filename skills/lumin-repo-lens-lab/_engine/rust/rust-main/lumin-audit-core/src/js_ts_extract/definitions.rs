use super::{
    binding_identifier_name, definition_id, line_for_span, module_export_identifier_name,
    ts_module_name, variable_kind_text, DefinitionRecord,
};
use oxc_ast::ast::{
    Declaration, ExportDefaultDeclaration, ExportDefaultDeclarationKind, ExportNamedDeclaration,
    Program, Statement, VariableDeclaration,
};
use oxc_span::{GetSpan, Span};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub(super) struct LocalDeclaration {
    node_kind: &'static str,
    span: Span,
}

#[derive(Debug, Clone)]
pub(super) struct ExportedIdentityRange {
    pub(super) start: u32,
    pub(super) end: u32,
    pub(super) identity: String,
}

pub(super) fn collect_top_level_declaration_targets(
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

pub(super) fn collect_exported_identity_ranges(
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

pub(super) fn collect_export_definitions(
    statement: &Statement<'_>,
    defs: &mut Vec<DefinitionRecord>,
    line_starts: &[usize],
    artifact_file_path: &str,
    local_declarations: &BTreeMap<String, LocalDeclaration>,
) {
    match statement {
        Statement::ExportDefaultDeclaration(export) => {
            defs.push(default_definition(export, line_starts, artifact_file_path));
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
