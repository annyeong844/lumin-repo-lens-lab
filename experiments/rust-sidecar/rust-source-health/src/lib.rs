mod analyzer;
mod dead_exports;
mod driver;
mod function_clones;
mod locations;
mod parallel;
mod path_policy;
pub mod protocol;
mod signals;
mod summary;
mod wrapper;

pub use driver::{
    analyze_request, analyze_request_with_options, main_entry, run_from_args, AnalysisOptions,
    CompactAnalysisResponse, FinalMeta,
};
pub(crate) use lumin_rust_common::{is_usage_error, usage_error};
pub use path_policy::is_test_like_rust_path;
pub use wrapper::{
    analyze_root, analyze_root_compact, run_cli, RustFileScanScope, RustSourceHealthOptions,
};
