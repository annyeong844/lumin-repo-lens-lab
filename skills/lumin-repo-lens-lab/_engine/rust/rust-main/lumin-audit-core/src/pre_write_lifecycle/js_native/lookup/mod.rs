use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

mod dependency;
mod file;
mod inline;
mod name;
mod shape;

pub(super) struct LookupProjection {
    pub(super) lookups: Vec<Value>,
}

pub(super) fn project(
    root: &Path,
    intent: &Value,
    evidence: &Value,
    failures: &mut Vec<Value>,
) -> Result<LookupProjection> {
    let symbols = evidence.get("symbols").unwrap_or(&Value::Null);
    let topology = evidence.get("topology").unwrap_or(&Value::Null);
    let shape_index = evidence.get("shapeIndex").unwrap_or(&Value::Null);
    let shape_normalizations = evidence
        .get("shapeIntentNormalizations")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let function_signatures = evidence.get("functionSignatures").unwrap_or(&Value::Null);
    let inline_patterns = evidence.get("inlinePatterns").unwrap_or(&Value::Null);
    let claims = name::load_canonical_claims(root)?;
    let mut lookups = Vec::new();

    for name in string_array(intent, "names") {
        let declaration = intent
            .get("nameDeclarations")
            .and_then(Value::as_array)
            .and_then(|values| {
                values
                    .iter()
                    .find(|entry| entry.get("name").and_then(Value::as_str) == Some(name))
            });
        lookups.push(name::lookup(name, symbols, &claims, declaration));
    }
    for file in string_array(intent, "files") {
        lookups.push(file::lookup(file, topology, symbols, root));
    }

    for dependency in string_array(intent, "dependencies") {
        let manifest = dependency::select_manifest(root, intent, dependency, failures)?;
        lookups.push(dependency::lookup(dependency, &manifest, symbols));
    }
    for shape in intent
        .get("shapes")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
    {
        lookups.push(shape::lookup(
            shape,
            shape_index,
            shape_normalizations,
            function_signatures,
        ));
    }
    if let Some(refactor_sources) = intent.get("refactorSources").and_then(Value::as_array) {
        if !refactor_sources.is_empty() {
            lookups.push(inline::lookup(
                refactor_sources,
                inline_patterns,
                evidence.get("files").unwrap_or(&Value::Null),
            ));
        }
    }

    Ok(LookupProjection { lookups })
}

fn string_array<'a>(value: &'a Value, key: &str) -> impl Iterator<Item = &'a str> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

pub(super) fn compute_drift(lookups: &[Value]) -> Vec<Value> {
    let mut drift = Vec::new();
    for lookup in lookups {
        if lookup.get("kind").and_then(Value::as_str) != Some("name") {
            continue;
        }
        let Some(claim) = lookup
            .get("canonicalClaim")
            .filter(|value| value.is_object())
        else {
            continue;
        };
        let status = lookup.get("canonicalAstStatus").and_then(Value::as_str);
        if !matches!(status, Some("ast-absent" | "owner-disagrees")) {
            continue;
        }
        drift.push(json!({
            "intentName": lookup.get("intentName").cloned().unwrap_or(Value::Null),
            "canonicalOwner": claim.get("ownerFile").cloned().unwrap_or(Value::Null),
            "canonicalFile": claim.get("file").cloned().unwrap_or(Value::Null),
            "canonicalLine": claim.get("line").cloned().unwrap_or(Value::Null),
            "astOwners": lookup.get("identities").and_then(Value::as_array).into_iter().flatten()
                .filter_map(|identity| identity.get("ownerFile").cloned()).collect::<Vec<_>>(),
            "kind": status.unwrap_or_default(),
        }));
    }
    drift
}

fn normalize_domain_token(token: &str) -> String {
    if token.len() > 3 && token.ends_with("ies") {
        format!("{}y", &token[..token.len() - 3])
    } else if token.len() > 3
        && token.ends_with('s')
        && !token.ends_with("ss")
        && !token.ends_with("us")
    {
        token[..token.len() - 1].to_string()
    } else {
        token.to_string()
    }
}

fn dirname(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(directory, _)| directory)
}

fn string_at<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn json_pointer_escape(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}
