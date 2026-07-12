use crate::policy::{CoverageEvidence, OracleBridge};
use crate::syntax_phase::SyntaxPhase;

use super::model::{ProductFiles, ProductFilesProjection, SemanticRefCounts};
use super::semantic_diagnostics::ProductSemanticDiagnostics;
use super::semantic_findings::ProductSemanticFindings;

pub(crate) fn merged_files<'a>(
    syntax_phase: SyntaxPhase<'a>,
    semantic_diagnostics: &ProductSemanticDiagnostics,
    semantic_findings: &ProductSemanticFindings,
    oracle_bridge: &OracleBridge<'_>,
    coverage: &CoverageEvidence<'_>,
) -> MergedFiles<'a> {
    let mut files = ProductFiles::new();
    let unlinked_semantic_refs = semantic_findings
        .unlinked_refs()
        .plus(semantic_diagnostics.unlinked_refs());

    for (path, file) in syntax_phase.files() {
        files.insert_syntax(path, file);
    }

    for finding in semantic_findings.entries() {
        if let Some(path) = finding.path() {
            files.push_semantic_finding(path, finding.summary(), finding.semantic_ref());
        }
    }

    for diagnostic in semantic_diagnostics.entries() {
        if let Some(path) = diagnostic.path() {
            files.push_semantic_diagnostic(path, diagnostic.semantic_ref());
        }
    }

    MergedFiles {
        files: files.with_oracle_bridges(oracle_bridge, coverage),
        unlinked_semantic_refs,
    }
}

pub(crate) struct MergedFiles<'a> {
    files: ProductFilesProjection<'a>,
    unlinked_semantic_refs: SemanticRefCounts,
}

impl<'a> MergedFiles<'a> {
    pub(crate) fn unlinked_semantic_refs(&self) -> SemanticRefCounts {
        self.unlinked_semantic_refs
    }

    pub(crate) fn into_projection(self) -> ProductFilesProjection<'a> {
        self.files
    }
}
