mod collection;
mod entry;
mod projection;
mod refs;

pub(super) use collection::ProductFiles;
pub(in crate::product_files) use entry::{SemanticDiagnosticRef, SemanticFindingRef};
pub(crate) use projection::ProductFilesProjection;
pub(crate) use refs::SemanticRefCounts;
