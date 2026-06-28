mod merge;
mod model;
mod path;
mod semantic_diagnostics;
mod semantic_findings;

pub(crate) use merge::merged_files;
pub(crate) use model::{ProductFilesProjection, SemanticRefCounts};
pub(crate) use semantic_diagnostics::{
    semantic_diagnostics_with_paths, ProductSemanticDiagnosticsProjection,
};
pub(crate) use semantic_findings::{
    semantic_findings_with_oracle_provenance, ProductSemanticFindingsProjection,
};
