use lumin_rust_source_health::protocol::{
    AstFacts, AstOpaqueReason, AstOpaqueSurface, AstOpaqueSurfaceKind, AstOpaqueSurfaceVisibility,
    CompactAstSummary,
};
use serde::Serialize;

use crate::policy::{AST_SAMPLE_LIMIT, FILE_AST_SAMPLE_LIMIT};
use crate::syntax_phase::{SyntaxFile, SyntaxPhase};

use super::super::ProductLocation;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy::syntax) struct AstExampleSamples<'a> {
    review_opaque_surfaces: Vec<ReviewOpaqueSurfaceExample<'a>>,
}

pub(in crate::policy::syntax) fn ast_examples_for_product<'a>(
    ast: &'a AstFacts,
    include_examples: bool,
) -> Option<AstExampleSamples<'a>> {
    if include_examples {
        return Some(ast_examples(ast));
    }

    None
}

pub(in crate::policy::syntax) fn ast_examples_for_compact_product<'a>(
    ast_summary: &'a CompactAstSummary,
    include_examples: bool,
) -> Option<AstExampleSamples<'a>> {
    include_examples.then(|| AstExampleSamples {
        review_opaque_surfaces: ast_summary
            .review_opaque_surface_examples
            .iter()
            .map(ReviewOpaqueSurfaceExample::from_surface)
            .collect(),
    })
}

fn ast_examples(ast: &AstFacts) -> AstExampleSamples<'_> {
    AstExampleSamples {
        review_opaque_surfaces: sample_opaque_surfaces_by_visibility(
            ast,
            AstOpaqueSurfaceVisibility::Review,
            FILE_AST_SAMPLE_LIMIT,
        ),
    }
}

fn sample_opaque_surfaces_by_visibility(
    ast: &AstFacts,
    visibility: AstOpaqueSurfaceVisibility,
    limit: usize,
) -> Vec<ReviewOpaqueSurfaceExample<'_>> {
    ast.opaque_surfaces
        .iter()
        .filter(|surface| surface.visibility == visibility)
        .take(limit)
        .map(ReviewOpaqueSurfaceExample::from_surface)
        .collect()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewOpaqueSurfaceExample<'a> {
    kind: AstOpaqueSurfaceKind,
    reason: AstOpaqueReason,
    detail: &'a str,
    location: ProductLocation,
}

impl<'a> ReviewOpaqueSurfaceExample<'a> {
    fn from_surface(surface: &'a AstOpaqueSurface) -> Self {
        Self {
            kind: surface.kind,
            reason: surface.reason,
            detail: &surface.detail,
            location: ProductLocation::from(&surface.location),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyntaxReviewOpaqueSurfaceExample<'a> {
    file: &'a str,
    kind: AstOpaqueSurfaceKind,
    reason: AstOpaqueReason,
    detail: &'a str,
    location: ProductLocation,
}

pub(crate) fn syntax_review_opaque_surface_examples(
    syntax: SyntaxPhase<'_>,
) -> Vec<SyntaxReviewOpaqueSurfaceExample<'_>> {
    let mut examples = Vec::new();
    for (path, file) in syntax.files() {
        match file {
            SyntaxFile::Full(file) => {
                examples.extend(
                    file.ast
                        .opaque_surfaces
                        .iter()
                        .filter(|surface| surface.visibility == AstOpaqueSurfaceVisibility::Review)
                        .map(|surface| SyntaxReviewOpaqueSurfaceExample {
                            file: path,
                            kind: surface.kind,
                            reason: surface.reason,
                            detail: &surface.detail,
                            location: ProductLocation::from(&surface.location),
                        }),
                );
            }
            SyntaxFile::Compact(file) => {
                examples.extend(file.ast_summary.review_opaque_surface_examples.iter().map(
                    |surface| SyntaxReviewOpaqueSurfaceExample {
                        file: path,
                        kind: surface.kind,
                        reason: surface.reason,
                        detail: &surface.detail,
                        location: ProductLocation::from(&surface.location),
                    },
                ));
            }
        }
        if examples.len() >= AST_SAMPLE_LIMIT {
            examples.truncate(AST_SAMPLE_LIMIT);
            break;
        }
    }
    examples
}
