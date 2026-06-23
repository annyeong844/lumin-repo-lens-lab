use serde::Deserialize;

use super::input;

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationSchemaRoundTrip {
    #[serde(
        default,
        deserialize_with = "input::deserialize_js_truthy_bool_or_false"
    )]
    attempted: bool,
    #[serde(default, deserialize_with = "input::deserialize_schema_drift_bugs")]
    known_schema_drift_bugs: Vec<CalibrationSchemaDriftBug>,
}

impl CalibrationSchemaRoundTrip {
    pub(crate) fn attempted(&self) -> bool {
        self.attempted
    }

    pub(crate) fn has_known_schema_drift_bugs(&self) -> bool {
        !self.known_schema_drift_bugs.is_empty()
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
pub(in crate::calibration) struct CalibrationSchemaDriftBug {}
