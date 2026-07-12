mod analysis;
pub(crate) mod cache;
mod entry;
mod validation;

pub(crate) use analysis::analyze_source_entries_compact_artifact;
pub(crate) use analysis::analyze_source_entries_with_options;
pub use analysis::CompactAnalysisResponse;
pub use analysis::{analyze_request, analyze_request_with_options, AnalysisOptions, FinalMeta};
pub use entry::{main_entry, run_from_args};
