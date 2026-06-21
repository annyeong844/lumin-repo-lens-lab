use crate::policy::{CoverageRunStatus, FileParseStatus, OracleBridgeStatus, OracleConfidence};

pub(super) fn file_oracle_confidence(
    parse_status: FileParseStatus,
    review_opaque_surfaces: usize,
    global_status: OracleBridgeStatus,
    cargo_event_status: CoverageRunStatus,
    absence_status: CoverageRunStatus,
) -> OracleConfidence {
    if parse_status == FileParseStatus::Error
        || !cargo_event_status.is_ran()
        || matches!(
            global_status,
            OracleBridgeStatus::Unavailable | OracleBridgeStatus::Missing
        )
    {
        return OracleConfidence::Low;
    }
    if parse_status == FileParseStatus::Missing
        || review_opaque_surfaces > 0
        || !absence_status.is_ran()
        || global_status == OracleBridgeStatus::Partial
    {
        return OracleConfidence::Medium;
    }
    OracleConfidence::High
}
