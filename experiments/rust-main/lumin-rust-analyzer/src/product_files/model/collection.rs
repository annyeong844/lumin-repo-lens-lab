use std::collections::BTreeMap;
use std::fmt;

use lumin_rust_source_health::protocol::FileHealth;
use serde::Serialize;

use super::entry::{ProductFileDraft, ProductFileEntry};
use super::{SemanticDiagnosticRef, SemanticFindingRef};
use crate::policy::{
    product_file_oracle_bridge, product_syntax_file, CoverageEvidence, OracleBridge,
    ProductFileSemanticSummary,
};

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub(crate) struct ProductFilesProjection<'a> {
    files: BTreeMap<String, ProductFileEntry<'a>>,
}

impl<'a> ProductFilesProjection<'a> {
    fn new(files: BTreeMap<String, ProductFileEntry<'a>>) -> Self {
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum SemanticRefContractError<'a> {
    FindingRefOutOfRange {
        path: &'a str,
        index: usize,
        finding_count: usize,
    },
    DiagnosticRefOutOfRange {
        path: &'a str,
        index: usize,
        diagnostic_count: usize,
    },
    FindingRefCountMismatch {
        actual_total: usize,
        expected_total: usize,
    },
    DiagnosticRefCountMismatch {
        actual_total: usize,
        expected_total: usize,
    },
}

impl fmt::Display for SemanticRefContractError<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FindingRefOutOfRange {
                path,
                index,
                finding_count,
            } => write!(
                formatter,
                "files[{path}].semantic.findings references semanticFindings[{index}], but semanticFindings.length={finding_count}"
            ),
            Self::DiagnosticRefOutOfRange {
                path,
                index,
                diagnostic_count,
            } => write!(
                formatter,
                "files[{path}].semantic.diagnostics references semanticDiagnostics[{index}], but semanticDiagnostics.length={diagnostic_count}"
            ),
            Self::FindingRefCountMismatch {
                actual_total,
                expected_total,
            } => write!(
                formatter,
                "files.semantic.findings.length + summary.semanticUnlinkedFindings must match semanticFindings.length: left={actual_total} right={expected_total}"
            ),
            Self::DiagnosticRefCountMismatch {
                actual_total,
                expected_total,
            } => write!(
                formatter,
                "files.semantic.diagnostics.length + summary.semanticUnlinkedDiagnostics must match semanticDiagnostics.length: left={actual_total} right={expected_total}"
            ),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub(crate) struct SemanticRefCounts {
    findings: usize,
    diagnostics: usize,
}

impl SemanticRefCounts {
    pub(crate) fn new(findings: usize, diagnostics: usize) -> Self {
        Self {
            findings,
            diagnostics,
        }
    }

    pub(in crate::product_files) fn plus(self, next: Self) -> Self {
        Self {
            findings: self.findings + next.findings,
            diagnostics: self.diagnostics + next.diagnostics,
        }
    }

    pub(crate) fn findings(self) -> usize {
        self.findings
    }

    pub(crate) fn diagnostics(self) -> usize {
        self.diagnostics
    }

    pub(in crate::product_files) fn contains_finding_ref(self, index: usize) -> bool {
        index < self.findings
    }

    pub(in crate::product_files) fn contains_diagnostic_ref(self, index: usize) -> bool {
        index < self.diagnostics
    }
}

pub(in crate::product_files) struct ProductFiles<'a> {
    files: BTreeMap<String, ProductFileDraft<'a>>,
}

impl<'a> ProductFiles<'a> {
    pub(in crate::product_files) fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    pub(in crate::product_files) fn insert_syntax(&mut self, path: &str, file: &'a FileHealth) {
        let syntax_file = product_syntax_file(file);
        let summary = syntax_file.summary();
        let projection = syntax_file.into_projection();
        self.files
            .entry(path.to_string())
            .or_insert_with(ProductFileDraft::empty)
            .set_syntax(projection, summary);
    }

    pub(in crate::product_files) fn push_semantic_finding(
        &mut self,
        path: &str,
        summary: ProductFileSemanticSummary,
        semantic_ref: SemanticFindingRef,
    ) {
        self.files
            .entry(path.to_string())
            .or_insert_with(ProductFileDraft::empty)
            .push_finding(semantic_ref, summary);
    }

    pub(in crate::product_files) fn push_semantic_diagnostic(
        &mut self,
        path: &str,
        semantic_ref: SemanticDiagnosticRef,
    ) {
        self.files
            .entry(path.to_string())
            .or_insert_with(ProductFileDraft::empty)
            .push_diagnostic(semantic_ref);
    }

    pub(in crate::product_files) fn with_oracle_bridges(
        self,
        oracle_bridge: &OracleBridge<'_>,
        coverage: &CoverageEvidence<'_>,
    ) -> ProductFilesProjection<'a> {
        let mut files = BTreeMap::new();
        for (path, file) in self.files {
            let semantic_summary = file.semantic_summary();
            let syntax_summary = file.syntax_summary();
            let file_bridge = product_file_oracle_bridge(
                syntax_summary,
                semantic_summary,
                oracle_bridge,
                coverage,
            );
            files.insert(path, file.into_entry(file_bridge));
        }
        ProductFilesProjection::new(files)
    }
}
