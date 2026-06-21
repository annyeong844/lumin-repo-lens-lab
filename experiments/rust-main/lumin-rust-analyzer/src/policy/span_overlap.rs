use lumin_rust_cargo_oracle::protocol::PrimarySpan;
use lumin_rust_source_health::protocol::{
    AstFacts, AstOpaqueSurface, AstOpaqueSurfaceVisibility, Location,
};

pub(super) fn review_opaque_surfaces_touching_span<'a>(
    ast: Option<&'a AstFacts>,
    span: Option<&PrimarySpan>,
    primary_spans: &[PrimarySpan],
) -> Vec<&'a AstOpaqueSurface> {
    let Some(finding_range) = finding_source_range(span, primary_spans) else {
        return Vec::new();
    };
    let Some(ast) = ast else {
        return Vec::new();
    };
    ast.opaque_surfaces
        .iter()
        .filter(|surface| {
            surface.visibility == AstOpaqueSurfaceVisibility::Review
                && opaque_surface_source_range(surface).is_some_and(|surface_range| {
                    source_ranges_overlap(finding_range, surface_range)
                })
        })
        .collect()
}

#[derive(Clone, Copy)]
struct SourceRange {
    line_start: i64,
    line_end: i64,
    column_start: i64,
    column_end: i64,
}

fn finding_source_range(
    span: Option<&PrimarySpan>,
    primary_spans: &[PrimarySpan],
) -> Option<SourceRange> {
    span.or_else(|| PrimarySpan::representative(primary_spans))
        .and_then(primary_span_source_range)
}

fn primary_span_source_range(span: &PrimarySpan) -> Option<SourceRange> {
    Some(SourceRange {
        line_start: span.line_start?,
        line_end: span.line_end?,
        column_start: span.column_start?,
        column_end: span.column_end?,
    })
}

fn opaque_surface_source_range(surface: &AstOpaqueSurface) -> Option<SourceRange> {
    location_source_range(&surface.location)
}

fn location_source_range(location: &Location) -> Option<SourceRange> {
    Some(SourceRange {
        line_start: i64::try_from(location.line).ok()?,
        line_end: i64::try_from(location.end_line).ok()?,
        column_start: i64::try_from(location.column).ok()?,
        column_end: i64::try_from(location.end_column).ok()?,
    })
}

fn source_ranges_overlap(left: SourceRange, right: SourceRange) -> bool {
    if left.line_end < right.line_start || right.line_end < left.line_start {
        return false;
    }
    if left.line_start == left.line_end
        && right.line_start == right.line_end
        && left.line_start == right.line_start
    {
        return left.column_start < right.column_end && right.column_start < left.column_end;
    }
    true
}
