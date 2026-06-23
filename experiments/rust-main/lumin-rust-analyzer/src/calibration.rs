use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

mod adjudication;
mod candidate_counts;
mod corpus;
mod input;
mod schema;
mod verdict;

pub(crate) use adjudication::{CalibrationAdjudication, CalibrationAdjudicationEntry};
pub(crate) use candidate_counts::CalibrationCandidateCounts;
pub(in crate::calibration) use candidate_counts::CalibrationCorpusCandidateCounts;
pub(crate) use corpus::CalibrationCorpusEntry;
pub(in crate::calibration) use schema::CalibrationSchemaDriftBug;
pub(crate) use schema::CalibrationSchemaRoundTrip;
pub(crate) use verdict::CalibrationVerdict;

pub(crate) fn load_adjudication(path: Option<&Path>) -> Result<Option<CalibrationAdjudication>> {
    let Some(path) = path else {
        return Ok(None);
    };
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read calibration adjudication {}", path.display()))?;
    input::parse_adjudication(&bytes)
        .with_context(|| {
            format!(
                "failed to parse calibration adjudication {}",
                path.display()
            )
        })
        .map(Some)
}
