use serde::Serialize;

use crate::policy::{evidence::CoverageEvidence, CoverageRunStatus, FileParseStatus};

use super::FINDING_ORACLE_BRIDGE_SCHEMA_VERSION;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FindingOracleBridgeProjection {
    schema_version: &'static str,
    file_parse_status: FileParseStatus,
    coverage: FindingOracleBridgeCoverageProjection,
    local_review_opaque_surfaces: usize,
    does_not_override_safe_action: bool,
}

impl FindingOracleBridgeProjection {
    pub(super) fn new(
        file_parse_status: FileParseStatus,
        coverage: &CoverageEvidence<'_>,
        local_review_opaque_surfaces: usize,
    ) -> Self {
        Self {
            schema_version: FINDING_ORACLE_BRIDGE_SCHEMA_VERSION,
            file_parse_status,
            coverage: FindingOracleBridgeCoverageProjection {
                cargo_event_stream: FindingCoverageStatusProjection {
                    status: coverage.cargo_event_status(),
                },
                absence_clean: FindingCoverageStatusProjection {
                    status: coverage.absence_status(),
                },
            },
            local_review_opaque_surfaces,
            does_not_override_safe_action: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FindingOracleBridgeCoverageProjection {
    cargo_event_stream: FindingCoverageStatusProjection,
    absence_clean: FindingCoverageStatusProjection,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FindingCoverageStatusProjection {
    status: CoverageRunStatus,
}
