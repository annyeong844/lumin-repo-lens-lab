mod analyzer;
mod driver;
mod locations;
mod parallel;
pub mod protocol;
mod signals;
mod summary;
mod wrapper;

pub use driver::{analyze_request, main_entry, run_from_args, FinalMeta};
pub(crate) use lumin_rust_common::{is_usage_error, usage_error};
pub use wrapper::{analyze_root, run_cli, RustSourceHealthOptions};
