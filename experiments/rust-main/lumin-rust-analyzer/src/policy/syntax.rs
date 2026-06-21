mod ast;
mod signals;
mod summary;

use lumin_rust_source_health::protocol::{
    Facts, FileHealth, Location, ParseError, ParseStatus, PathClassification, PathMeta,
};
use serde::Serialize;

pub(crate) use ast::{syntax_review_opaque_surface_examples, SyntaxReviewOpaqueSurfaceExample};
pub(crate) use signals::{syntax_review_signal_examples, SyntaxReviewSignalExample};
pub(crate) use summary::ProductSyntaxFileSummary;

use super::{FILE_SIGNAL_SAMPLE_LIMIT, PARSE_ERROR_SAMPLE_LIMIT};
use ast::{ast_examples_for_product, ast_summary, AstExampleSamples, AstSummary};
use signals::{signal_summary, signals_for_product, ProductSignalExample, SignalSummary};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy::syntax) struct ProductLocation {
    line: usize,
    column: usize,
}

impl From<&Location> for ProductLocation {
    fn from(location: &Location) -> Self {
        Self {
            line: location.line,
            column: location.column,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductSyntaxFileProjection<'a> {
    sha256: &'a str,
    #[serde(skip_serializing_if = "ProductFactsProjection::is_empty")]
    facts: ProductFactsProjection,
    parse: ProductParseProjection<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<ProductPathProjection>,
    #[serde(skip_serializing_if = "SignalSummary::is_empty")]
    signal_summary: SignalSummary,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    review_signals: Vec<ProductSignalExample>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    muted_signals: Vec<ProductSignalExample>,
    #[serde(skip_serializing_if = "AstSummary::is_empty")]
    ast_summary: AstSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    ast_examples: Option<AstExampleSamples<'a>>,
}

pub(crate) fn product_syntax_file(file: &FileHealth) -> ProductSyntaxFile<'_> {
    let (review_signals, muted_signals) = signals::partition_by_visibility(&file.signals);
    let ast_summary = ast_summary(&file.ast);
    let summary = ProductSyntaxFileSummary::from_file_with_opaque_surface_counts(
        file,
        ast_summary.opaque_surface_counts(),
    );
    let include_ast_examples = summary.review_opaque_surfaces() > 0;

    ProductSyntaxFile {
        projection: ProductSyntaxFileProjection {
            sha256: &file.sha256,
            facts: ProductFactsProjection::from_facts(&file.facts),
            parse: ProductParseProjection::from_parse(&file.parse),
            path: ProductPathProjection::from_path(&file.path),
            signal_summary: signal_summary(&review_signals, &muted_signals),
            review_signals: signals_for_product(&review_signals, FILE_SIGNAL_SAMPLE_LIMIT),
            muted_signals: signals_for_product(&muted_signals, FILE_SIGNAL_SAMPLE_LIMIT),
            ast_summary,
            ast_examples: ast_examples_for_product(&file.ast, include_ast_examples),
        },
        summary,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductFactsProjection {
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    functions: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_function_lines: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unsafe_blocks: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unsafe_functions: Option<usize>,
}

impl ProductFactsProjection {
    fn from_facts(facts: &Facts) -> Self {
        Self {
            items: (facts.items > 0).then_some(facts.items),
            functions: (facts.functions > 0).then_some(facts.functions),
            max_function_lines: (facts.max_function_lines > 0).then_some(facts.max_function_lines),
            unsafe_blocks: (facts.unsafe_blocks > 0).then_some(facts.unsafe_blocks),
            unsafe_functions: (facts.unsafe_functions > 0).then_some(facts.unsafe_functions),
        }
    }

    fn is_empty(&self) -> bool {
        self.items.is_none()
            && self.functions.is_none()
            && self.max_function_lines.is_none()
            && self.unsafe_blocks.is_none()
            && self.unsafe_functions.is_none()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductPathProjection {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    classifications: Vec<PathClassification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suppressed: Option<bool>,
}

impl ProductPathProjection {
    fn from_path(path: &PathMeta) -> Option<Self> {
        let classifications = path
            .classifications
            .iter()
            .copied()
            .filter(|classification| *classification != PathClassification::Source)
            .collect::<Vec<_>>();
        if classifications.is_empty() && !path.suppressed {
            return None;
        }
        Some(Self {
            classifications,
            suppressed: path.suppressed.then_some(true),
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductParseProjection<'a> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sample_limit: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    error_examples: Vec<&'a ParseError>,
}

impl<'a> ProductParseProjection<'a> {
    fn from_parse(parse: &'a ParseStatus) -> Self {
        let has_errors = !parse.errors.is_empty();
        Self {
            ok: parse.ok,
            error_count: has_errors.then_some(parse.errors.len()),
            sample_limit: if has_errors {
                Some(PARSE_ERROR_SAMPLE_LIMIT)
            } else {
                None
            },
            error_examples: parse.errors.iter().take(PARSE_ERROR_SAMPLE_LIMIT).collect(),
        }
    }
}

pub(crate) struct ProductSyntaxFile<'a> {
    projection: ProductSyntaxFileProjection<'a>,
    summary: ProductSyntaxFileSummary,
}

impl<'a> ProductSyntaxFile<'a> {
    pub(crate) fn summary(&self) -> ProductSyntaxFileSummary {
        self.summary
    }

    pub(crate) fn into_projection(self) -> ProductSyntaxFileProjection<'a> {
        self.projection
    }
}
