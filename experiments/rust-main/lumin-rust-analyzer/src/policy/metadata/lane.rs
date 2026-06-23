use serde::Serialize;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum RawLaneOwner {
    RustSourceHealth,
    RustCargoOracle,
}
