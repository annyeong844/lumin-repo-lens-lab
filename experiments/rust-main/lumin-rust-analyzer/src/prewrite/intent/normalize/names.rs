use anyhow::Result;

use crate::prewrite::intent::input::NameInput;
use crate::prewrite::intent::model::NameDeclaration;

use super::validate::{require_non_empty, validate_optional_string};

pub(super) fn normalize_names(
    inputs: Vec<NameInput>,
) -> Result<(Vec<String>, Vec<NameDeclaration>)> {
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
