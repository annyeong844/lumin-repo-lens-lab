use serde::Serialize;

use crate::policy::{
    ProductFileOracleBridgeProjection, ProductFileSemanticSummary, ProductSyntaxFileProjection,
    ProductSyntaxFileSummary,
};

use super::refs::SemanticRefCounts;

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::product_files) struct SemanticFindingRef {
    index: usize,
}

impl SemanticFindingRef {
    pub(in crate::product_files) fn from_index(index: usize) -> Self {
        Self { index }
    }

    pub(super) fn index(self) -> usize {
        self.index
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::product_files) struct SemanticDiagnosticRef {
    index: usize,
}

impl SemanticDiagnosticRef {
    pub(in crate::product_files) fn from_index(index: usize) -> Self {
        Self { index }
    }

    pub(super) fn index(self) -> usize {
        self.index
    }
}

pub(super) struct ProductFileDraft<'a> {
    syntax: Option<ProductSyntaxFileProjection<'a>>,
    syntax_summary: ProductSyntaxFileSummary,
    semantic: ProductFileSemanticLane,
    semantic_summary: ProductFileSemanticSummary,
}

impl<'a> ProductFileDraft<'a> {
    pub(super) fn empty() -> Self {
        Self {
            syntax: None,
            syntax_summary: ProductSyntaxFileSummary::missing(),
            semantic: ProductFileSemanticLane::default(),
            semantic_summary: ProductFileSemanticSummary::empty(),
        }
    }

    pub(super) fn set_syntax(
        &mut self,
        syntax: ProductSyntaxFileProjection<'a>,
        syntax_summary: ProductSyntaxFileSummary,
    ) {
        self.syntax = Some(syntax);
        self.syntax_summary = syntax_summary;
    }

    pub(super) fn push_finding(
        &mut self,
        semantic_ref: SemanticFindingRef,
        summary: ProductFileSemanticSummary,
    ) {
        self.semantic.findings.push(semantic_ref);
        self.semantic_summary.merge(summary);
    }

    pub(super) fn push_diagnostic(&mut self, semantic_ref: SemanticDiagnosticRef) {
        self.semantic.diagnostics.push(semantic_ref);
        self.semantic_summary.record_diagnostic();
    }

    pub(super) fn syntax_summary(&self) -> ProductSyntaxFileSummary {
        self.syntax_summary
    }

    pub(super) fn semantic_summary(&self) -> ProductFileSemanticSummary {
        self.semantic_summary
    }

    pub(super) fn into_entry(
        self,
        oracle_bridge: ProductFileOracleBridgeProjection,
    ) -> ProductFileEntry<'a> {
        ProductFileEntry {
            syntax: self.syntax,
            semantic: self.semantic,
            oracle_bridge,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProductFileEntry<'a> {
    syntax: Option<ProductSyntaxFileProjection<'a>>,
    #[serde(skip_serializing_if = "ProductFileSemanticLane::is_empty")]
    semantic: ProductFileSemanticLane,
    #[serde(skip_serializing_if = "ProductFileOracleBridgeProjection::is_empty")]
    oracle_bridge: ProductFileOracleBridgeProjection,
}

impl ProductFileEntry<'_> {
    pub(super) fn semantic_finding_ref_count(&self) -> usize {
        self.semantic.findings.len()
    }

    pub(super) fn semantic_diagnostic_ref_count(&self) -> usize {
        self.semantic.diagnostics.len()
    }

    pub(super) fn first_out_of_range_finding_ref(
        &self,
        expected_refs: SemanticRefCounts,
    ) -> Option<usize> {
        self.semantic
            .findings
            .iter()
            .map(|finding| finding.index())
            .find(|index| !expected_refs.contains_finding_ref(*index))
    }

    pub(super) fn first_out_of_range_diagnostic_ref(
        &self,
        expected_refs: SemanticRefCounts,
    ) -> Option<usize> {
        self.semantic
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.index())
            .find(|index| !expected_refs.contains_diagnostic_ref(*index))
    }
}

#[derive(Debug, Default, Serialize)]
struct ProductFileSemanticLane {
    findings: Vec<SemanticFindingRef>,
    diagnostics: Vec<SemanticDiagnosticRef>,
}

impl ProductFileSemanticLane {
    fn is_empty(&self) -> bool {
        self.findings.is_empty() && self.diagnostics.is_empty()
    }
}
