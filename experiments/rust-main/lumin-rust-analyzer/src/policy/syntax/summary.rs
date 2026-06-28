use lumin_rust_source_health::protocol::{FileHealth, SignalVisibility};

use crate::policy::FileParseStatus;

use super::ast::{ast_opaque_surface_counts, AstOpaqueSurfaceCounts};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProductSyntaxFileSummary {
    parse_status: FileParseStatus,
    parse_errors: usize,
    review_signals: usize,
    muted_signals: usize,
    review_opaque_surfaces: usize,
    muted_opaque_surfaces: usize,
}

impl ProductSyntaxFileSummary {
    pub(crate) fn missing() -> Self {
        Self {
            parse_status: FileParseStatus::Missing,
            parse_errors: 0,
            review_signals: 0,
            muted_signals: 0,
            review_opaque_surfaces: 0,
            muted_opaque_surfaces: 0,
        }
    }

    pub(in crate::policy) fn from_file(file: &FileHealth) -> Self {
        Self::from_file_with_opaque_surface_counts(file, ast_opaque_surface_counts(&file.ast))
    }

    pub(in crate::policy::syntax) fn from_file_with_opaque_surface_counts(
        file: &FileHealth,
        opaque_surface_counts: AstOpaqueSurfaceCounts,
    ) -> Self {
        Self {
            parse_status: parse_status(file),
            parse_errors: file.parse.errors.len(),
            review_signals: file
                .signals
                .iter()
                .filter(|signal| signal.visibility.visibility() == SignalVisibility::Review)
                .count(),
            muted_signals: file
                .signals
                .iter()
                .filter(|signal| signal.visibility.visibility() == SignalVisibility::Muted)
                .count(),
            review_opaque_surfaces: opaque_surface_counts.review(),
            muted_opaque_surfaces: opaque_surface_counts.muted(),
        }
    }

    pub(in crate::policy) fn is_present(self) -> bool {
        self.parse_status != FileParseStatus::Missing
    }

    pub(in crate::policy) fn parse_status(self) -> FileParseStatus {
        self.parse_status
    }

    pub(in crate::policy) fn parse_errors(self) -> usize {
        self.parse_errors
    }

    pub(in crate::policy) fn review_signals(self) -> usize {
        self.review_signals
    }

    pub(in crate::policy) fn muted_signals(self) -> usize {
        self.muted_signals
    }

    pub(in crate::policy) fn review_opaque_surfaces(self) -> usize {
        self.review_opaque_surfaces
    }

    pub(in crate::policy) fn muted_opaque_surfaces(self) -> usize {
        self.muted_opaque_surfaces
    }
}

fn parse_status(file: &FileHealth) -> FileParseStatus {
    if file.parse.ok && file.parse.errors.is_empty() {
        FileParseStatus::Ok
    } else {
        FileParseStatus::Error
    }
}
