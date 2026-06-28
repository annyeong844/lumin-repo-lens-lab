use std::ffi::OsStr;

use anyhow::Result;

use super::super::scoped_var::with_env_var;

pub fn with_rustflags<T>(value: Option<&str>, run: impl FnOnce() -> Result<T>) -> Result<T> {
    with_env_var("RUSTFLAGS", value.map(OsStr::new), run)
}
