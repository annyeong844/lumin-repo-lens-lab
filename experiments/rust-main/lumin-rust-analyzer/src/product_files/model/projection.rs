use std::collections::BTreeMap;

use serde::Serialize;

use super::entry::ProductFileEntry;
use super::refs::{SemanticRefContractError, SemanticRefCounts};

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub(crate) struct ProductFilesProjection<'a> {
    files: BTreeMap<String, ProductFileEntry<'a>>,
}

impl<'a> ProductFilesProjection<'a> {
    pub(super) fn from_files(files: BTreeMap<String, ProductFileEntry<'a>>) -> Self {
        Self { files }
    }

    pub(crate) fn len(&self) -> usize {
        self.files.len()
    }

    fn semantic_ref_counts(&self) -> SemanticRefCounts {
        self.files
            .values()
            .map(|file| {
                SemanticRefCounts::new(
                    file.semantic_finding_ref_count(),
                    file.semantic_diagnostic_ref_count(),
                )
            })
            .fold(SemanticRefCounts::default(), SemanticRefCounts::plus)
    }

    pub(crate) fn first_semantic_ref_contract_error(
        &self,
        expected_refs: SemanticRefCounts,
        unlinked_refs: SemanticRefCounts,
    ) -> Option<SemanticRefContractError<'_>> {
        for (path, file) in &self.files {
            if let Some(index) = file.first_out_of_range_finding_ref(expected_refs) {
                return Some(SemanticRefContractError::FindingRefOutOfRange {
                    path,
                    index,
                    finding_count: expected_refs.findings(),
                });
            }
            if let Some(index) = file.first_out_of_range_diagnostic_ref(expected_refs) {
                return Some(SemanticRefContractError::DiagnosticRefOutOfRange {
                    path,
                    index,
                    diagnostic_count: expected_refs.diagnostics(),
                });
            }
        }
        let linked_refs = self.semantic_ref_counts();
        let total_refs = linked_refs.plus(unlinked_refs);
        if total_refs.findings() != expected_refs.findings() {
            return Some(SemanticRefContractError::FindingRefCountMismatch {
                actual_total: total_refs.findings(),
                expected_total: expected_refs.findings(),
            });
        }
        if total_refs.diagnostics() != expected_refs.diagnostics() {
            return Some(SemanticRefContractError::DiagnosticRefCountMismatch {
                actual_total: total_refs.diagnostics(),
                expected_total: expected_refs.diagnostics(),
            });
        }
        None
    }
}
