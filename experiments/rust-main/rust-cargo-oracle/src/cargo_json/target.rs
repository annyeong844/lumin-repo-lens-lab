use serde::Deserialize;

use crate::protocol::CargoTargetKind;

#[derive(Debug, Clone, Deserialize)]
pub(super) struct CargoJsonTargetData {
    #[serde(default = "unknown_target_name")]
    name: String,
    #[serde(default)]
    #[serde(rename = "kind")]
    kinds: Vec<CargoTargetKind>,
}

#[derive(Clone, Copy)]
pub(crate) struct CargoJsonTarget<'a> {
    data: &'a CargoJsonTargetData,
}

impl<'a> CargoJsonTarget<'a> {
    pub(super) fn new(data: &'a CargoJsonTargetData) -> Self {
        Self { data }
    }

    pub(crate) fn name(self) -> &'a str {
        &self.data.name
    }

    pub(crate) fn kinds(self) -> Vec<CargoTargetKind> {
        self.data.kinds.clone()
    }
}

fn unknown_target_name() -> String {
    "<unknown>".to_string()
}
