use std::ffi::OsString;

use anyhow::Result;

const COMPILATION_ENV_KEYS: [&str; 4] = [
    "CARGO_BUILD_TARGET",
    "RUSTFLAGS",
    "CARGO_BUILD_RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
];

pub fn with_clean_compilation_env<T>(run: impl FnOnce() -> Result<T>) -> Result<T> {
    let previous = save_compilation_env();
    clear_compilation_env();
    let result = run();
    restore_compilation_env(previous);
    result
}

fn save_compilation_env() -> Vec<(&'static str, Option<OsString>)> {
    COMPILATION_ENV_KEYS
        .into_iter()
        .map(|key| (key, std::env::var_os(key)))
        .collect()
}

fn clear_compilation_env() {
    for key in COMPILATION_ENV_KEYS {
        std::env::remove_var(key);
    }
}

fn restore_compilation_env(previous: Vec<(&'static str, Option<OsString>)>) {
    for (key, value) in previous {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
