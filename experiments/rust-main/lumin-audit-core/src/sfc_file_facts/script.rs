use anyhow::{anyhow, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ExportDefaultDeclarationKind, Expression, ImportDeclarationSpecifier, ImportOrExportKind,
    ModuleExportName, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use std::collections::BTreeMap;
use std::path::Path;

use super::blocks::{line_of, ScriptBlock, ScriptDialect, SfcLanguage};
use super::protocol::SfcScriptImportConsumer;

#[derive(Debug, Clone)]
pub(super) struct ComponentBinding {
    pub(super) binding_name: String,
    pub(super) binding_source: String,
    pub(super) binding_kind: String,
    pub(super) imported_name: Option<String>,
}

#[derive(Debug, Default)]
pub(super) struct ComponentBindings {
    pub(super) imports: BTreeMap<String, ComponentBinding>,
    pub(super) namespace_imports: BTreeMap<String, ComponentBinding>,
    pub(super) exposed_names: BTreeMap<String, ComponentBinding>,
}

#[derive(Debug)]
pub(super) struct ScriptFacts {
    pub(super) imports: Vec<SfcScriptImportConsumer>,
    pub(super) bindings: ComponentBindings,
}

pub(super) fn extract_script_facts(
    source: &str,
    file_path: &str,
    language: SfcLanguage,
    blocks: &[ScriptBlock<'_>],
) -> Result<ScriptFacts> {
    let mut imports = Vec::new();
    let mut bindings = ComponentBindings::default();

    for block in blocks {
        let allocator = Allocator::default();
        let parsed = parse_program(&allocator, block.content, block.dialect).map_err(|error| {
            anyhow!(
                "sfc-file-facts-artifact: failed to parse {} block '{}' in {file_path}: {error}",
                language.as_str(),
                block.kind
            )
        })?;
        let block_facts = collect_block_facts(
            &parsed.program,
            source,
            file_path,
            block.start_offset,
            &block.kind,
        );
        imports.extend(block_facts.imports);

        for (name, binding) in &block_facts.import_bindings {
            bindings.imports.insert(name.clone(), binding.clone());
        }
        for (name, binding) in &block_facts.namespace_bindings {
            bindings
                .namespace_imports
                .insert(name.clone(), binding.clone());
        }

        if language == SfcLanguage::Vue && block.kind != "vue-script-setup" {
            for (tag_name, binding_name) in block_facts.options_components {
                if let Some(binding) = block_facts.import_bindings.get(&binding_name) {
                    bindings.exposed_names.insert(tag_name, binding.clone());
                }
            }
        } else {
            for (name, binding) in block_facts.import_bindings {
                bindings.exposed_names.insert(name, binding);
            }
        }
    }

    Ok(ScriptFacts { imports, bindings })
}

struct BlockFacts {
    imports: Vec<SfcScriptImportConsumer>,
    import_bindings: BTreeMap<String, ComponentBinding>,
    namespace_bindings: BTreeMap<String, ComponentBinding>,
    options_components: Vec<(String, String)>,
}

fn collect_block_facts(
    program: &Program<'_>,
    source: &str,
    file_path: &str,
    start_offset: usize,
    block_kind: &str,
) -> BlockFacts {
    let mut imports = Vec::new();
    let mut import_bindings = BTreeMap::new();
    let mut namespace_bindings = BTreeMap::new();

    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        let from_spec = import.source.value.to_string();
        if from_spec.is_empty() {
            continue;
        }
        let line = line_of(source, start_offset + import.span.start as usize);
        let declaration_type_only = is_type_only(import.import_kind);
        let specifiers = import
            .specifiers
            .as_ref()
            .map_or(&[][..], |items| items.as_slice());
        if specifiers.is_empty() {
            imports.push(SfcScriptImportConsumer {
                consumer_file: file_path.to_string(),
                from_spec,
                name: "*".to_string(),
                local_name: None,
                kind: "import-side-effect".to_string(),
                type_only: false,
                line,
                sfc_block_kind: block_kind.to_string(),
            });
            continue;
        }

        for specifier in specifiers {
            match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    let local_name = specifier.local.name.to_string();
                    imports.push(SfcScriptImportConsumer {
                        consumer_file: file_path.to_string(),
                        from_spec: from_spec.clone(),
                        name: "default".to_string(),
                        local_name: Some(local_name.clone()),
                        kind: "default".to_string(),
                        type_only: declaration_type_only,
                        line,
                        sfc_block_kind: block_kind.to_string(),
                    });
                    if !declaration_type_only {
                        import_bindings.insert(
                            local_name.clone(),
                            ComponentBinding {
                                binding_name: local_name,
                                binding_source: from_spec.clone(),
                                binding_kind: "default".to_string(),
                                imported_name: Some("default".to_string()),
                            },
                        );
                    }
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    let local_name = specifier.local.name.to_string();
                    imports.push(SfcScriptImportConsumer {
                        consumer_file: file_path.to_string(),
                        from_spec: from_spec.clone(),
                        name: "*".to_string(),
                        local_name: Some(local_name.clone()),
                        kind: "namespace".to_string(),
                        type_only: declaration_type_only,
                        line,
                        sfc_block_kind: block_kind.to_string(),
                    });
                    if !declaration_type_only {
                        namespace_bindings.insert(
                            local_name.clone(),
                            ComponentBinding {
                                binding_name: local_name,
                                binding_source: from_spec.clone(),
                                binding_kind: "namespace".to_string(),
                                imported_name: None,
                            },
                        );
                    }
                }
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    let imported_name = module_export_name(&specifier.imported);
                    let Some(imported_name) = imported_name else {
                        continue;
                    };
                    let local_name = specifier.local.name.to_string();
                    let type_only = declaration_type_only || is_type_only(specifier.import_kind);
                    imports.push(SfcScriptImportConsumer {
                        consumer_file: file_path.to_string(),
                        from_spec: from_spec.clone(),
                        name: imported_name.clone(),
                        local_name: Some(local_name.clone()),
                        kind: "import".to_string(),
                        type_only,
                        line,
                        sfc_block_kind: block_kind.to_string(),
                    });
                    if !type_only {
                        import_bindings.insert(
                            local_name.clone(),
                            ComponentBinding {
                                binding_name: local_name,
                                binding_source: from_spec.clone(),
                                binding_kind: "named".to_string(),
                                imported_name: Some(imported_name),
                            },
                        );
                    }
                }
            }
        }
    }

    BlockFacts {
        imports,
        options_components: collect_vue_options_components(program),
        import_bindings,
        namespace_bindings,
    }
}

fn collect_vue_options_components(program: &Program<'_>) -> Vec<(String, String)> {
    for statement in &program.body {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        let ExportDefaultDeclarationKind::ObjectExpression(options) = &export.declaration else {
            continue;
        };
        for property in &options.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if property_key_name(&property.key, property.computed).as_deref() != Some("components")
            {
                continue;
            }
            let Expression::ObjectExpression(components) = &property.value else {
                continue;
            };
            return components
                .properties
                .iter()
                .filter_map(|component| {
                    let ObjectPropertyKind::ObjectProperty(component) = component else {
                        return None;
                    };
                    let tag_name = property_key_name(&component.key, component.computed)?;
                    let Expression::Identifier(binding) = &component.value else {
                        return None;
                    };
                    Some((tag_name, binding.name.to_string()))
                })
                .collect();
        }
    }
    Vec::new()
}

fn property_key_name(key: &PropertyKey<'_>, computed: bool) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) if !computed => Some(identifier.name.to_string()),
        PropertyKey::StringLiteral(literal) if !computed => Some(literal.value.to_string()),
        PropertyKey::Identifier(identifier) if !computed => Some(identifier.name.to_string()),
        _ => None,
    }
}

fn module_export_name(name: &ModuleExportName<'_>) -> Option<String> {
    match name {
        ModuleExportName::IdentifierName(identifier) => Some(identifier.name.to_string()),
        ModuleExportName::IdentifierReference(identifier) => Some(identifier.name.to_string()),
        ModuleExportName::StringLiteral(literal) => Some(literal.value.to_string()),
    }
}

fn is_type_only(kind: ImportOrExportKind) -> bool {
    matches!(kind, ImportOrExportKind::Type)
}

fn parse_program<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    dialect: ScriptDialect,
) -> Result<ParserReturn<'a>> {
    let candidates: &[&str] = match dialect {
        ScriptDialect::Ts => &["block.ts", "block.tsx"],
        ScriptDialect::Tsx => &["block.tsx", "block.ts"],
        ScriptDialect::Js => &["block.js", "block.jsx", "block.ts"],
        ScriptDialect::Jsx => &["block.jsx", "block.ts"],
    };
    let mut first_error = None;
    for candidate in candidates {
        let source_type = SourceType::from_path(Path::new(candidate))?;
        let parsed = Parser::new(allocator, source, source_type).parse();
        if parsed.diagnostics.is_empty() {
            return Ok(parsed);
        }
        if first_error.is_none() {
            first_error = parsed
                .diagnostics
                .first()
                .map(|diagnostic| format!("{diagnostic:?}"));
        }
    }
    Err(anyhow!(
        "oxc-parser: {}",
        first_error.unwrap_or_else(|| "syntax error".to_string())
    ))
}
