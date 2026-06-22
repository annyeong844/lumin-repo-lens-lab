use anyhow::Result;
use lumin_rust_common::usage_error;

use super::input::{DependencyInput, NameInput, Present, RawIntent, ShapeIntentInput};
use super::model::{
    DependencyDeclaration, IntentKey, IntentWarning, LoadedIntent, NameDeclaration,
    NormalizedIntent, PlannedTypeEscape, RefactorSource, ShapeIntent,
};

pub(super) fn normalize(raw: RawIntent) -> Result<LoadedIntent> {
    let mut warnings = Vec::new();
    let names = required_array(raw.names, IntentKey::Names, &mut warnings);
    let shapes = required_array(raw.shapes, IntentKey::Shapes, &mut warnings);
    let files = required_array(raw.files, IntentKey::Files, &mut warnings);
    let dependencies = required_array(raw.dependencies, IntentKey::Dependencies, &mut warnings);
    let planned_type_escapes = required_array(
        raw.planned_type_escapes,
        IntentKey::PlannedTypeEscapes,
        &mut warnings,
    );

    let (names, name_declarations) = normalize_names(names)?;
    let shapes = normalize_shapes(shapes)?;
    validate_non_empty_strings(&files, "files")?;
    let (dependencies, dependency_declarations) = normalize_dependencies(dependencies)?;
    let refactor_sources = normalize_refactor_sources(raw.refactor_sources)?;
    validate_planned_type_escapes(&planned_type_escapes)?;
    let task_id = raw.task_id.0;
    if task_id.as_deref() == Some("") {
        return Err(usage_error(
            "taskId must be a non-empty string when present",
        ));
    }

    Ok(LoadedIntent {
        intent: NormalizedIntent {
            names,
            name_declarations,
            shapes,
            files,
            dependencies,
            dependency_declarations,
            refactor_sources,
            planned_type_escapes,
            task_id,
        },
        warnings,
    })
}

fn required_array<T>(
    field: Present<Vec<T>>,
    key: IntentKey,
    warnings: &mut Vec<IntentWarning>,
) -> Vec<T> {
    match field.0 {
        Some(values) => values,
        None => {
            warnings.push(IntentWarning::missing(key));
            Vec::new()
        }
    }
}

fn normalize_names(inputs: Vec<NameInput>) -> Result<(Vec<String>, Vec<NameDeclaration>)> {
    let mut names = Vec::with_capacity(inputs.len());
    let mut declarations = Vec::new();
    for (index, input) in inputs.into_iter().enumerate() {
        match input {
            NameInput::Name(name) => {
                require_non_empty(&name, &format!("names[{index}]"))?;
                names.push(name);
            }
            NameInput::Declaration(mut declaration) => {
                require_non_empty(&declaration.name, &format!("names[{index}].name"))?;
                validate_optional_string(
                    declaration.kind.as_deref(),
                    &format!("names[{index}].kind"),
                )?;
                validate_optional_string(
                    declaration.why.as_deref(),
                    &format!("names[{index}].why"),
                )?;
                validate_optional_string(
                    declaration.owner_file.as_deref(),
                    &format!("names[{index}].ownerFile"),
                )?;
                validate_optional_string(
                    declaration.file.as_deref(),
                    &format!("names[{index}].file"),
                )?;
                validate_optional_string(
                    declaration.target_file.as_deref(),
                    &format!("names[{index}].targetFile"),
                )?;
                if declaration.owner_file.is_none() {
                    declaration.owner_file = declaration
                        .file
                        .as_ref()
                        .or(declaration.target_file.as_ref())
                        .cloned();
                }
                names.push(declaration.name.clone());
                declarations.push(declaration);
            }
        }
    }
    Ok((names, declarations))
}

fn normalize_shapes(inputs: Vec<ShapeIntentInput>) -> Result<Vec<ShapeIntent>> {
    inputs
        .into_iter()
        .enumerate()
        .map(|(index, shape)| {
            if shape.fields.is_none() && shape.hash.is_none() && shape.type_literal.is_none() {
                return Err(usage_error(format!(
                    "shapes[{index}].fields must be an array"
                )));
            }
            let fields = shape.fields.unwrap_or_default();
            validate_non_empty_strings(&fields, &format!("shapes[{index}].fields"))?;
            if let Some(hash) = &shape.hash {
                if !valid_sha256(hash) {
                    return Err(usage_error(format!(
                        "shapes[{index}].hash must be sha256:<64 lowercase hex> when present"
                    )));
                }
            }
            if let Some(type_literal) = &shape.type_literal {
                if type_literal.trim().is_empty() {
                    return Err(usage_error(format!(
                        "shapes[{index}].typeLiteral must be a non-empty string when present"
                    )));
                }
            }
            validate_optional_string(shape.name.as_deref(), &format!("shapes[{index}].name"))?;
            validate_optional_string(shape.why.as_deref(), &format!("shapes[{index}].why"))?;
            Ok(ShapeIntent {
                fields,
                hash: shape.hash,
                type_literal: shape.type_literal,
                name: shape.name,
                why: shape.why,
            })
        })
        .collect()
}

fn normalize_dependencies(
    inputs: Vec<DependencyInput>,
) -> Result<(Vec<String>, Vec<DependencyDeclaration>)> {
    let mut dependencies = Vec::with_capacity(inputs.len());
    let mut declarations = Vec::new();
    for (index, input) in inputs.into_iter().enumerate() {
        match input {
            DependencyInput::Specifier(specifier) => {
                require_non_empty(&specifier, &format!("dependencies[{index}]"))?;
                dependencies.push(specifier);
            }
            DependencyInput::Declaration(declaration) => {
                require_non_empty(
                    &declaration.specifier,
                    &format!("dependencies[{index}].specifier"),
                )?;
                validate_optional_string(
                    declaration.why.as_deref(),
                    &format!("dependencies[{index}].why"),
                )?;
                dependencies.push(declaration.specifier.clone());
                declarations.push(declaration);
            }
        }
    }
    Ok((dependencies, declarations))
}

fn validate_planned_type_escapes(entries: &[PlannedTypeEscape]) -> Result<()> {
    for (index, entry) in entries.iter().enumerate() {
        require_non_empty(
            &entry.location_hint,
            &format!("plannedTypeEscapes[{index}].locationHint"),
        )?;
        require_non_empty(
            &entry.reason,
            &format!("plannedTypeEscapes[{index}].reason"),
        )?;
    }
    Ok(())
}

fn normalize_refactor_sources(
    entries: Present<Vec<RefactorSource>>,
) -> Result<Option<Vec<RefactorSource>>> {
    let Some(entries) = entries.0 else {
        return Ok(None);
    };
    for (index, entry) in entries.iter().enumerate() {
        if is_unsafe_repo_relative_path(&entry.file) {
            return Err(usage_error(format!(
                "refactorSources[{index}].file must be a repository-relative path"
            )));
        }
        if let Some(lines) = &entry.lines {
            if lines.is_empty() {
                return Err(usage_error(format!(
                    "refactorSources[{index}].lines must be a non-empty array of positive integers when present"
                )));
            }
            for (line_index, line) in lines.iter().enumerate() {
                if *line == 0 {
                    return Err(usage_error(format!(
                        "refactorSources[{index}].lines[{line_index}] must be a positive integer"
                    )));
                }
            }
        }
        validate_optional_string(
            entry.why.as_deref(),
            &format!("refactorSources[{index}].why"),
        )?;
    }
    Ok(Some(entries))
}

fn is_unsafe_repo_relative_path(value: &str) -> bool {
    if value.is_empty() || value.contains('\\') || value.starts_with('/') {
        return true;
    }
    let bytes = value.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return true;
    }
    value.split('/').any(|part| part.is_empty() || part == "..")
}

fn validate_non_empty_strings(values: &[String], path: &str) -> Result<()> {
    for (index, value) in values.iter().enumerate() {
        require_non_empty(value, &format!("{path}[{index}]"))?;
    }
    Ok(())
}

fn validate_optional_string(value: Option<&str>, path: &str) -> Result<()> {
    if value == Some("") {
        return Err(usage_error(format!(
            "{path} must be a non-empty string when present"
        )));
    }
    Ok(())
}

fn require_non_empty(value: &str, path: &str) -> Result<()> {
    if value.is_empty() {
        return Err(usage_error(format!("{path} must be a non-empty string")));
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
