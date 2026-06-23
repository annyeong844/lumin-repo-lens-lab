use anyhow::Result;
use lumin_rust_common::usage_error;

use crate::prewrite::intent::input::Present;
use crate::prewrite::intent::model::RefactorSource;

use super::validate::{is_unsafe_repo_relative_path, validate_optional_string};

pub(super) fn normalize_refactor_sources(
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
