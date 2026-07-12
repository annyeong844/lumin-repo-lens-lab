use std::path::Path;

use lumin_rust_cargo_oracle::protocol::Finding;
use serde::Serialize;

use crate::policy::{
    product_semantic_finding, CoverageEvidence, ProductFileSemanticSummary,
    ProductSemanticFindingProjection,
};
use crate::syntax_phase::SyntaxPhase;

use super::model::{SemanticFindingRef, SemanticRefCounts};
use super::path::typed_finding_relative_path;

pub(crate) fn semantic_findings_with_oracle_provenance<'a>(
    root: &Path,
    syntax_phase: SyntaxPhase<'a>,
    findings: &'a [Finding],
    coverage: &CoverageEvidence<'_>,
) -> ProductSemanticFindings<'a> {
    let entries = findings
        .iter()
        .enumerate()
        .map(|(index, finding)| {
            let path = typed_finding_relative_path(root, finding);
            let syntax_file = path.as_deref().and_then(|path| syntax_phase.file(path));
            let product_finding = product_semantic_finding(finding, syntax_file, coverage);
            ProductSemanticFindingEntry {
                semantic_ref: SemanticFindingRef::from_index(index),
                path,
                summary: product_finding.summary,
                projection: product_finding.projection,
            }
        })
        .collect::<Vec<_>>();
    ProductSemanticFindings { entries }
}

pub(crate) struct ProductSemanticFindings<'a> {
    entries: Vec<ProductSemanticFindingEntry<'a>>,
}

impl<'a> ProductSemanticFindings<'a> {
    pub(super) fn entries(&self) -> &[ProductSemanticFindingEntry<'a>] {
        &self.entries
    }

    pub(super) fn unlinked_refs(&self) -> SemanticRefCounts {
        SemanticRefCounts::new(
            self.entries
                .iter()
                .filter(|entry| entry.path.is_none())
                .count(),
            0,
        )
    }

    pub(crate) fn into_projection(self) -> ProductSemanticFindingsProjection<'a> {
        ProductSemanticFindingsProjection {
            entries: self
                .entries
                .into_iter()
                .map(|entry| entry.projection)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub(crate) struct ProductSemanticFindingsProjection<'a> {
    entries: Vec<ProductSemanticFindingProjection<'a>>,
}

impl ProductSemanticFindingsProjection<'_> {
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}

pub(super) struct ProductSemanticFindingEntry<'a> {
    semantic_ref: SemanticFindingRef,
    path: Option<String>,
    summary: ProductFileSemanticSummary,
    projection: ProductSemanticFindingProjection<'a>,
}

impl ProductSemanticFindingEntry<'_> {
    pub(super) fn semantic_ref(&self) -> SemanticFindingRef {
        self.semantic_ref
    }

    pub(super) fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub(super) fn summary(&self) -> ProductFileSemanticSummary {
        self.summary
    }
}
