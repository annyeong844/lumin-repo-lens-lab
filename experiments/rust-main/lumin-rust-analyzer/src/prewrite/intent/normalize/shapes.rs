use anyhow::Result;
use lumin_rust_common::usage_error;

use crate::prewrite::intent::input::ShapeIntentInput;
use crate::prewrite::intent::model::ShapeIntent;

use super::validate::{valid_sha256, validate_non_empty_strings, validate_optional_string};

pub(super) fn normalize_shapes(inputs: Vec<ShapeIntentInput>) -> Result<Vec<ShapeIntent>> {
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
