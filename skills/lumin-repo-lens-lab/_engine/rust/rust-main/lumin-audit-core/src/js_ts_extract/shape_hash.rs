use super::{code_shape::normalize_type_text, line_for_span};
use lumin_rust_common::sha256_text;
use oxc_ast::ast::{
    Declaration, ModuleExportName, Program, PropertyKey, Statement, TSInterfaceDeclaration,
    TSLiteral, TSSignature, TSType, TSTypeAliasDeclaration,
};
use oxc_span::{GetSpan, Span};
use serde_json::{json, Number, Value};
use std::collections::{BTreeMap, BTreeSet};

const NORMALIZED_VERSION: &str = "shape-hash.normalized.v1";

#[derive(Clone, Copy)]
enum ShapeDeclaration<'a> {
    Interface(&'a TSInterfaceDeclaration<'a>),
    TypeAlias(&'a TSTypeAliasDeclaration<'a>),
}

impl<'a> ShapeDeclaration<'a> {
    fn span(self) -> Span {
        match self {
            Self::Interface(value) => value.span,
            Self::TypeAlias(value) => value.span,
        }
    }

    fn local_name(self) -> &'a str {
        match self {
            Self::Interface(value) => value.id.name.as_str(),
            Self::TypeAlias(value) => value.id.name.as_str(),
        }
    }

    fn type_kind(self) -> &'static str {
        match self {
            Self::Interface(_) => "TSInterfaceDeclaration",
            Self::TypeAlias(_) => "TSTypeAliasDeclaration",
        }
    }

    fn has_type_parameters(self) -> bool {
        match self {
            Self::Interface(value) => value
                .type_parameters
                .as_ref()
                .is_some_and(|parameters| !parameters.params.is_empty()),
            Self::TypeAlias(value) => value
                .type_parameters
                .as_ref()
                .is_some_and(|parameters| !parameters.params.is_empty()),
        }
    }
}

#[derive(Clone, Copy)]
struct ShapeEntry<'a> {
    declaration: ShapeDeclaration<'a>,
    exported_name: &'a str,
}

struct NormalizedShape {
    shape_kind: &'static str,
    fields: Vec<Value>,
    literals: Vec<Value>,
    normalized: Value,
}

type ShapeNormalization = Result<NormalizedShape, Vec<Value>>;

pub(super) fn collect_shape_hash_facts(
    program: &Program<'_>,
    source: &str,
    owner_file: &str,
    line_starts: &[usize],
) -> (Vec<Value>, Vec<Value>) {
    let aliases = exported_aliases(program);
    let mut entries = BTreeMap::<String, Vec<ShapeEntry<'_>>>::new();

    for statement in &program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(declaration) = export.declaration.as_ref().and_then(shape_declaration) {
                    add_entry(
                        &mut entries,
                        owner_file,
                        declaration,
                        declaration.local_name(),
                    );
                }
            }
            Statement::TSInterfaceDeclaration(value) => {
                add_alias_entries(
                    &mut entries,
                    owner_file,
                    ShapeDeclaration::Interface(value),
                    &aliases,
                );
            }
            Statement::TSTypeAliasDeclaration(value) => {
                add_alias_entries(
                    &mut entries,
                    owner_file,
                    ShapeDeclaration::TypeAlias(value),
                    &aliases,
                );
            }
            _ => {}
        }
    }

    let mut facts = Vec::new();
    let mut diagnostics = Vec::new();
    for (identity, identity_entries) in entries {
        if identity_entries.len() > 1 {
            let first = identity_entries[0];
            diagnostics.push(diagnostic(
                "declaration-merge-unsupported",
                "multiple exported type declarations share one identity; TS declaration merging is deferred",
                json!({
                    "file": owner_file,
                    "identity": identity,
                    "exportedName": first.exported_name,
                    "declarations": identity_entries.iter().map(|entry| json!({
                        "line": line_for_span(line_starts, entry.declaration.span()),
                        "type": entry.declaration.type_kind(),
                    })).collect::<Vec<_>>(),
                }),
            ));
            continue;
        }

        let entry = identity_entries[0];
        let line = line_for_span(line_starts, entry.declaration.span());
        match normalize_declaration(entry.declaration, source, &identity) {
            Ok(normalized) => {
                let hash = sha256_text(&stable_json(&normalized.normalized));
                let generated_file = generated_file_evidence(owner_file, source);
                let mut fact = json!({
                    "kind": "shape-hash",
                    "hash": hash,
                    "identities": [identity],
                    "identity": identity,
                    "exportedName": entry.exported_name,
                    "ownerFile": owner_file,
                    "typeKind": entry.declaration.type_kind(),
                    "shapeKind": normalized.shape_kind,
                    "line": line,
                    "fields": normalized.fields,
                    "source": "fresh-ast-pass",
                    "confidence": "high",
                });
                if !normalized.literals.is_empty() {
                    fact["literals"] = Value::Array(normalized.literals);
                }
                if let Some(generated_file) = generated_file {
                    fact["generatedFile"] = generated_file;
                }
                facts.push(fact);
            }
            Err(mut declaration_diagnostics) => {
                for value in &mut declaration_diagnostics {
                    if let Some(object) = value.as_object_mut() {
                        object.insert("ownerFile".to_string(), json!(owner_file));
                        object.insert("exportedName".to_string(), json!(entry.exported_name));
                        object.insert("identity".to_string(), json!(identity));
                    }
                }
                diagnostics.extend(declaration_diagnostics);
            }
        }
    }
    (facts, diagnostics)
}

fn add_alias_entries<'a>(
    entries: &mut BTreeMap<String, Vec<ShapeEntry<'a>>>,
    owner_file: &str,
    declaration: ShapeDeclaration<'a>,
    aliases: &'a BTreeMap<String, BTreeSet<String>>,
) {
    let Some(exported_names) = aliases.get(declaration.local_name()) else {
        return;
    };
    for exported_name in exported_names {
        add_entry(entries, owner_file, declaration, exported_name);
    }
}

fn add_entry<'a>(
    entries: &mut BTreeMap<String, Vec<ShapeEntry<'a>>>,
    owner_file: &str,
    declaration: ShapeDeclaration<'a>,
    exported_name: &'a str,
) {
    entries
        .entry(format!("{owner_file}::{exported_name}"))
        .or_default()
        .push(ShapeEntry {
            declaration,
            exported_name,
        });
}

fn exported_aliases(program: &Program<'_>) -> BTreeMap<String, BTreeSet<String>> {
    let mut aliases = BTreeMap::<String, BTreeSet<String>>::new();
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.source.is_some() || export.declaration.is_some() {
            continue;
        }
        for specifier in &export.specifiers {
            let (Some(local), Some(exported)) = (
                module_export_name(&specifier.local),
                module_export_name(&specifier.exported),
            ) else {
                continue;
            };
            aliases.entry(local).or_default().insert(exported);
        }
    }
    aliases
}

fn module_export_name(value: &ModuleExportName<'_>) -> Option<String> {
    match value {
        ModuleExportName::IdentifierName(value) => Some(value.name.to_string()),
        ModuleExportName::IdentifierReference(value) => Some(value.name.to_string()),
        ModuleExportName::StringLiteral(_) => None,
    }
}

fn shape_declaration<'a>(declaration: &'a Declaration<'a>) -> Option<ShapeDeclaration<'a>> {
    match declaration {
        Declaration::TSInterfaceDeclaration(value) => Some(ShapeDeclaration::Interface(value)),
        Declaration::TSTypeAliasDeclaration(value) => Some(ShapeDeclaration::TypeAlias(value)),
        _ => None,
    }
}

fn normalize_declaration(
    declaration: ShapeDeclaration<'_>,
    source: &str,
    identity: &str,
) -> ShapeNormalization {
    if declaration.has_type_parameters() {
        return Err(vec![diagnostic(
            "unsupported-type-parameters",
            "generic type declarations are deferred until checker-grade shape support",
            json!({ "identity": identity }),
        )]);
    }

    if let ShapeDeclaration::TypeAlias(alias) = declaration {
        if let Some(result) = normalize_literal_alias(&alias.type_annotation, source, identity) {
            return result;
        }
    }

    let members = match declaration {
        ShapeDeclaration::Interface(value) => Some(value.body.body.as_slice()),
        ShapeDeclaration::TypeAlias(value) => match &value.type_annotation {
            TSType::TSTypeLiteral(literal) => Some(literal.members.as_slice()),
            _ => None,
        },
    };
    let Some(members) = members else {
        return Err(vec![diagnostic(
            "unsupported-declaration-shape",
            &format!("unsupported declaration shape: {}", declaration.type_kind()),
            json!({ "identity": identity }),
        )]);
    };

    let mut fields = Vec::new();
    let mut diagnostics = Vec::new();
    for member in members {
        match member {
            TSSignature::TSPropertySignature(property) => {
                let Some(name) = property_key_name(&property.key, property.computed, source) else {
                    diagnostics.push(diagnostic(
                        "unsupported-member-key",
                        "unsupported or computed member key",
                        json!({ "identity": identity }),
                    ));
                    continue;
                };
                let Some(annotation) = &property.type_annotation else {
                    diagnostics.push(diagnostic(
                        "missing-type-annotation",
                        &format!("missing type annotation for field {name}"),
                        json!({ "identity": identity }),
                    ));
                    continue;
                };
                fields.push(json!({
                    "kind": "property",
                    "name": name,
                    "optional": property.optional,
                    "readonly": property.readonly,
                    "type": normalize_type_text(source_slice(source, annotation.type_annotation.span())),
                }));
            }
            TSSignature::TSMethodSignature(method) => {
                let Some(name) = property_key_name(&method.key, method.computed, source) else {
                    diagnostics.push(diagnostic(
                        "unsupported-member-key",
                        "unsupported or computed member key",
                        json!({ "identity": identity }),
                    ));
                    continue;
                };
                fields.push(json!({
                    "kind": "method",
                    "name": name,
                    "optional": method.optional,
                    "readonly": false,
                    "type": format!(
                        "method{}",
                        normalize_type_text(source_slice(
                            source,
                            Span::new(method.key.span().end, method.span.end),
                        )),
                    ),
                }));
            }
            other => diagnostics.push(diagnostic(
                "unsupported-member-kind",
                &format!("unsupported member kind: {}", signature_kind(other)),
                json!({ "identity": identity }),
            )),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    fields.sort_by(|left, right| {
        value_text(left, "name")
            .cmp(value_text(right, "name"))
            .then_with(|| value_text(left, "kind").cmp(value_text(right, "kind")))
            .then_with(|| stable_json(left).cmp(&stable_json(right)))
    });
    let normalized = json!({
        "schemaVersion": NORMALIZED_VERSION,
        "shapeKind": "object",
        "fields": fields,
    });
    Ok(NormalizedShape {
        shape_kind: "object",
        fields: normalized["fields"].as_array().cloned().unwrap_or_default(),
        literals: Vec::new(),
        normalized,
    })
}

fn normalize_literal_alias(
    annotation: &TSType<'_>,
    source: &str,
    identity: &str,
) -> Option<ShapeNormalization> {
    let members = match annotation {
        TSType::TSUnionType(union) => union.types.iter().collect::<Vec<_>>(),
        TSType::TSLiteralType(_)
        | TSType::TSNullKeyword(_)
        | TSType::TSUndefinedKeyword(_)
        | TSType::TSTemplateLiteralType(_) => vec![annotation],
        _ => return None,
    };
    let mut literals = BTreeMap::<String, Value>::new();
    let mut diagnostics = Vec::new();
    for member in members {
        match normalize_literal(member, source) {
            Some(literal) => {
                literals.insert(stable_json(&literal), literal);
            }
            None => diagnostics.push(diagnostic(
                "unsupported-literal-union-member",
                &format!(
                    "unsupported literal union member: {}",
                    source_slice(source, member.span())
                ),
                json!({ "identity": identity }),
            )),
        }
    }
    if !diagnostics.is_empty() {
        return Some(Err(diagnostics));
    }
    let literals = literals.into_values().collect::<Vec<_>>();
    let normalized = json!({
        "schemaVersion": NORMALIZED_VERSION,
        "shapeKind": "literal-union",
        "literals": literals,
    });
    Some(Ok(NormalizedShape {
        shape_kind: "literal-union",
        fields: Vec::new(),
        literals: normalized["literals"]
            .as_array()
            .cloned()
            .unwrap_or_default(),
        normalized,
    }))
}

fn normalize_literal(value: &TSType<'_>, _source: &str) -> Option<Value> {
    match value {
        TSType::TSNullKeyword(_) => Some(json!({ "kind": "null", "value": null })),
        TSType::TSUndefinedKeyword(_) => Some(json!({ "kind": "undefined", "value": null })),
        TSType::TSTemplateLiteralType(template)
            if template.types.is_empty() && template.quasis.len() == 1 =>
        {
            let quasi = &template.quasis[0].value;
            Some(json!({
                "kind": "string",
                "value": quasi.cooked.as_ref().unwrap_or(&quasi.raw).as_str(),
            }))
        }
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::BooleanLiteral(value) => {
                Some(json!({ "kind": "boolean", "value": value.value }))
            }
            TSLiteral::StringLiteral(value) => {
                Some(json!({ "kind": "string", "value": value.value.as_str() }))
            }
            TSLiteral::NumericLiteral(value) => Some(json!({
                "kind": "number",
                "value": json_number(value.value)?,
            })),
            TSLiteral::BigIntLiteral(value) => {
                Some(json!({ "kind": "bigint", "value": value.value.as_str() }))
            }
            TSLiteral::TemplateLiteral(value)
                if value.expressions.is_empty() && value.quasis.len() == 1 =>
            {
                let quasi = &value.quasis[0].value;
                Some(json!({
                    "kind": "string",
                    "value": quasi.cooked.as_ref().unwrap_or(&quasi.raw).as_str(),
                }))
            }
            TSLiteral::TemplateLiteral(_) => None,
            TSLiteral::UnaryExpression(_) => None,
        },
        _ => None,
    }
}

fn json_number(value: f64) -> Option<Number> {
    if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
        Some(Number::from(value as i64))
    } else {
        Number::from_f64(value)
    }
}

fn property_key_name(key: &PropertyKey<'_>, computed: bool, source: &str) -> Option<String> {
    if computed {
        return None;
    }
    match key {
        PropertyKey::StaticIdentifier(value) => Some(value.name.to_string()),
        PropertyKey::PrivateIdentifier(value) => Some(value.name.to_string()),
        PropertyKey::Identifier(value) => Some(value.name.to_string()),
        PropertyKey::StringLiteral(value) => Some(value.value.to_string()),
        PropertyKey::NumericLiteral(value) => Some(value.value.to_string()),
        other => {
            let raw = source_slice(source, other.span());
            (!raw.is_empty()).then(|| raw.trim_matches(['\'', '"']).to_string())
        }
    }
}

fn signature_kind(value: &TSSignature<'_>) -> &'static str {
    match value {
        TSSignature::TSIndexSignature(_) => "TSIndexSignature",
        TSSignature::TSPropertySignature(_) => "TSPropertySignature",
        TSSignature::TSCallSignatureDeclaration(_) => "TSCallSignatureDeclaration",
        TSSignature::TSConstructSignatureDeclaration(_) => "TSConstructSignatureDeclaration",
        TSSignature::TSMethodSignature(_) => "TSMethodSignature",
    }
}

fn diagnostic(code: &str, message: &str, extra: Value) -> Value {
    let mut value = json!({
        "kind": "shape-hash-diagnostic",
        "code": code,
        "severity": if code == "parse-error" { "error" } else { "warning" },
        "message": message,
    });
    if let (Some(target), Some(extra)) = (value.as_object_mut(), extra.as_object()) {
        target.extend(extra.clone());
    }
    value
}

fn source_slice(source: &str, span: Span) -> &str {
    source
        .get(span.start as usize..span.end as usize)
        .unwrap_or("")
}

fn stable_json(value: &Value) -> String {
    value.to_string()
}

fn value_text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

pub(super) fn generated_file_evidence(file: &str, source: &str) -> Option<Value> {
    let normalized = file.replace('\\', "/");
    if has_path_segment(&normalized, "__generated__") || has_path_segment(&normalized, "generated")
    {
        return Some(json!({
            "kind": "generated-file",
            "source": "path",
            "evidence": "path:generated-directory",
        }));
    }
    let name = normalized.rsplit('/').next().unwrap_or(&normalized);
    if name == "routeTree.gen.ts"
        || name == "routeTree.gen.tsx"
        || name == "routeTree.gen.js"
        || name == "routeTree.gen.jsx"
        || name == "routeTree.gen.mjs"
        || name == "routeTree.gen.cjs"
    {
        return Some(json!({
            "kind": "generated-file",
            "source": "path",
            "evidence": "path:routeTree.gen",
        }));
    }
    if generated_suffix(name) {
        return Some(json!({
            "kind": "generated-file",
            "source": "path",
            "evidence": "path:generated-suffix",
        }));
    }
    let header = source
        .chars()
        .take(2048)
        .collect::<String>()
        .to_ascii_lowercase();
    if contains_bounded_marker(&header, "@generated", false, true)
        || contains_bounded_marker(&header, "<auto-generated", false, true)
        || contains_bounded_marker(&header, "auto-generated", true, true)
        || contains_bounded_marker(&header, "generated by", true, true)
        || contains_bounded_marker(&header, "this file is generated", true, true)
    {
        return Some(json!({
            "kind": "generated-file",
            "source": "header",
            "evidence": "header:generated-marker",
        }));
    }
    None
}

fn has_path_segment(path: &str, segment: &str) -> bool {
    path.split('/').any(|part| part == segment)
}

fn contains_bounded_marker(
    text: &str,
    marker: &str,
    require_start_boundary: bool,
    require_end_boundary: bool,
) -> bool {
    text.match_indices(marker).any(|(start, _)| {
        let before = text[..start].chars().next_back();
        let after = text[start + marker.len()..].chars().next();
        (!require_start_boundary || before.is_none_or(|value| !is_word_character(value)))
            && (!require_end_boundary || after.is_none_or(|value| !is_word_character(value)))
    })
}

fn is_word_character(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_'
}

fn generated_suffix(name: &str) -> bool {
    const EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mjs", "cjs"];
    EXTENSIONS.iter().any(|extension| {
        [
            format!(".gen.{extension}"),
            format!(".generated.{extension}"),
            format!(".gen.d.{extension}"),
            format!(".generated.d.{extension}"),
        ]
        .iter()
        .any(|suffix| name.ends_with(suffix))
    })
}
