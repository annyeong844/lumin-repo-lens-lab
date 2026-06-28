use serde::Deserialize;

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CalibrationVerdict {
    TrueDead,
    FalsePositive,
    #[default]
    Inconclusive,
    NotApplicable,
}
