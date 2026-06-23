use std::fmt;

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
