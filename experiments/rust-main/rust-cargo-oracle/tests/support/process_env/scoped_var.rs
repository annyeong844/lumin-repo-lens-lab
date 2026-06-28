use std::ffi::{OsStr, OsString};

pub fn with_env_var<T>(key: &str, value: Option<&OsStr>, run: impl FnOnce() -> T) -> T {
    let previous = std::env::var_os(key);
    set_env_var(key, value);
    let result = run();
    restore_env_var(key, previous);
    result
}

fn set_env_var(key: &str, value: Option<&OsStr>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

fn restore_env_var(key: &str, value: Option<OsString>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}
