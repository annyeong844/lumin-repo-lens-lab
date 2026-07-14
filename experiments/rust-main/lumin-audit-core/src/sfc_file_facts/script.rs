use anyhow::{anyhow, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, BindingIdentifier, BindingPattern, CallExpression, ExportDefaultDeclarationKind,
    Expression, IdentifierReference, ImportDeclarationSpecifier, ImportOrExportKind,
    ModuleExportName, ObjectPropertyKind, Program, PropertyKey, Statement, VariableDeclarationKind,
};
use oxc_ast_visit::{walk, Visit};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use oxc_syntax::scope::{ScopeFlags, ScopeId};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::blocks::{line_of, ScriptBlock, ScriptDialect, SfcLanguage};
use super::conventions::{
    store_binding, svelte_store_subscription, vue_macro_registration, vue_options_registration,
};
use super::protocol::{SfcFileConvention, SfcScriptImportConsumer};

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
    pub(super) local_actions: BTreeMap<String, ComponentBinding>,
    pub(super) local_stores: BTreeMap<String, ComponentBinding>,
}

#[derive(Debug)]
pub(super) struct ScriptFacts {
    pub(super) imports: Vec<SfcScriptImportConsumer>,
    pub(super) bindings: ComponentBindings,
    pub(super) conventions: Vec<SfcFileConvention>,
}

pub(super) fn extract_script_facts(
    source: &str,
    file_path: &str,
    language: SfcLanguage,
    blocks: &[ScriptBlock<'_>],
) -> Result<ScriptFacts> {
    let mut imports = Vec::new();
    let mut bindings = ComponentBindings::default();
    let mut conventions = Vec::new();
    let mut store_references = Vec::new();
    let mut store_reference_keys = BTreeSet::new();

    for block in blocks {
        let allocator = Allocator::default();
        let parsed = parse_program(&allocator, block.content, block.dialect).map_err(|error| {
            anyhow!(
                "sfc-file-facts-artifact: failed to parse {} block '{}' in {file_path}: {error}",
                language.as_str(),
                block.kind
            )
        })?;
        let block_facts = collect_block_facts(&parsed.program, source, file_path, language, block);
        imports.extend(block_facts.imports);
        conventions.extend(block_facts.conventions);
        for reference in block_facts.store_references {
            let key = format!(
                "{file_path}|${}|{}|{}",
                reference.store_name, reference.line, reference.block_kind
            );
            if store_reference_keys.insert(key) {
                store_references.push(reference);
            }
        }

        for (name, binding) in &block_facts.import_bindings {
            bindings.imports.insert(name.clone(), binding.clone());
        }
        for (name, binding) in &block_facts.namespace_bindings {
            bindings
                .namespace_imports
                .insert(name.clone(), binding.clone());
        }
        bindings.local_actions.extend(block_facts.local_actions);
        bindings.local_stores.extend(block_facts.local_stores);

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

    for reference in store_references {
        let Some(binding) = store_binding(&reference.store_name, &bindings) else {
            continue;
        };
        conventions.push(svelte_store_subscription(
            file_path,
            &reference.store_name,
            binding,
            reference.line,
            &reference.block_kind,
        ));
    }

    Ok(ScriptFacts {
        imports,
        bindings,
        conventions,
    })
}

struct BlockFacts {
    imports: Vec<SfcScriptImportConsumer>,
    import_bindings: BTreeMap<String, ComponentBinding>,
    namespace_bindings: BTreeMap<String, ComponentBinding>,
    options_components: Vec<(String, String)>,
    local_actions: BTreeMap<String, ComponentBinding>,
    local_stores: BTreeMap<String, ComponentBinding>,
    conventions: Vec<SfcFileConvention>,
    store_references: Vec<SvelteStoreReference>,
}

struct SvelteStoreReference {
    store_name: String,
    line: usize,
    block_kind: String,
}

fn collect_block_facts(
    program: &Program<'_>,
    source: &str,
    file_path: &str,
    language: SfcLanguage,
    block: &ScriptBlock<'_>,
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
        let line = line_of(source, block.start_offset + import.span.start as usize);
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
                sfc_block_kind: block.kind.clone(),
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
                        sfc_block_kind: block.kind.clone(),
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
                        sfc_block_kind: block.kind.clone(),
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
                        sfc_block_kind: block.kind.clone(),
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

    let options_components = collect_vue_options_components(program);
    let local_actions = if language == SfcLanguage::Svelte {
        collect_local_svelte_actions(program, file_path)
    } else {
        BTreeMap::new()
    };
    let local_stores = if language == SfcLanguage::Svelte {
        collect_local_svelte_stores(program, file_path, &import_bindings)
    } else {
        BTreeMap::new()
    };
    let mut conventions = Vec::new();
    if language == SfcLanguage::Vue && block.kind == "vue-script-setup" {
        for candidate in collect_vue_macro_components(program, source, block.start_offset) {
            let Some(binding) = import_bindings.get(&candidate.binding_name) else {
                continue;
            };
            conventions.push(vue_macro_registration(
                file_path,
                &candidate.component_name,
                binding,
                candidate.line,
                &block.kind,
            ));
        }
    }
    if language == SfcLanguage::Vue && block.kind == "vue-script" {
        for (component_name, binding_name, line) in
            collect_vue_options_registration_components(program, source, block.start_offset)
        {
            let Some(binding) = import_bindings.get(&binding_name) else {
                continue;
            };
            conventions.push(vue_options_registration(
                file_path,
                &component_name,
                binding,
                line,
                &block.kind,
            ));
        }
    }
    let store_references = if language == SfcLanguage::Svelte {
        collect_svelte_store_references(program, source, block)
    } else {
        Vec::new()
    };

    BlockFacts {
        imports,
        options_components,
        import_bindings,
        namespace_bindings,
        local_actions,
        local_stores,
        conventions,
        store_references,
    }
}

fn collect_vue_options_components(program: &Program<'_>) -> Vec<(String, String)> {
    collect_vue_options_component_candidates(program)
        .into_iter()
        .map(|candidate| (candidate.component_name, candidate.binding_name))
        .collect()
}

struct VueComponentCandidate {
    component_name: String,
    binding_name: String,
    span_start: usize,
}

fn collect_vue_options_component_candidates(program: &Program<'_>) -> Vec<VueComponentCandidate> {
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
                    Some(VueComponentCandidate {
                        component_name: tag_name,
                        binding_name: binding.name.to_string(),
                        span_start: component.span.start as usize,
                    })
                })
                .collect();
        }
    }
    Vec::new()
}

fn collect_vue_options_registration_components(
    program: &Program<'_>,
    source: &str,
    start_offset: usize,
) -> Vec<(String, String, usize)> {
    collect_vue_options_component_candidates(program)
        .into_iter()
        .map(|candidate| {
            (
                candidate.component_name,
                candidate.binding_name,
                line_of(source, start_offset + candidate.span_start),
            )
        })
        .collect()
}

fn collect_vue_macro_components(
    program: &Program<'_>,
    source: &str,
    start_offset: usize,
) -> Vec<VueMacroComponentCandidate> {
    let mut visitor = VueMacroComponentVisitor::default();
    visitor.visit_program(program);
    visitor
        .candidates
        .into_iter()
        .map(|candidate| VueMacroComponentCandidate {
            component_name: candidate.component_name,
            binding_name: candidate.binding_name,
            line: line_of(source, start_offset + candidate.span_start),
        })
        .collect()
}

struct VueMacroComponentCandidate {
    component_name: String,
    binding_name: String,
    line: usize,
}

#[derive(Default)]
struct VueMacroComponentVisitor {
    candidates: Vec<VueComponentCandidate>,
}

impl<'a> Visit<'a> for VueMacroComponentVisitor {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let is_define_options = matches!(
            &call.callee,
            Expression::Identifier(identifier) if identifier.name == "defineOptions"
        );
        if is_define_options {
            if let Some(Argument::ObjectExpression(options)) = call.arguments.first() {
                for property in &options.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        continue;
                    };
                    if property_key_name(&property.key, property.computed).as_deref()
                        != Some("components")
                    {
                        continue;
                    }
                    let Expression::ObjectExpression(components) = &property.value else {
                        continue;
                    };
                    for component in &components.properties {
                        let ObjectPropertyKind::ObjectProperty(component) = component else {
                            continue;
                        };
                        let Some(component_name) =
                            property_key_name(&component.key, component.computed)
                        else {
                            continue;
                        };
                        let Expression::Identifier(binding) = &component.value else {
                            continue;
                        };
                        self.candidates.push(VueComponentCandidate {
                            component_name,
                            binding_name: binding.name.to_string(),
                            span_start: component.span.start as usize,
                        });
                    }
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}

fn collect_local_svelte_actions(
    program: &Program<'_>,
    file_path: &str,
) -> BTreeMap<String, ComponentBinding> {
    let mut actions = BTreeMap::new();
    for statement in &program.body {
        match statement {
            Statement::FunctionDeclaration(function) => {
                let Some(binding_name) = function.id.as_ref().map(|id| id.name.to_string()) else {
                    continue;
                };
                actions.insert(
                    binding_name.clone(),
                    ComponentBinding {
                        binding_name,
                        binding_source: file_path.to_string(),
                        binding_kind: "local-function".to_string(),
                        imported_name: None,
                    },
                );
            }
            Statement::VariableDeclaration(declaration)
                if declaration.kind == VariableDeclarationKind::Const =>
            {
                for declarator in &declaration.declarations {
                    let Some(binding_name) = binding_pattern_name(&declarator.id) else {
                        continue;
                    };
                    if !matches!(
                        declarator.init.as_ref(),
                        Some(
                            Expression::ArrowFunctionExpression(_)
                                | Expression::FunctionExpression(_)
                        )
                    ) {
                        continue;
                    }
                    actions.insert(
                        binding_name.clone(),
                        ComponentBinding {
                            binding_name,
                            binding_source: file_path.to_string(),
                            binding_kind: "local-const-function".to_string(),
                            imported_name: None,
                        },
                    );
                }
            }
            _ => {}
        }
    }
    actions
}

fn collect_local_svelte_stores(
    program: &Program<'_>,
    file_path: &str,
    imports: &BTreeMap<String, ComponentBinding>,
) -> BTreeMap<String, ComponentBinding> {
    let factory_bindings: BTreeSet<&str> = imports
        .values()
        .filter(|binding| {
            binding.binding_source == "svelte/store"
                && matches!(
                    binding.imported_name.as_deref(),
                    Some("writable" | "readable" | "derived")
                )
        })
        .map(|binding| binding.binding_name.as_str())
        .collect();
    if factory_bindings.is_empty() {
        return BTreeMap::new();
    }

    let mut stores = BTreeMap::new();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        if declaration.kind != VariableDeclarationKind::Const {
            continue;
        }
        for declarator in &declaration.declarations {
            let Some(binding_name) = binding_pattern_name(&declarator.id) else {
                continue;
            };
            let Some(Expression::CallExpression(call)) = &declarator.init else {
                continue;
            };
            let Expression::Identifier(callee) = &call.callee else {
                continue;
            };
            if !factory_bindings.contains(callee.name.as_str()) {
                continue;
            }
            stores.insert(
                binding_name.clone(),
                ComponentBinding {
                    binding_name,
                    binding_source: file_path.to_string(),
                    binding_kind: "local-store-factory".to_string(),
                    imported_name: None,
                },
            );
        }
    }
    stores
}

fn binding_pattern_name(pattern: &BindingPattern<'_>) -> Option<String> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.to_string()),
        _ => None,
    }
}

fn collect_svelte_store_references(
    program: &Program<'_>,
    source: &str,
    block: &ScriptBlock<'_>,
) -> Vec<SvelteStoreReference> {
    let mut scope_collector = BindingScopeCollector::default();
    scope_collector.visit_program(program);
    let mut reference_collector = SvelteStoreReferenceCollector {
        scopes: &scope_collector.scopes,
        scope_stack: Vec::new(),
        next_scope: 0,
        references: Vec::new(),
    };
    reference_collector.visit_program(program);
    reference_collector
        .references
        .into_iter()
        .map(|reference| SvelteStoreReference {
            store_name: reference.name,
            line: line_of(source, block.start_offset + reference.span_start),
            block_kind: block.kind.clone(),
        })
        .collect()
}

#[derive(Default)]
struct BindingScopeCollector {
    scopes: Vec<BTreeSet<String>>,
    scope_stack: Vec<usize>,
}

impl<'a> Visit<'a> for BindingScopeCollector {
    fn enter_scope(&mut self, _flags: ScopeFlags, _scope_id: &Cell<Option<ScopeId>>) {
        let index = self.scopes.len();
        self.scopes.push(BTreeSet::new());
        self.scope_stack.push(index);
    }

    fn leave_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        if let Some(index) = self.scope_stack.last().copied() {
            self.scopes[index].insert(identifier.name.to_string());
        }
    }
}

struct RawSvelteStoreReference {
    name: String,
    span_start: usize,
}

struct SvelteStoreReferenceCollector<'a> {
    scopes: &'a [BTreeSet<String>],
    scope_stack: Vec<usize>,
    next_scope: usize,
    references: Vec<RawSvelteStoreReference>,
}

impl<'a> Visit<'a> for SvelteStoreReferenceCollector<'_> {
    fn enter_scope(&mut self, _flags: ScopeFlags, _scope_id: &Cell<Option<ScopeId>>) {
        self.scope_stack.push(self.next_scope);
        self.next_scope += 1;
    }

    fn leave_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        let name = identifier.name.as_str();
        let Some(store_name) = name.strip_prefix('$') else {
            return;
        };
        if store_name.is_empty() || store_name.starts_with('$') {
            return;
        }
        if self.scope_stack.iter().rev().any(|index| {
            self.scopes
                .get(*index)
                .is_some_and(|scope| scope.contains(name))
        }) {
            return;
        }
        self.references.push(RawSvelteStoreReference {
            name: store_name.to_string(),
            span_start: identifier.span.start as usize,
        });
    }
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
