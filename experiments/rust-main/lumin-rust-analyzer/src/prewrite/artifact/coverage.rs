use anyhow::{bail, Result};
use serde::Serialize;

use crate::prewrite::intent::NormalizedIntent;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum LaneStatus {
    Ran,
    NotRequested,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct IntentLaneCoverage {
    names: LaneStatus,
    shapes: LaneStatus,
    files: LaneStatus,
    dependencies: LaneStatus,
    inline_patterns: LaneStatus,
    planned_type_escapes: LaneStatus,
}

impl IntentLaneCoverage {
    pub(super) fn from_intent(intent: &NormalizedIntent) -> Self {
        Self {
            names: LaneStatus::Ran,
            shapes: ran_if_requested(!intent.shapes.is_empty()),
            files: ran_if_requested(!intent.files.is_empty()),
            dependencies: ran_if_requested(!intent.dependencies.is_empty()),
            inline_patterns: ran_if_requested(intent.has_refactor_sources()),
            planned_type_escapes: LaneStatus::Ran,
        }
    }

    pub(super) fn validate(&self, intent: &NormalizedIntent) -> Result<()> {
        if self.names != LaneStatus::Ran
            || self.shapes != ran_if_requested(!intent.shapes.is_empty())
            || self.files != ran_if_requested(!intent.files.is_empty())
            || self.dependencies != ran_if_requested(!intent.dependencies.is_empty())
            || self.inline_patterns != ran_if_requested(intent.has_refactor_sources())
            || self.planned_type_escapes != LaneStatus::Ran
        {
            bail!("blocked-artifact-contract: intent lane coverage drifted from normalized input");
        }
        Ok(())
    }
}

fn ran_if_requested(requested: bool) -> LaneStatus {
    if requested {
        LaneStatus::Ran
    } else {
        LaneStatus::NotRequested
    }
}
