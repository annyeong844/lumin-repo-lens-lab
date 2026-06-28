use anyhow::Result;
use lumin_rust_common::usage_error;

use super::input::{Present, RawIntent};
use super::model::{IntentKey, IntentWarning, LoadedIntent, NormalizedIntent};
use dependencies::normalize_dependencies;
use names::normalize_names;
use planned_type_escapes::validate_planned_type_escapes;
use refactor_sources::normalize_refactor_sources;
use shapes::normalize_shapes;
use validate::validate_non_empty_strings;

mod dependencies;
mod names;
mod planned_type_escapes;
mod refactor_sources;
mod shapes;
mod validate;

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
