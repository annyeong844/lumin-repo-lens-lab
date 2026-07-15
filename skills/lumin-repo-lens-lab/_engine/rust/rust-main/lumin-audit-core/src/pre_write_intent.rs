use anyhow::{bail, Result};
use serde_json::{json, Map, Value};

pub const ESCAPE_KINDS: &[&str] = &[
    "explicit-any",
    "as-any",
    "angle-any",
    "as-unknown-as-T",
    "rest-any-args",
    "index-sig-any",
    "generic-default-any",
    "ts-ignore",
    "ts-expect-error",
    "no-explicit-any-disable",
    "jsdoc-any",
];

const TOP_LEVEL_ARRAY_KEYS: &[&str] = &[
    "names",
    "shapes",
    "files",
    "dependencies",
    "plannedTypeEscapes",
];

#[derive(Debug, Clone)]
pub struct NormalizedJsTsIntent {
    value: Value,
    warnings: Vec<Value>,
}

impl NormalizedJsTsIntent {
    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn into_value(self) -> Value {
        self.value
    }

    pub fn warnings(&self) -> &[Value] {
        &self.warnings
    }

    pub fn string_array(&self, key: &str) -> Vec<String> {
        self.array(key)
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    }

    pub fn array(&self, key: &str) -> &[Value] {
        self.value
            .get(key)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

pub fn normalize_js_ts_intent_text(text: &str) -> Result<NormalizedJsTsIntent> {
    let value = serde_json::from_str::<Value>(text)
        .map_err(|error| anyhow::anyhow!("intent JSON parse failed: {error}"))?;
    normalize_js_ts_intent(value)
}

pub fn normalize_js_ts_intent(value: Value) -> Result<NormalizedJsTsIntent> {
    let Value::Object(mut object) = value else {
        bail!("intent schema error at \"\": intent must be a plain object");
    };
    validate_language(object.get("language"))?;

    let mut warnings = Vec::new();
    for key in TOP_LEVEL_ARRAY_KEYS {
        if !object.contains_key(*key) {
            object.insert((*key).to_string(), Value::Array(Vec::new()));
            warnings.push(json!({
                "kind": "missing-intent-key-defaulted",
                "key": key,
                "action": "defaulted-to-empty-array",
            }));
        }
    }
    for key in TOP_LEVEL_ARRAY_KEYS {
        if !object.get(*key).is_some_and(Value::is_array) {
            return schema_error(key, format!("{key} must be an array"));
        }
    }

    let (names, name_declarations) = normalize_names(required_array(&object, "names"))?;
    let shapes = normalize_shapes(required_array(&object, "shapes"))?;
    let files = normalize_string_array(required_array(&object, "files"), "files")?;
    let (dependencies, dependency_declarations) =
        normalize_dependencies(required_array(&object, "dependencies"))?;
    let planned_type_escapes =
        normalize_planned_type_escapes(required_array(&object, "plannedTypeEscapes"))?;
    let refactor_sources = match object.get("refactorSources") {
        None => None,
        Some(Value::Array(values)) => Some(normalize_refactor_sources(values)?),
        Some(_) => {
            return schema_error(
                "refactorSources",
                "refactorSources must be an array when present",
            )
        }
    };

    object.insert("names".to_string(), Value::Array(names));
    if name_declarations.is_empty() {
        object.remove("nameDeclarations");
    } else {
        object.insert(
            "nameDeclarations".to_string(),
            Value::Array(name_declarations),
        );
    }
    object.insert("shapes".to_string(), Value::Array(shapes));
    object.insert("files".to_string(), Value::Array(files));
    object.insert("dependencies".to_string(), Value::Array(dependencies));
    if dependency_declarations.is_empty() {
        object.remove("dependencyDeclarations");
    } else {
        object.insert(
            "dependencyDeclarations".to_string(),
            Value::Array(dependency_declarations),
        );
    }
    if let Some(refactor_sources) = refactor_sources {
        object.insert(
            "refactorSources".to_string(),
            Value::Array(refactor_sources),
        );
    }
    object.insert(
        "plannedTypeEscapes".to_string(),
        Value::Array(planned_type_escapes),
    );

    Ok(NormalizedJsTsIntent {
        value: Value::Object(object),
        warnings,
    })
}

fn validate_language(value: Option<&Value>) -> Result<()> {
    match value {
        None => Ok(()),
        Some(Value::String(value)) if value == "js-ts" => Ok(()),
        Some(Value::String(value)) if value == "rust" => bail!(
            "intent.language \"rust\" is owned by lumin-rust-analyzer; use the Rust pre-write owner"
        ),
        Some(_) => bail!(
            "intent schema error at \"language\": intent.language must be \"js-ts\" or omitted"
        ),
    }
}

fn required_array<'a>(object: &'a Map<String, Value>, key: &str) -> &'a [Value] {
    object
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn normalize_names(values: &[Value]) -> Result<(Vec<Value>, Vec<Value>)> {
    let mut names = Vec::with_capacity(values.len());
    let mut declarations = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let path = format!("names[{index}]");
        if let Some(name) = value.as_str() {
            if name.is_empty() {
                return schema_error(&path, format!("{path} must be a non-empty string"));
            }
            names.push(json!(name));
            continue;
        }
        let Some(entry) = value.as_object() else {
            return schema_error(
                &path,
                format!("{path} must be a non-empty string or object with name"),
            );
        };
        let name = required_nonempty_string(entry, "name", &path, false)?;
        for field in ["kind", "why", "ownerFile", "file", "targetFile"] {
            optional_nonempty_string(entry, field, &path)?;
        }
        let owner_file = entry
            .get("ownerFile")
            .or_else(|| entry.get("file"))
            .or_else(|| entry.get("targetFile"));
        let mut declaration = Map::new();
        declaration.insert("name".to_string(), json!(name));
        copy_present(entry, &mut declaration, "kind");
        copy_present(entry, &mut declaration, "why");
        if let Some(owner_file) = owner_file {
            declaration.insert("ownerFile".to_string(), owner_file.clone());
        }
        copy_present(entry, &mut declaration, "file");
        copy_present(entry, &mut declaration, "targetFile");
        names.push(json!(name));
        declarations.push(Value::Object(declaration));
    }
    Ok((names, declarations))
}

fn normalize_dependencies(values: &[Value]) -> Result<(Vec<Value>, Vec<Value>)> {
    let mut dependencies = Vec::with_capacity(values.len());
    let mut declarations = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let path = format!("dependencies[{index}]");
        if let Some(specifier) = value.as_str() {
            if specifier.is_empty() {
                return schema_error(&path, format!("{path} must be a non-empty string"));
            }
            dependencies.push(json!(specifier));
            continue;
        }
        let Some(entry) = value.as_object() else {
            return schema_error(
                &path,
                format!("{path} must be a non-empty string or object with specifier"),
            );
        };
        let specifier = required_nonempty_string(entry, "specifier", &path, false)?;
        for field in ["why", "ownerFile", "file", "targetFile"] {
            optional_nonempty_string(entry, field, &path)?;
        }
        let owner_file = entry
            .get("ownerFile")
            .or_else(|| entry.get("file"))
            .or_else(|| entry.get("targetFile"));
        if let Some(owner_file) = owner_file.and_then(Value::as_str) {
            if unsafe_repo_relative_path(owner_file) {
                return schema_error(
                    &format!("{path}.ownerFile"),
                    format!("{path}.ownerFile must be a safe repo-relative path"),
                );
            }
        }
        let mut declaration = Map::new();
        declaration.insert("specifier".to_string(), json!(specifier));
        copy_present(entry, &mut declaration, "why");
        if let Some(owner_file) = owner_file {
            declaration.insert("ownerFile".to_string(), owner_file.clone());
        }
        copy_present(entry, &mut declaration, "file");
        copy_present(entry, &mut declaration, "targetFile");
        dependencies.push(json!(specifier));
        declarations.push(Value::Object(declaration));
    }
    Ok((dependencies, declarations))
}

fn normalize_shapes(values: &[Value]) -> Result<Vec<Value>> {
    let mut shapes = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let path = format!("shapes[{index}]");
        let Some(entry) = value.as_object() else {
            return schema_error(&path, format!("{path} must be an object"));
        };
        let has_exact = entry.contains_key("hash") || entry.contains_key("typeLiteral");
        if !entry.contains_key("fields") && !has_exact {
            return schema_error(
                &format!("{path}.fields"),
                format!("{path}.fields must be an array"),
            );
        }
        let fields = match entry.get("fields") {
            None => Vec::new(),
            Some(Value::Array(fields)) => {
                normalize_string_array(fields, &format!("{path}.fields"))?
            }
            Some(_) => {
                return schema_error(
                    &format!("{path}.fields"),
                    format!("{path}.fields must be an array"),
                )
            }
        };
        if let Some(hash) = entry.get("hash") {
            if !hash.as_str().is_some_and(valid_shape_hash) {
                return schema_error(
                    &format!("{path}.hash"),
                    format!("{path}.hash must be sha256:<64 lowercase hex> when present"),
                );
            }
        }
        if let Some(type_literal) = entry.get("typeLiteral") {
            if type_literal
                .as_str()
                .is_none_or(|value| value.trim().is_empty())
            {
                return schema_error(
                    &format!("{path}.typeLiteral"),
                    format!("{path}.typeLiteral must be a non-empty string when present"),
                );
            }
        }
        optional_nonempty_string(entry, "name", &path)?;
        optional_nonempty_string(entry, "why", &path)?;
        let mut shape = Map::new();
        shape.insert("fields".to_string(), Value::Array(fields));
        for field in ["hash", "typeLiteral", "name", "why"] {
            copy_present(entry, &mut shape, field);
        }
        shapes.push(Value::Object(shape));
    }
    Ok(shapes)
}

fn normalize_refactor_sources(values: &[Value]) -> Result<Vec<Value>> {
    let mut sources = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let path = format!("refactorSources[{index}]");
        let Some(entry) = value.as_object() else {
            return schema_error(&path, format!("{path} must be an object"));
        };
        let file = entry
            .get("file")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if unsafe_repo_relative_path(file) {
            return schema_error(
                &format!("{path}.file"),
                format!("{path}.file must be a repository-relative path"),
            );
        }
        let mut source = Map::new();
        source.insert("file".to_string(), json!(file));
        if let Some(lines) = entry.get("lines") {
            let Some(lines) = lines.as_array().filter(|lines| !lines.is_empty()) else {
                return schema_error(
                    &format!("{path}.lines"),
                    format!(
                        "{path}.lines must be a non-empty array of positive integers when present"
                    ),
                );
            };
            for (line_index, line) in lines.iter().enumerate() {
                if line.as_u64().is_none_or(|line| line == 0) {
                    return schema_error(
                        &format!("{path}.lines[{line_index}]"),
                        format!("{path}.lines[{line_index}] must be a positive integer"),
                    );
                }
            }
            source.insert("lines".to_string(), Value::Array(lines.clone()));
        }
        optional_nonempty_string(entry, "why", &path)?;
        copy_present(entry, &mut source, "why");
        sources.push(Value::Object(source));
    }
    Ok(sources)
}

fn normalize_planned_type_escapes(values: &[Value]) -> Result<Vec<Value>> {
    let mut escapes = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let path = format!("plannedTypeEscapes[{index}]");
        let Some(entry) = value.as_object() else {
            return schema_error(&path, format!("{path} must be an object"));
        };
        let escape_kind = entry.get("escapeKind").and_then(Value::as_str);
        if !escape_kind.is_some_and(|kind| ESCAPE_KINDS.contains(&kind)) {
            return schema_error(
                &format!("{path}.escapeKind"),
                format!(
                    "{path}.escapeKind must be one of {}; got {}",
                    serde_json::to_string(ESCAPE_KINDS)?,
                    serde_json::to_string(entry.get("escapeKind").unwrap_or(&Value::Null))?
                ),
            );
        }
        required_nonempty_string(entry, "locationHint", &path, true)?;
        required_nonempty_string(entry, "reason", &path, true)?;
        optional_string(entry, "codeShape", &path)?;
        optional_string(entry, "alternativeConsidered", &path)?;
        let mut escape = Map::new();
        for field in [
            "escapeKind",
            "locationHint",
            "reason",
            "codeShape",
            "alternativeConsidered",
        ] {
            copy_present(entry, &mut escape, field);
        }
        escapes.push(Value::Object(escape));
    }
    Ok(escapes)
}

fn normalize_string_array(values: &[Value], path: &str) -> Result<Vec<Value>> {
    let mut normalized = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        let Some(value) = value.as_str().filter(|value| !value.is_empty()) else {
            return schema_error(
                &item_path,
                format!("{item_path} must be a non-empty string"),
            );
        };
        normalized.push(json!(value));
    }
    Ok(normalized)
}

fn required_nonempty_string<'a>(
    entry: &'a Map<String, Value>,
    field: &str,
    parent_path: &str,
    planned_escape: bool,
) -> Result<&'a str> {
    let path = format!("{parent_path}.{field}");
    let Some(value) = entry
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    else {
        let message = if planned_escape && field == "locationHint" {
            format!(
                "{path} is required and must be a non-empty string (use literal \"unknown\" when the identity is not yet known)"
            )
        } else if planned_escape && field == "reason" {
            format!(
                "{path} is required and must be a non-empty string (the intent-side half of the three-stage any-defense needs WHY)"
            )
        } else {
            format!("{path} must be a non-empty string")
        };
        return schema_error(&path, message);
    };
    Ok(value)
}

fn optional_nonempty_string(
    entry: &Map<String, Value>,
    field: &str,
    parent_path: &str,
) -> Result<()> {
    let Some(value) = entry.get(field) else {
        return Ok(());
    };
    if value.as_str().is_none_or(str::is_empty) {
        let path = format!("{parent_path}.{field}");
        return schema_error(
            &path,
            format!("{path} must be a non-empty string when present"),
        );
    }
    Ok(())
}

fn optional_string(entry: &Map<String, Value>, field: &str, parent_path: &str) -> Result<()> {
    let Some(value) = entry.get(field) else {
        return Ok(());
    };
    if !value.is_string() {
        let path = format!("{parent_path}.{field}");
        return schema_error(&path, format!("{path} must be a string when present"));
    }
    Ok(())
}

fn copy_present(source: &Map<String, Value>, target: &mut Map<String, Value>, field: &str) {
    if let Some(value) = source.get(field) {
        target.insert(field.to_string(), value.clone());
    }
}

fn valid_shape_hash(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn unsafe_repo_relative_path(value: &str) -> bool {
    value.is_empty()
        || value.contains('\\')
        || value.starts_with('/')
        || (value.len() >= 2
            && value.as_bytes()[0].is_ascii_alphabetic()
            && value.as_bytes()[1] == b':')
        || value.split('/').any(|part| part.is_empty() || part == "..")
}

fn schema_error<T>(path: &str, message: impl AsRef<str>) -> Result<T> {
    bail!("intent schema error at \"{path}\": {}", message.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_defaults_declarations_and_refactor_sources() -> Result<()> {
        let normalized = normalize_js_ts_intent(json!({
            "names": [
                "formatDate",
                {"name": "formatTimestamp", "kind": "function", "file": "src/time.ts"}
            ],
            "dependencies": [{
                "specifier": "@scope/pkg",
                "why": "boundary",
                "ownerFile": "apps/daemon/src/server.ts"
            }],
            "refactorSources": [{"file": "src/server.ts", "lines": [7, 9]}],
            "taskId": "T-1"
        }))?;
        assert_eq!(normalized.warnings().len(), 3);
        assert_eq!(
            normalized.value()["names"],
            json!(["formatDate", "formatTimestamp"])
        );
        assert_eq!(
            normalized.value()["nameDeclarations"][0]["ownerFile"],
            "src/time.ts"
        );
        assert_eq!(normalized.value()["dependencies"], json!(["@scope/pkg"]));
        assert_eq!(
            normalized.value()["dependencyDeclarations"][0]["ownerFile"],
            "apps/daemon/src/server.ts"
        );
        assert_eq!(
            normalized.value()["refactorSources"][0]["lines"],
            json!([7, 9])
        );
        assert_eq!(normalized.value()["taskId"], "T-1");
        Ok(())
    }

    #[test]
    fn accepts_exact_shapes_and_every_canonical_escape_kind() -> Result<()> {
        let escapes = ESCAPE_KINDS
            .iter()
            .map(|kind| {
                json!({
                    "escapeKind": kind,
                    "locationHint": "unknown",
                    "reason": "boundary",
                })
            })
            .collect::<Vec<_>>();
        let normalized = normalize_js_ts_intent(json!({
            "names": [],
            "shapes": [
                {"hash": format!("sha256:{}", "a".repeat(64))},
                {"typeLiteral": "{ value: string }"}
            ],
            "files": [],
            "dependencies": [],
            "plannedTypeEscapes": escapes,
        }))?;
        assert_eq!(normalized.array("shapes").len(), 2);
        assert_eq!(
            normalized.array("plannedTypeEscapes").len(),
            ESCAPE_KINDS.len()
        );
        Ok(())
    }

    #[test]
    fn rejects_malformed_fields_with_checked_error_paths() {
        for (value, path) in [
            (json!({"names": "x"}), "names"),
            (json!({"names": [{"kind": "function"}]}), "names[0].name"),
            (json!({"shapes": [{}]}), "shapes[0].fields"),
            (
                json!({"refactorSources": [{"file": "../outside.ts"}]}),
                "refactorSources[0].file",
            ),
            (
                json!({"plannedTypeEscapes": [{
                    "escapeKind": "any",
                    "locationHint": "unknown",
                    "reason": "bad",
                }]}),
                "plannedTypeEscapes[0].escapeKind",
            ),
        ] {
            let result = normalize_js_ts_intent(value);
            assert!(result.is_err(), "request for {path} must fail");
            let Some(error) = result.err() else {
                continue;
            };
            assert!(
                error.to_string().contains(&format!("at \"{path}\"")),
                "{error:#}"
            );
        }
    }
}
