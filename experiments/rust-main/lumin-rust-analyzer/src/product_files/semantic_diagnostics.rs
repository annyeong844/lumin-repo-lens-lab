use std::path::Path;

use lumin_rust_cargo_oracle::protocol::{
    ClaimKind, ClassificationEvidence, ClassificationRule, CodeKind, CodeNamespace, CodePresence,
    ConfidenceTier, CoverageEffect, DiagnosticEvidence, Disposition, NormalizedDiagnostic,
    PrimarySpan, PrimarySpanClass, RustcDiagnosticLevel,
};
use serde::Serialize;

use crate::policy::ProductPrimarySpanProjection;

use super::model::{SemanticDiagnosticRef, SemanticRefCounts};
use super::path::diagnostic_relative_path;

pub(crate) fn semantic_diagnostics_with_paths<'a>(
    root: &Path,
    diagnostics: &'a [DiagnosticEvidence],
) -> ProductSemanticDiagnostics<'a> {
    let entries = diagnostics
        .iter()
        .enumerate()
        .map(|(index, diagnostic)| ProductSemanticDiagnosticEntry {
            semantic_ref: SemanticDiagnosticRef::from_index(index),
            path: diagnostic_relative_path(root, diagnostic),
            projection: ProductSemanticDiagnosticProjection::from_diagnostic(diagnostic),
        })
        .collect();
    ProductSemanticDiagnostics { entries }
}

pub(crate) struct ProductSemanticDiagnostics<'a> {
    entries: Vec<ProductSemanticDiagnosticEntry<'a>>,
}

impl<'a> ProductSemanticDiagnostics<'a> {
    pub(super) fn entries(&self) -> &[ProductSemanticDiagnosticEntry<'a>] {
        &self.entries
    }

    pub(crate) fn unlinked_refs(&self) -> SemanticRefCounts {
        SemanticRefCounts::new(
            0,
            self.entries
                .iter()
                .filter(|entry| entry.path.is_none())
                .count(),
        )
    }

    pub(crate) fn into_projection(self) -> ProductSemanticDiagnosticsProjection<'a> {
        ProductSemanticDiagnosticsProjection {
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
pub(crate) struct ProductSemanticDiagnosticsProjection<'a> {
    entries: Vec<ProductSemanticDiagnosticProjection<'a>>,
}

impl ProductSemanticDiagnosticsProjection<'_> {
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}

pub(super) struct ProductSemanticDiagnosticEntry<'a> {
    semantic_ref: SemanticDiagnosticRef,
    path: Option<String>,
    projection: ProductSemanticDiagnosticProjection<'a>,
}

impl ProductSemanticDiagnosticEntry<'_> {
    pub(super) fn semantic_ref(&self) -> SemanticDiagnosticRef {
        self.semantic_ref
    }

    pub(super) fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductSemanticDiagnosticProjection<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<&'a RustcDiagnosticLevel>,
    normalized: ProductNormalizedDiagnosticProjection<'a>,
    classification: ProductClassificationProjection,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_span: Option<ProductPrimarySpanProjection<'a>>,
    primary_span_count: usize,
}

impl<'a> ProductSemanticDiagnosticProjection<'a> {
    fn from_diagnostic(diagnostic: &'a DiagnosticEvidence) -> Self {
        Self {
            level: diagnostic.level.as_ref(),
            normalized: ProductNormalizedDiagnosticProjection::from_normalized(
                &diagnostic.normalized,
            ),
            classification: ProductClassificationProjection::from_classification(
                &diagnostic.classification,
            ),
            message: diagnostic.message.as_deref(),
            primary_span: PrimarySpan::representative(&diagnostic.primary_spans)
                .map(ProductPrimarySpanProjection::from_span),
            primary_span_count: diagnostic.primary_spans.len(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductNormalizedDiagnosticProjection<'a> {
    code_presence: CodePresence,
    #[serde(skip_serializing_if = "Option::is_none")]
    code_value: Option<&'a str>,
    code_namespace: CodeNamespace,
    code_kind: CodeKind,
    primary_span: PrimarySpanClass,
}

impl<'a> ProductNormalizedDiagnosticProjection<'a> {
    fn from_normalized(normalized: &'a NormalizedDiagnostic) -> Self {
        Self {
            code_presence: normalized.code_presence,
            code_value: normalized.code_value.as_deref(),
            code_namespace: normalized.code_namespace,
            code_kind: normalized.code_kind,
            primary_span: normalized.primary_span,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductClassificationProjection {
    disposition: Disposition,
    #[serde(skip_serializing_if = "Option::is_none")]
    confidence: Option<ConfidenceTier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    claim_kind: Option<ClaimKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    coverage_effect: Option<CoverageEffect>,
    rule: ClassificationRule,
}

impl ProductClassificationProjection {
    fn from_classification(classification: &ClassificationEvidence) -> Self {
        Self {
            disposition: classification.disposition,
            confidence: classification.confidence,
            claim_kind: classification.claim_kind,
            coverage_effect: classification.coverage_effect,
            rule: classification.rule,
        }
    }
}
