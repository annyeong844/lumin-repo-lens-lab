use super::{
    binding_identifier_name, binding_identifier_name_ref, line_for_span, method_kind_text,
    module_export_identifier_name, property_key_name, visibility_text, ClassMethodRecord,
};
use oxc_ast::ast::{
    Class, ClassElement, Declaration, ExportDefaultDeclarationKind, Expression, Function,
    FunctionBody, MethodDefinition, Program, PropertyDefinition, Statement,
    VariableDeclarationKind, VariableDeclarator,
};
use oxc_span::{GetSpan, Span};
use serde_json::json;
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy)]
enum ContainerDeclaration<'a> {
    Function(&'a Function<'a>),
    Variable(&'a VariableDeclarator<'a>),
}

#[derive(Debug, Clone, Copy)]
struct FactoryContainer<'a> {
    name: &'a str,
    container_kind: &'static str,
    body: &'a FunctionBody<'a>,
}

#[derive(Debug, Clone, Copy)]
struct LocalOperationCandidate<'a> {
    name: &'a str,
    span: Span,
}

pub(super) fn collect_pre_write_local_operation_surface(
    program: &Program<'_>,
    line_starts: &[usize],
    artifact_file_path: &str,
) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    for container in collect_exported_factory_containers(program) {
        for statement in &container.body.statements {
            for candidate in local_function_candidates_from_statement(statement) {
                let Some((operation_family, domain_tokens)) = local_operation_info(candidate.name)
                else {
                    continue;
                };
                out.push(json!({
                    "identity": format!("{artifact_file_path}::{}#{}", container.name, candidate.name),
                    "name": candidate.name,
                    "ownerFile": artifact_file_path,
                    "containerName": container.name,
                    "containerKind": container.container_kind,
                    "scopeKind": "nested-function",
                    "matchedField": "preWriteLocalOperationIndex",
                    "line": line_for_span(line_starts, candidate.span),
                    "operationFamily": operation_family,
                    "domainTokens": domain_tokens,
                    "visibility": "local-only",
                    "eligibleForDeadExportRanking": false,
                    "eligibleForSafeFix": false,
                }));
            }
        }
    }
    out.sort_by_key(local_operation_sort_key);
    out
}

fn local_operation_sort_key(value: &serde_json::Value) -> String {
    format!(
        "{}|{}|{}|{:0>6}",
        value
            .get("ownerFile")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default(),
        value
            .get("containerName")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default(),
        value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default(),
        value
            .get("line")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default()
    )
}

fn collect_exported_factory_containers<'a>(program: &'a Program<'a>) -> Vec<FactoryContainer<'a>> {
    let local_declarations = collect_top_level_container_targets(program);
    let mut containers = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::ExportDefaultDeclaration(export) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(function) =
                    &export.declaration
                {
                    push_factory_container_from_function(function, &mut containers);
                }
            }
            Statement::ExportNamedDeclaration(export) if export.source.is_none() => {
                if let Some(declaration) = export.declaration.as_ref() {
                    push_factory_containers_from_declaration(declaration, &mut containers);
                    continue;
                }
                for specifier in &export.specifiers {
                    let Some(local_name) = module_export_identifier_name(&specifier.local) else {
                        continue;
                    };
                    match local_declarations.get(local_name.as_str()) {
                        Some(ContainerDeclaration::Function(function)) => {
                            push_factory_container_from_function(function, &mut containers);
                        }
                        Some(ContainerDeclaration::Variable(declarator)) => {
                            push_factory_container_from_declarator(
                                declarator,
                                VariableDeclarationKind::Const,
                                &mut containers,
                            );
                        }
                        None => {}
                    }
                }
            }
            _ => {}
        }
    }
    containers.sort_by(|left, right| {
        format!("{}|{}", left.name, left.container_kind)
            .cmp(&format!("{}|{}", right.name, right.container_kind))
    });
    containers
}

fn collect_top_level_container_targets<'a>(
    program: &'a Program<'a>,
) -> BTreeMap<String, ContainerDeclaration<'a>> {
    let mut out = BTreeMap::new();
    for statement in &program.body {
        match statement {
            Statement::FunctionDeclaration(function) => {
                if let Some(id) = function.id.as_ref() {
                    out.entry(id.name.to_string())
                        .or_insert(ContainerDeclaration::Function(function));
                }
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    if binding_identifier_name(&declarator.id).is_some() {
                        insert_container_declarator(declarator, &mut out);
                    }
                }
            }
            Statement::ExportNamedDeclaration(export) if export.source.is_none() => {
                if let Some(declaration) = export.declaration.as_ref() {
                    collect_container_declaration(declaration, &mut out);
                }
            }
            _ => {}
        }
    }
    out
}

fn collect_container_declaration<'a>(
    declaration: &'a Declaration<'a>,
    out: &mut BTreeMap<String, ContainerDeclaration<'a>>,
) {
    match declaration {
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = function.id.as_ref() {
                out.entry(id.name.to_string())
                    .or_insert(ContainerDeclaration::Function(function));
            }
        }
        Declaration::VariableDeclaration(declaration) => {
            for declarator in &declaration.declarations {
                insert_container_declarator(declarator, out);
            }
        }
        _ => {}
    }
}

fn insert_container_declarator<'a>(
    declarator: &'a VariableDeclarator<'a>,
    out: &mut BTreeMap<String, ContainerDeclaration<'a>>,
) {
    if let Some(name) = binding_identifier_name(&declarator.id) {
        out.entry(name)
            .or_insert(ContainerDeclaration::Variable(declarator));
    }
}

fn push_factory_containers_from_declaration<'a>(
    declaration: &'a Declaration<'a>,
    containers: &mut Vec<FactoryContainer<'a>>,
) {
    match declaration {
        Declaration::FunctionDeclaration(function) => {
            push_factory_container_from_function(function, containers);
        }
        Declaration::VariableDeclaration(declaration) => {
            for declarator in &declaration.declarations {
                push_factory_container_from_declarator(declarator, declaration.kind, containers);
            }
        }
        _ => {}
    }
}

fn push_factory_container_from_function<'a>(
    function: &'a Function<'a>,
    containers: &mut Vec<FactoryContainer<'a>>,
) {
    let Some(id) = function.id.as_ref() else {
        return;
    };
    let Some(body) = function.body.as_deref() else {
        return;
    };
    let name = id.name.as_str();
    if is_local_operation_container_name(name) {
        containers.push(FactoryContainer {
            name,
            container_kind: "function-declaration",
            body,
        });
    }
}

fn push_factory_container_from_declarator<'a>(
    declarator: &'a VariableDeclarator<'a>,
    declaration_kind: VariableDeclarationKind,
    containers: &mut Vec<FactoryContainer<'a>>,
) {
    if declaration_kind != VariableDeclarationKind::Const {
        return;
    }
    let Some(name) = binding_identifier_name_ref(&declarator.id) else {
        return;
    };
    let Some((container_kind, body)) = function_like_init_body(declarator.init.as_ref()) else {
        return;
    };
    if is_local_operation_container_name(name) {
        containers.push(FactoryContainer {
            name,
            container_kind,
            body,
        });
    }
}

fn function_like_init_body<'a>(
    expression: Option<&'a Expression<'a>>,
) -> Option<(&'static str, &'a FunctionBody<'a>)> {
    match expression {
        Some(Expression::FunctionExpression(function)) => function
            .body
            .as_deref()
            .map(|body| ("const-function-expression", body)),
        Some(Expression::ArrowFunctionExpression(arrow)) if !arrow.expression => {
            Some(("const-arrow-function", arrow.body.as_ref()))
        }
        _ => None,
    }
}

fn local_function_candidates_from_statement<'a>(
    statement: &'a Statement<'a>,
) -> Vec<LocalOperationCandidate<'a>> {
    match statement {
        Statement::FunctionDeclaration(function) => function
            .id
            .as_ref()
            .map(|id| {
                vec![LocalOperationCandidate {
                    name: id.name.as_str(),
                    span: id.span,
                }]
            })
            .unwrap_or_default(),
        Statement::VariableDeclaration(declaration)
            if declaration.kind == VariableDeclarationKind::Const =>
        {
            declaration
                .declarations
                .iter()
                .filter_map(|declarator| {
                    if !matches!(
                        declarator.init.as_ref(),
                        Some(
                            Expression::FunctionExpression(_)
                                | Expression::ArrowFunctionExpression(_)
                        )
                    ) {
                        return None;
                    }
                    binding_identifier_name_ref(&declarator.id).map(|name| {
                        LocalOperationCandidate {
                            name,
                            span: declarator.id.span(),
                        }
                    })
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn is_local_operation_container_name(name: &str) -> bool {
    let tokens = unique_pre_write_tokens(name);
    tokens
        .first()
        .is_some_and(|token| matches!(token.as_str(), "build" | "create" | "make"))
        && tokens
            .iter()
            .any(|token| matches!(token.as_str(), "repository" | "service"))
}

fn local_operation_info(name: &str) -> Option<(&'static str, Vec<String>)> {
    let tokens = unique_pre_write_tokens(name);
    let verb = tokens.first()?;
    if !is_local_operation_read_query_verb(verb) || is_local_operation_mutation_verb(verb) {
        return None;
    }
    let domain_tokens = tokens
        .into_iter()
        .skip(1)
        .filter(|token| {
            !token.is_empty()
                && !is_local_operation_read_query_verb(token)
                && !is_local_operation_mutation_verb(token)
        })
        .collect::<Vec<_>>();
    (!domain_tokens.is_empty()).then_some(("read-query", domain_tokens))
}

fn is_local_operation_read_query_verb(token: &str) -> bool {
    matches!(
        token,
        "fetch"
            | "find"
            | "get"
            | "list"
            | "load"
            | "lookup"
            | "query"
            | "read"
            | "resolve"
            | "retrieve"
            | "search"
    )
}

fn is_local_operation_mutation_verb(token: &str) -> bool {
    matches!(
        token,
        "add"
            | "create"
            | "delete"
            | "destroy"
            | "dispatch"
            | "emit"
            | "patch"
            | "remove"
            | "save"
            | "send"
            | "set"
            | "update"
            | "upsert"
            | "write"
    )
}

fn unique_pre_write_tokens(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    for token in tokenize_pre_write(value) {
        if !out.contains(&token) {
            out.push(token);
        }
    }
    out
}

fn tokenize_pre_write(value: &str) -> Vec<String> {
    let chars = value.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut current = String::new();
    for (index, ch) in chars.iter().copied().enumerate() {
        if !ch.is_ascii_alphanumeric() {
            push_normalized_token(&mut tokens, &mut current);
            continue;
        }
        if !current.is_empty()
            && should_split_pre_write_token(chars[index - 1], ch, chars.get(index + 1).copied())
        {
            push_normalized_token(&mut tokens, &mut current);
        }
        current.push(ch);
    }
    push_normalized_token(&mut tokens, &mut current);
    tokens
}

fn should_split_pre_write_token(prev: char, current: char, next: Option<char>) -> bool {
    (prev.is_ascii_uppercase()
        && current.is_ascii_uppercase()
        && next.is_some_and(|next| next.is_ascii_lowercase()))
        || ((prev.is_ascii_lowercase() || prev.is_ascii_digit()) && current.is_ascii_uppercase())
        || (prev.is_ascii_alphabetic() && current.is_ascii_digit())
        || (prev.is_ascii_digit() && current.is_ascii_alphabetic())
}

fn push_normalized_token(tokens: &mut Vec<String>, current: &mut String) {
    if current.is_empty() {
        return;
    }
    let normalized = normalize_pre_write_token(current);
    if !normalized.is_empty() {
        tokens.push(normalized);
    }
    current.clear();
}

fn normalize_pre_write_token(token: &str) -> String {
    let lower = token.to_ascii_lowercase();
    match lower.as_str() {
        "artifacts" => "artifact".to_string(),
        "rel" => "relative".to_string(),
        "ctx" => "context".to_string(),
        "cfg" => "config".to_string(),
        "config" => "configuration".to_string(),
        "exists" | "existing" | "existence" => "exist".to_string(),
        "series" | "species" => lower,
        _ if lower.len() > 4 && lower.ends_with("ies") => {
            format!("{}y", &lower[..lower.len() - 3])
        }
        _ => lower,
    }
}

pub(super) fn collect_class_method_surface(
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
        kind: "ClassMethod".to_string(),
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
        kind: "ClassMethod".to_string(),
        member_kind: "class-field-function".to_string(),
        visibility: visibility_text(property.accessibility, &property.key),
        r#static: property.r#static,
        computed: property.computed,
        line,
        end_line: (end_line != line).then_some(end_line),
    })
}
