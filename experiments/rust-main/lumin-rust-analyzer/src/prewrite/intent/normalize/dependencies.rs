use anyhow::Result;

use crate::prewrite::intent::input::DependencyInput;
use crate::prewrite::intent::model::DependencyDeclaration;

use super::validate::{require_non_empty, validate_optional_string};

pub(super) fn normalize_dependencies(
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
