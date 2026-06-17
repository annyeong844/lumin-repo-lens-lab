use serde_json::{json, Value};
use std::path::Path;

use crate::command::run_command;
use crate::protocol::ToolchainMeta;

#[derive(Debug, Clone)]
pub(crate) struct Toolchain {
    pub(crate) cargo_version: Option<String>,
    pub(crate) rustc_version_verbose: Option<String>,
    pub(crate) rustc_bin: String,
    pub(crate) rustc_source: &'static str,
    pub(crate) host: Option<String>,
}

pub(crate) fn collect_toolchain(root: &Path, cargo_bin: &str, timeout_ms: u64) -> Toolchain {
    let cargo_version = run_command(cargo_bin, &["--version".to_string()], root, timeout_ms)
        .ok()
        .filter(|output| output.status == Some(0))
        .map(|output| output.stdout.trim().to_string());
    let (rustc_bin, rustc_source) = rustc_command_from_env();
    let rustc_version_verbose = run_command(&rustc_bin, &["-vV".to_string()], root, timeout_ms)
        .ok()
        .filter(|output| output.status == Some(0))
        .map(|output| output.stdout.trim().to_string());
    let host = rustc_version_verbose
        .as_deref()
        .and_then(|text| text.lines().find_map(|line| line.strip_prefix("host: ")))
        .map(str::to_string);
    Toolchain {
        cargo_version,
        rustc_version_verbose,
        rustc_bin,
        rustc_source,
        host,
    }
}

fn rustc_command_from_env() -> (String, &'static str) {
    if let Some(value) = non_empty_env("CARGO_BUILD_RUSTC") {
        return (value, "env:CARGO_BUILD_RUSTC");
    }
    if let Some(value) = non_empty_env("RUSTC") {
        return (value, "env:RUSTC");
    }
    ("rustc".to_string(), "default:rustc")
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn toolchain_meta(toolchain: &Toolchain) -> ToolchainMeta {
    ToolchainMeta {
        cargo_version: toolchain.cargo_version.clone(),
        rustc_version_verbose: toolchain.rustc_version_verbose.clone(),
        rustc_bin: toolchain.rustc_bin.clone(),
        rustc_source: toolchain.rustc_source,
        host: toolchain.host.clone(),
        profile: "dev",
    }
}

pub(crate) fn toolchain_json(toolchain: &Toolchain) -> Value {
    json!({
        "cargoVersion": toolchain.cargo_version,
        "rustcVersionVerbose": toolchain.rustc_version_verbose,
        "rustcBin": toolchain.rustc_bin,
        "rustcSource": toolchain.rustc_source,
        "host": toolchain.host,
        "profile": "dev",
    })
}

#[cfg(test)]
mod tests {
    use super::rustc_command_from_env;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn rustc_command_prefers_cargo_build_rustc_then_rustc() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let _build_env = EnvRestore::new("CARGO_BUILD_RUSTC");
        let _rustc_env = EnvRestore::new("RUSTC");

        std::env::set_var("CARGO_BUILD_RUSTC", "cargo-build-rustc");
        std::env::set_var("RUSTC", "plain-rustc");
        assert_eq!(
            rustc_command_from_env(),
            ("cargo-build-rustc".to_string(), "env:CARGO_BUILD_RUSTC")
        );

        std::env::remove_var("CARGO_BUILD_RUSTC");
        assert_eq!(
            rustc_command_from_env(),
            ("plain-rustc".to_string(), "env:RUSTC")
        );

        std::env::remove_var("RUSTC");
        assert_eq!(
            rustc_command_from_env(),
            ("rustc".to_string(), "default:rustc")
        );
    }

    struct EnvRestore {
        key: &'static str,
        value: Option<std::ffi::OsString>,
    }

    impl EnvRestore {
        fn new(key: &'static str) -> Self {
            Self {
                key,
                value: std::env::var_os(key),
            }
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            if let Some(value) = &self.value {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }
}
