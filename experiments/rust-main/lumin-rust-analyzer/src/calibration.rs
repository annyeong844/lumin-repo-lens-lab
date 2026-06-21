use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::policy::ActionPolicyTier;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CalibrationAdjudication {
    #[serde(default)]
    entries: Vec<CalibrationAdjudicationEntry>,
}

impl CalibrationAdjudication {
    pub(crate) fn entries(&self) -> &[CalibrationAdjudicationEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationAdjudicationEntry {
    pub(crate) tier: Option<ActionPolicyTier>,
    #[serde(default)]
    pub(crate) verdict: CalibrationVerdict,
    pub(crate) file: Option<String>,
    pub(crate) diagnostic_code: Option<String>,
    pub(crate) line_start: Option<i64>,
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CalibrationVerdict {
    TrueDead,
    FalsePositive,
    #[default]
    Inconclusive,
    NotApplicable,
}

enum CalibrationAdjudicationInput {
    Entries(Vec<CalibrationAdjudicationEntry>),
    Object(CalibrationAdjudication),
}

pub(crate) fn load_adjudication(path: Option<&Path>) -> Result<Option<CalibrationAdjudication>> {
    let Some(path) = path else {
        return Ok(None);
    };
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read calibration adjudication {}", path.display()))?;
    let input = CalibrationAdjudicationInput::from_slice(&bytes).with_context(|| {
        format!(
            "failed to parse calibration adjudication {}",
            path.display()
        )
    })?;
    Ok(Some(input.into_adjudication()))
}

impl CalibrationAdjudicationInput {
    fn from_slice(bytes: &[u8]) -> serde_json::Result<Self> {
        if bytes
            .iter()
            .copied()
            .find(|byte| !byte.is_ascii_whitespace())
            == Some(b'[')
        {
            return serde_json::from_slice(bytes).map(Self::Entries);
        }
        serde_json::from_slice(bytes).map(Self::Object)
    }

    fn into_adjudication(self) -> CalibrationAdjudication {
        match self {
            Self::Entries(entries) => CalibrationAdjudication { entries },
            Self::Object(adjudication) => adjudication,
        }
    }
}
