use super::{binding_identifier_name_ref, code_shape::normalize_type_text, line_for_span};
use lumin_rust_common::sha256_text;
use oxc_ast::ast::{
    ArrowFunctionExpression, Declaration, ExportDefaultDeclarationKind, Expression,
    FormalParameters, Function, ModuleExportName, Program, Statement, TSTypeAnnotation,
    TSTypeParameterDeclaration,
};
use oxc_span::{GetSpan, SourceType, Span};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const NORMALIZED_VERSION: &str = "function-signature.normalized.v1";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionSignatureFact {
    pub identity: String,
    pub owner_file: String,
    pub exported_name: String,
    pub local_name: String,
    pub visibility: String,
    pub exported: bool,
    pub line: usize,
    pub normalized_signature_hash: String,
    pub signature: String,
    pub signature_param_count: usize,
    pub confidence: String,
}

#[derive(Clone, Copy)]
enum FunctionLike<'a> {
    Function(&'a Function<'a>),
    Arrow(&'a ArrowFunctionExpression<'a>),
}

impl<'a> FunctionLike<'a> {
    fn span(self) -> Span {
        match self {
            Self::Function(value) => value.span,
            Self::Arrow(value) => value.span,
        }
    }

    fn type_parameters(self) -> Option<&'a TSTypeParameterDeclaration<'a>> {
        match self {
            Self::Function(value) => value.type_parameters.as_deref(),
            Self::Arrow(value) => value.type_parameters.as_deref(),
        }
    }

    fn params(self) -> &'a FormalParameters<'a> {
        match self {
            Self::Function(value) => &value.params,
            Self::Arrow(value) => &value.params,
        }
    }

    fn return_type(self) -> Option<&'a TSTypeAnnotation<'a>> {
        match self {
            Self::Function(value) => value.return_type.as_deref(),
            Self::Arrow(value) => value.return_type.as_deref(),
        }
    }

    fn has_body(self) -> bool {
        match self {
            Self::Function(value) => value.body.is_some(),
            Self::Arrow(_) => true,
        }
    }
}

struct FunctionEntry<'a> {
    function: FunctionLike<'a>,
    local_name: String,
    exported_name: String,
    visibility: &'static str,
    exported: bool,
}

struct NormalizedSignature {
    value: Value,
    display: String,
    param_count: usize,
}

pub(super) fn looks_like_type_literal(type_literal: &str) -> bool {
    matches!(
        type_literal.trim_start().as_bytes().first(),
        Some(b'(' | b'<')
    )
}

pub(super) fn normalize_type_literal(type_literal: &str) -> Value {
    let literal = type_literal.trim().trim_end_matches(';').trim_end();
    if literal.is_empty() {
        return json!({
            "typeLiteral": type_literal,
            "ok": false,
            "reason": "empty-function-signature-literal",
        });
    }
    let source = format!("export type __IntentFunction = {literal};\n");
    let allocator = oxc_allocator::Allocator::default();
    let parsed = match super::parse_program(&allocator, &source, SourceType::ts()) {
        Ok(parsed) => parsed,
        Err(error) => {
            return json!({
                "typeLiteral": type_literal,
                "ok": false,
                "reason": "function-signature-parse-error",
                "message": error.to_string(),
            });
        }
    };
    let Some(function_type) = parsed.program.body.iter().find_map(|statement| {
        let Statement::ExportNamedDeclaration(export) = statement else {
            return None;
        };
        let Some(Declaration::TSTypeAliasDeclaration(alias)) = export.declaration.as_ref() else {
            return None;
        };
        let oxc_ast::ast::TSType::TSFunctionType(function_type) = &alias.type_annotation else {
            return None;
        };
        Some(function_type.as_ref())
    }) else {
        return json!({
            "typeLiteral": type_literal,
            "ok": false,
            "reason": "unsupported-function-signature-literal",
        });
    };
    let normalized = normalize_signature(
        function_type.type_parameters.as_deref(),
        &function_type.params,
        Some(&function_type.return_type),
        &source,
    );
    let Some(normalized) = normalized else {
        return json!({
            "typeLiteral": type_literal,
            "ok": false,
            "reason": "no-explicit-function-signature",
        });
    };
    json!({
        "typeLiteral": type_literal,
        "ok": true,
        "hash": sha256_text(&stable_json(&normalized.value)),
        "shapeKind": "function-signature",
        "signature": normalized.display,
        "evidenceCount": normalized.param_count,
    })
}

pub(super) fn collect_function_signature_facts(
    program: &Program<'_>,
    source: &str,
    owner_file: &str,
    line_starts: &[usize],
) -> Vec<FunctionSignatureFact> {
    let aliases = exported_aliases(program);
    let mut entries = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(declaration) = export.declaration.as_ref() {
                    collect_exported_declaration(declaration, &mut entries);
                }
            }
            Statement::ExportDefaultDeclaration(export) => match &export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(function)
                    if function.body.is_some() =>
                {
                    entries.push(FunctionEntry {
                        function: FunctionLike::Function(function),
                        local_name: function
                            .id
                            .as_ref()
                            .map_or_else(|| "default".to_string(), |id| id.name.to_string()),
                        exported_name: "default".to_string(),
                        visibility: "exported",
                        exported: true,
                    });
                }
                ExportDefaultDeclarationKind::Identifier(identifier) => {
                    let _ = identifier;
                }
                _ => {}
            },
            Statement::FunctionDeclaration(function) if function.body.is_some() => {
                if let Some(local_name) = function.id.as_ref().map(|id| id.name.to_string()) {
                    add_alias_or_local(
                        &mut entries,
                        FunctionLike::Function(function),
                        local_name,
                        &aliases,
                    );
                }
            }
            Statement::VariableDeclaration(declaration) => {
                collect_alias_or_local_variables(declaration, &aliases, &mut entries);
            }
            _ => {}
        }
    }

    let mut facts = entries
        .into_iter()
        .filter(|entry| entry.function.has_body())
        .filter_map(|entry| {
            let normalized = normalize_signature(
                entry.function.type_parameters(),
                entry.function.params(),
                entry.function.return_type(),
                source,
            )?;
            Some(FunctionSignatureFact {
                identity: format!("{owner_file}::{}", entry.exported_name),
                owner_file: owner_file.to_string(),
                exported_name: entry.exported_name,
                local_name: entry.local_name,
                visibility: entry.visibility.to_string(),
                exported: entry.exported,
                line: line_for_span(line_starts, entry.function.span()),
                normalized_signature_hash: sha256_text(&stable_json(&normalized.value)),
                signature: normalized.display,
                signature_param_count: normalized.param_count,
                confidence: "high".to_string(),
            })
        })
        .collect::<Vec<_>>();
    facts.sort_by(|left, right| left.identity.cmp(&right.identity));
    facts
}

fn collect_exported_declaration<'a>(
    declaration: &'a Declaration<'a>,
    entries: &mut Vec<FunctionEntry<'a>>,
) {
    match declaration {
        Declaration::FunctionDeclaration(function) if function.body.is_some() => {
            if let Some(name) = function.id.as_ref().map(|id| id.name.to_string()) {
                entries.push(FunctionEntry {
                    function: FunctionLike::Function(function),
                    local_name: name.clone(),
                    exported_name: name,
                    visibility: "exported",
                    exported: true,
                });
            }
        }
        Declaration::VariableDeclaration(declaration) => {
            for declarator in &declaration.declarations {
                let Some(local_name) = binding_identifier_name_ref(&declarator.id) else {
                    continue;
                };
                let Some(function) = declarator.init.as_ref().and_then(function_like_expression)
                else {
                    continue;
                };
                entries.push(FunctionEntry {
                    function,
                    local_name: local_name.to_string(),
                    exported_name: local_name.to_string(),
                    visibility: "exported",
                    exported: true,
                });
            }
        }
        _ => {}
    }
}

fn collect_alias_or_local_variables<'a>(
    declaration: &'a oxc_ast::ast::VariableDeclaration<'a>,
    aliases: &BTreeMap<String, BTreeSet<String>>,
    entries: &mut Vec<FunctionEntry<'a>>,
) {
    for declarator in &declaration.declarations {
        let Some(local_name) = binding_identifier_name_ref(&declarator.id) else {
            continue;
        };
        let Some(function) = declarator.init.as_ref().and_then(function_like_expression) else {
            continue;
        };
        add_alias_or_local(entries, function, local_name.to_string(), aliases);
    }
}

fn add_alias_or_local<'a>(
    entries: &mut Vec<FunctionEntry<'a>>,
    function: FunctionLike<'a>,
    local_name: String,
    aliases: &BTreeMap<String, BTreeSet<String>>,
) {
    if let Some(exported_names) = aliases.get(&local_name) {
        for exported_name in exported_names {
            entries.push(FunctionEntry {
                function,
                local_name: local_name.clone(),
                exported_name: exported_name.clone(),
                visibility: "exported",
                exported: true,
            });
        }
    } else {
        entries.push(FunctionEntry {
            function,
            exported_name: local_name.clone(),
            local_name,
            visibility: "file-local",
            exported: false,
        });
    }
}

fn exported_aliases(program: &Program<'_>) -> BTreeMap<String, BTreeSet<String>> {
    let mut aliases = BTreeMap::<String, BTreeSet<String>>::new();
    for statement in &program.body {
        match statement {
            Statement::ExportDefaultDeclaration(export) => {
                if let ExportDefaultDeclarationKind::Identifier(identifier) = &export.declaration {
                    aliases
                        .entry(identifier.name.to_string())
                        .or_default()
                        .insert("default".to_string());
                }
            }
            Statement::ExportNamedDeclaration(export)
                if export.source.is_none() && export.declaration.is_none() =>
            {
                for specifier in &export.specifiers {
                    aliases
                        .entry(module_export_name(&specifier.local))
                        .or_default()
                        .insert(module_export_name(&specifier.exported));
                }
            }
            _ => {}
        }
    }
    aliases
}

fn module_export_name(value: &ModuleExportName<'_>) -> String {
    value.name().to_string()
}

fn function_like_expression<'a>(expression: &'a Expression<'a>) -> Option<FunctionLike<'a>> {
    match expression {
        Expression::ArrowFunctionExpression(function) => Some(FunctionLike::Arrow(function)),
        Expression::FunctionExpression(function) if function.body.is_some() => {
            Some(FunctionLike::Function(function))
        }
        _ => None,
    }
}

fn normalize_signature(
    type_parameters: Option<&TSTypeParameterDeclaration<'_>>,
    params: &FormalParameters<'_>,
    return_type: Option<&TSTypeAnnotation<'_>>,
    source: &str,
) -> Option<NormalizedSignature> {
    let type_parameter_names = type_parameters
        .map(|parameters| {
            parameters
                .params
                .iter()
                .enumerate()
                .map(|(index, parameter)| (parameter.name.name.to_string(), format!("$T{index}")))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let normalized_type_parameters = type_parameters
        .map(|parameters| {
            parameters
                .params
                .iter()
                .enumerate()
                .map(|(index, parameter)| {
                    json!({
                        "name": format!("$T{index}"),
                        "constraint": parameter.constraint.as_ref().and_then(|value| {
                            normalize_type_node(value.span(), source, &type_parameter_names)
                        }),
                        "default": parameter.default.as_ref().and_then(|value| {
                            normalize_type_node(value.span(), source, &type_parameter_names)
                        }),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut normalized_params =
        Vec::with_capacity(params.items.len() + usize::from(params.rest.is_some()));
    for parameter in &params.items {
        let type_text = parameter.type_annotation.as_ref().and_then(|annotation| {
            normalize_type_node(
                annotation.type_annotation.span(),
                source,
                &type_parameter_names,
            )
        })?;
        normalized_params.push(json!({
            "rest": false,
            "optional": parameter.optional,
            "type": type_text,
        }));
    }
    if let Some(rest) = &params.rest {
        let type_text = rest.type_annotation.as_ref().and_then(|annotation| {
            normalize_type_node(
                annotation.type_annotation.span(),
                source,
                &type_parameter_names,
            )
        })?;
        normalized_params.push(json!({
            "rest": true,
            "optional": false,
            "type": type_text,
        }));
    }
    let normalized_return_type = return_type.and_then(|annotation| {
        normalize_type_node(
            annotation.type_annotation.span(),
            source,
            &type_parameter_names,
        )
    })?;
    let value = json!({
        "schemaVersion": NORMALIZED_VERSION,
        "typeParameters": normalized_type_parameters,
        "params": normalized_params,
        "returnType": normalized_return_type,
    });
    let display = signature_text(&value);
    Some(NormalizedSignature {
        value,
        display,
        param_count: params.items.len() + usize::from(params.rest.is_some()),
    })
}

fn normalize_type_node(
    span: Span,
    source: &str,
    type_parameter_names: &BTreeMap<String, String>,
) -> Option<String> {
    let raw = source.get(span.start as usize..span.end as usize)?;
    if raw.is_empty() {
        return None;
    }
    let normalized = normalize_type_text(raw);
    let renamed = replace_identifiers(&normalized, type_parameter_names);
    Some(strip_function_parameter_names(&renamed))
}

fn replace_identifiers(input: &str, replacements: &BTreeMap<String, String>) -> String {
    let mut out = String::with_capacity(input.len());
    let mut token = String::new();
    let flush = |out: &mut String, token: &mut String| {
        if token.is_empty() {
            return;
        }
        if let Some(replacement) = replacements.get(token) {
            out.push_str(replacement);
        } else {
            out.push_str(token);
        }
        token.clear();
    };
    for character in input.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '_' | '$') {
            token.push(character);
        } else {
            flush(&mut out, &mut token);
            out.push(character);
        }
    }
    flush(&mut out, &mut token);
    out
}

fn strip_function_parameter_names(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        out.push(byte);
        index += 1;
        if !matches!(byte, b'(' | b',') {
            continue;
        }
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if bytes.get(index..index + 3) == Some(b"...") {
            index += 3;
        }
        let identifier_start = index;
        if index < bytes.len() && is_identifier_start(bytes[index]) {
            index += 1;
            while index < bytes.len() && is_identifier_continue(bytes[index]) {
                index += 1;
            }
            if index == identifier_start {
                index = start;
                continue;
            }
            if bytes.get(index) == Some(&b'?') {
                index += 1;
            }
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            if bytes.get(index) == Some(&b':') {
                index += 1;
                continue;
            }
        }
        index = start;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}

fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

fn signature_text(value: &Value) -> String {
    let type_parameters = value
        .get("typeParameters")
        .and_then(Value::as_array)
        .map(|parameters| {
            parameters
                .iter()
                .map(|parameter| {
                    let mut text = parameter
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("$T")
                        .to_string();
                    if let Some(constraint) = parameter.get("constraint").and_then(Value::as_str) {
                        text.push_str(" extends ");
                        text.push_str(constraint);
                    }
                    if let Some(default) = parameter.get("default").and_then(Value::as_str) {
                        text.push_str(" = ");
                        text.push_str(default);
                    }
                    text
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let params = value
        .get("params")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|parameter| {
            format!(
                "{}{}{}",
                if parameter.get("rest").and_then(Value::as_bool) == Some(true) {
                    "..."
                } else {
                    ""
                },
                if parameter.get("optional").and_then(Value::as_bool) == Some(true) {
                    "?"
                } else {
                    ""
                },
                parameter
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let prefix = if type_parameters.is_empty() {
        String::new()
    } else {
        format!("<{}>", type_parameters.join(","))
    };
    format!(
        "{prefix}({params}):{}",
        value
            .get("returnType")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    )
}

fn stable_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{collect_function_signature_facts, normalize_type_literal};
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    #[test]
    fn normalizes_generic_function_literals_without_parameter_names() {
        let left =
            normalize_type_literal("<T extends object>(value: T, map: (item: T) => string) => T");
        let right =
            normalize_type_literal("<U extends object>(input: U, map: (other: U) => string) => U");
        assert_eq!(
            left.get("ok").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(left.get("hash"), right.get("hash"));
        assert_eq!(
            left.get("signature").and_then(serde_json::Value::as_str),
            Some("<$T0 extends object>($T0,($T0)=>string):$T0")
        );
    }

    #[test]
    fn collects_export_alias_default_and_file_local_signature_facts() {
        let source = r#"
            function local<T>(value: T): T { return value; }
            export { local as renamed };
            export const direct = (value: string): number => value.length;
            export default function fallback(value: boolean): void {}
            const privateHelper = (value: number): string => String(value);
        "#;
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty());
        let facts =
            collect_function_signature_facts(&parsed.program, source, "src/example.ts", &[0]);
        let identities = facts
            .iter()
            .map(|fact| fact.identity.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            identities,
            [
                "src/example.ts::default",
                "src/example.ts::direct",
                "src/example.ts::privateHelper",
                "src/example.ts::renamed",
            ]
        );
        assert_eq!(facts[2].visibility, "file-local");
        assert!(!facts[2].exported);
    }
}
