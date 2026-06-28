use lumin_rust_cargo_oracle::protocol::{
    CleanKind, CoverageEntry, CoverageId, CoverageKind, CoverageStatus, OracleId,
    ABSENCE_CLEAN_COVERAGE_ID, EVENT_STREAM_COVERAGE_ID,
};
use serde::Serialize;

use super::{SupportEvidence, TaintEffect, TaintEvidence};
use crate::policy::{CoverageRunStatus, OracleBridgeStatus, ProductCoverageUnavailableReason};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum CoverageBridgeMissingReason {
    CoverageEntryMissing,
}

pub(crate) struct CoverageEvidence<'coverage> {
    cargo_event_stream: CoverageLaneEvidence<'coverage>,
    absence_clean: CoverageLaneEvidence<'coverage>,
    unavailable_entries: usize,
}

impl<'coverage> CoverageEvidence<'coverage> {
    pub(crate) fn from_coverage_entries(coverage: &'coverage [CoverageEntry]) -> Self {
        Self {
            cargo_event_stream: CoverageLaneEvidence::find(coverage, EVENT_STREAM_COVERAGE_ID),
            absence_clean: CoverageLaneEvidence::find(coverage, ABSENCE_CLEAN_COVERAGE_ID),
            unavailable_entries: coverage
                .iter()
                .filter(|entry| entry.status == CoverageStatus::Unavailable)
                .count(),
        }
    }

    pub(in crate::policy) fn cargo_event_status(&self) -> CoverageRunStatus {
        self.cargo_event_stream.status()
    }

    pub(in crate::policy) fn absence_status(&self) -> CoverageRunStatus {
        self.absence_clean.status()
    }

    pub(in crate::policy) fn oracle_bridge_status(&self) -> OracleBridgeStatus {
        match (self.cargo_event_status(), self.absence_status()) {
            (CoverageRunStatus::Ran, CoverageRunStatus::Ran) => OracleBridgeStatus::Covered,
            (CoverageRunStatus::Ran, CoverageRunStatus::Unavailable) => OracleBridgeStatus::Partial,
            (CoverageRunStatus::Unavailable, _) | (_, CoverageRunStatus::Unavailable) => {
                OracleBridgeStatus::Unavailable
            }
            _ => OracleBridgeStatus::Missing,
        }
    }

    pub(in crate::policy) fn push_supported_by(&self, supported_by: &mut Vec<SupportEvidence>) {
        let cargo_event_status = self.cargo_event_status();
        if cargo_event_status.is_ran() {
            supported_by.push(SupportEvidence::cargo_check_event_stream(
                cargo_event_status,
            ));
        }
        let absence_status = self.absence_status();
        if absence_status.is_ran() {
            supported_by.push(SupportEvidence::cargo_check_absence_clean(
                absence_status,
                self.absence_clean.clean(),
                self.absence_clean.clean_kind(),
            ));
        }
    }

    pub(in crate::policy) fn push_tainted_by<'a>(
        &self,
        tainted_by: &mut Vec<TaintEvidence<'a>>,
        cargo_event_effect: TaintEffect,
        absence_effect: TaintEffect,
    ) {
        let cargo_event_status = self.cargo_event_status();
        if !cargo_event_status.is_ran() {
            tainted_by.push(TaintEvidence::cargo_event_stream_not_run(
                cargo_event_status,
                cargo_event_effect,
            ));
        }
        let absence_status = self.absence_status();
        if !absence_status.is_ran() {
            tainted_by.push(TaintEvidence::cargo_absence_clean_unavailable(
                absence_status,
                absence_effect,
            ));
        }
    }

    pub(in crate::policy) fn bridge_entries(&self) -> (CoverageBridgeEntry, CoverageBridgeEntry) {
        (
            self.cargo_event_stream.bridge_entry(),
            self.absence_clean.bridge_entry(),
        )
    }

    pub(in crate::policy) fn unavailable_entries(&self) -> usize {
        self.unavailable_entries
    }
}

#[derive(Debug, Copy, Clone)]
struct CoverageLaneEvidence<'coverage> {
    entry: Option<&'coverage CoverageEntry>,
}

impl<'coverage> CoverageLaneEvidence<'coverage> {
    fn find(coverage: &'coverage [CoverageEntry], id: CoverageId) -> Self {
        Self {
            entry: coverage.iter().find(|entry| entry.id == id),
        }
    }

    fn status(self) -> CoverageRunStatus {
        CoverageRunStatus::from_coverage_status(self.entry.map(|entry| entry.status))
    }

    fn clean(self) -> Option<bool> {
        self.entry.and_then(|entry| entry.clean)
    }

    fn clean_kind(self) -> Option<CleanKind> {
        self.entry.and_then(|entry| entry.clean_kind)
    }

    fn bridge_entry(self) -> CoverageBridgeEntry {
        CoverageBridgeEntry::from_coverage_entry(self.entry)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub(in crate::policy) enum CoverageBridgeEntry {
    Present(CoverageBridgePresentEntry),
    Missing(CoverageBridgeMissingEntry),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct CoverageBridgePresentEntry {
    id: CoverageId,
    oracle_id: OracleId,
    coverage_kind: CoverageKind,
    status: CoverageRunStatus,
    reason: Option<ProductCoverageUnavailableReason>,
    clean: Option<bool>,
    clean_kind: Option<CleanKind>,
    exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct CoverageBridgeMissingEntry {
    status: CoverageRunStatus,
    reason: CoverageBridgeMissingReason,
}

impl CoverageBridgeEntry {
    fn from_coverage_entry(entry: Option<&CoverageEntry>) -> Self {
        let Some(entry) = entry else {
            return Self::Missing(CoverageBridgeMissingEntry {
                status: CoverageRunStatus::Missing,
                reason: CoverageBridgeMissingReason::CoverageEntryMissing,
            });
        };
        Self::Present(CoverageBridgePresentEntry {
            id: entry.id,
            oracle_id: entry.oracle_id,
            coverage_kind: entry.coverage_kind,
            status: CoverageRunStatus::from_coverage_status(Some(entry.status)),
            reason: entry
                .reason
                .as_ref()
                .map(ProductCoverageUnavailableReason::from_reason),
            clean: entry.clean,
            clean_kind: entry.clean_kind,
            exit_code: entry.exit_code,
        })
    }
}
