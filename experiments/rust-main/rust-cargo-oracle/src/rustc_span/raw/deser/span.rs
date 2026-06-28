use serde::Deserialize;

use super::expansion::{expansion_field, PresentExpansion};
use super::fields::{
    bool_or_false, optional_file_name, optional_i64, presence_applicability, presence_string,
    PresentApplicability, PresentString,
};
use crate::rustc_span::raw::model::RustcSpan;

#[derive(Debug, Deserialize)]
pub(super) struct RawRustcSpan {
    #[serde(default, deserialize_with = "optional_file_name")]
    file_name: Option<String>,
    #[serde(default, deserialize_with = "optional_i64")]
    line_start: Option<i64>,
    #[serde(default, deserialize_with = "optional_i64")]
    line_end: Option<i64>,
    #[serde(default, deserialize_with = "optional_i64")]
    column_start: Option<i64>,
    #[serde(default, deserialize_with = "optional_i64")]
    column_end: Option<i64>,
    #[serde(default, deserialize_with = "bool_or_false")]
    is_primary: bool,
    #[serde(default, deserialize_with = "presence_applicability")]
    suggestion_applicability: PresentApplicability,
    #[serde(default, deserialize_with = "presence_string")]
    suggested_replacement: PresentString,
    #[serde(default, deserialize_with = "expansion_field")]
    expansion: PresentExpansion,
}

impl From<RawRustcSpan> for RustcSpan {
    fn from(raw: RawRustcSpan) -> Self {
        Self {
            file_name: raw.file_name,
            line_start: raw.line_start,
            line_end: raw.line_end,
            column_start: raw.column_start,
            column_end: raw.column_end,
            is_primary: raw.is_primary,
            suggestion_applicability: raw.suggestion_applicability.value,
            suggested_replacement: raw.suggested_replacement.value,
            has_suggestion_applicability_field: raw.suggestion_applicability.present,
            has_suggested_replacement_field: raw.suggested_replacement.present,
            has_expansion: raw.expansion.present_non_null,
            expansion: raw.expansion.value.map(Box::new),
        }
    }
}
