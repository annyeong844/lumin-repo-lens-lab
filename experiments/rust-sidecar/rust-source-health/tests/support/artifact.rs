#![allow(dead_code, unused_imports)]

#[path = "artifact/analysis.rs"]
mod analysis;
#[path = "artifact/file.rs"]
mod file;
#[path = "artifact/summary.rs"]
mod summary;

#[allow(unused_imports)]
pub use crate::cli::{run_sidecar, stdout_json};
#[allow(unused_imports)]
pub use crate::request::{file, request};
#[allow(unused_imports)]
pub use analysis::analyze_file;
#[allow(unused_imports)]
pub use file::{
    assert_file_fact_count, assert_file_parse_ok, file_health, file_signals, opaque_surfaces,
};
#[allow(unused_imports)]
pub use summary::{assert_syntax_artifact_metadata, summary_bucket_count, summary_count};
