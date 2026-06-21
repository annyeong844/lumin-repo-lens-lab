mod collection;
mod entry;

pub(super) use collection::ProductFiles;
pub(crate) use collection::{ProductFilesProjection, SemanticRefCounts};
pub(in crate::product_files) use entry::{SemanticDiagnosticRef, SemanticFindingRef};
