use serde::Deserialize;

use super::super::target::CargoJsonTargetData;
use super::reason::CargoJsonReason;
use crate::rustc_diagnostic::RustcDiagnostic;

#[derive(Debug, Deserialize)]
pub(super) struct RawCargoJsonRecord {
    reason: Option<String>,
    pub(super) package_id: Option<String>,
    #[serde(default, deserialize_with = "optional_message")]
    pub(super) message: Option<RawCargoJsonMessage>,
    pub(super) target: Option<CargoJsonTargetData>,
    pub(super) success: Option<bool>,
}

impl RawCargoJsonRecord {
    pub(super) fn reason(&self) -> Option<CargoJsonReason> {
        self.reason.as_deref().and_then(CargoJsonReason::from_str)
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct RawCargoJsonMessage {
    #[serde(default, deserialize_with = "optional_target")]
    pub(super) target: Option<CargoJsonTargetData>,
    #[serde(flatten)]
    pub(super) diagnostic: RustcDiagnostic,
}

fn optional_message<'de, D>(deserializer: D) -> Result<Option<RawCargoJsonMessage>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<RawCargoJsonMessage>::deserialize(deserializer)
        .ok()
        .flatten())
}

fn optional_target<'de, D>(deserializer: D) -> Result<Option<CargoJsonTargetData>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<CargoJsonTargetData>::deserialize(deserializer)
        .ok()
        .flatten())
}
