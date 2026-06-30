use std::collections::BTreeMap;

use super::entry::ProductFileDraft;
use super::projection::ProductFilesProjection;
use super::{SemanticDiagnosticRef, SemanticFindingRef};
use crate::policy::{
    product_compact_syntax_file, product_file_oracle_bridge, product_syntax_file, CoverageEvidence,
    OracleBridge, ProductFileSemanticSummary,
};
use crate::syntax_phase::SyntaxFile;

pub(in crate::product_files) struct ProductFiles<'a> {
    files: BTreeMap<String, ProductFileDraft<'a>>,
}

impl<'a> ProductFiles<'a> {
    pub(in crate::product_files) fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    pub(in crate::product_files) fn insert_syntax(&mut self, path: &str, file: SyntaxFile<'a>) {
        let syntax_file = match file {
            SyntaxFile::Full(file) => product_syntax_file(file),
            SyntaxFile::Compact(file) => product_compact_syntax_file(file),
        };
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
        ProductFilesProjection::from_files(files)
    }
}
