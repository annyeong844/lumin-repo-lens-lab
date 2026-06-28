mod cli;
mod error;
mod path;

#[cfg(feature = "hash")]
mod hash;

#[cfg(feature = "json")]
mod json;

pub use cli::{
    parse_enum, parse_min_usize, parse_nonzero_usize, take_path, take_string, CliAction, CliResult,
};
pub use error::{is_usage_error, usage_error, UsageError};

#[cfg(feature = "json")]
pub use json::{atomic_write_json, atomic_write_json_pretty};

pub use path::{
    canonical_existing_dir, canonical_existing_dir_usage, find_repo_root,
    find_repo_root_with_fallback, path_has_segment, posix_path_has_segment, posix_path_text,
};

#[cfg(feature = "hash")]
pub use hash::{sha256_bytes, sha256_file, sha256_text};

#[cfg(test)]
mod tests;
