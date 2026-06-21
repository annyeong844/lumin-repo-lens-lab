use std::path::Path;

use crate::command::run_command;
use crate::protocol::{ArtifactProfile, RustcCommandSource, ToolchainMeta};

#[derive(Debug, Clone)]
pub(crate) struct Toolchain {
    pub(crate) cargo_version: Option<String>,
    pub(crate) rustc_version_verbose: Option<String>,
    pub(crate) rustc_bin: String,
    pub(crate) rustc_source: RustcCommandSource,
    pub(crate) host: Option<String>,
}

pub(crate) fn collect_toolchain(root: &Path, cargo_bin: &str) -> Toolchain {
    let cargo_version = run_command(cargo_bin, &["--version".to_string()], root, None)
        .ok()
        .filter(|output| output.status == Some(0))
        .map(|output| output.stdout.trim().to_string());
    let (rustc_bin, rustc_source) = rustc_command_from_env();
    let rustc_version_verbose = run_command(&rustc_bin, &["-vV".to_string()], root, None)
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

fn rustc_command_from_env() -> (String, RustcCommandSource) {
    if let Some(value) = non_empty_env("CARGO_BUILD_RUSTC") {
        return (value, RustcCommandSource::CargoBuildRustc);
    }
    if let Some(value) = non_empty_env("RUSTC") {
        return (value, RustcCommandSource::RustcEnv);
    }
    ("rustc".to_string(), RustcCommandSource::DefaultRustc)
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
        profile: ArtifactProfile::Dev,
    }
}

#[cfg(test)]
mod tests {
    use super::rustc_command_from_env;
    use crate::protocol::RustcCommandSource;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn rustc_command_prefers_cargo_build_rustc_then_rustc() {
        let _guard = match ENV_LOCK.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let _build_env = EnvRestore::new("CARGO_BUILD_RUSTC");
        let _rustc_env = EnvRestore::new("RUSTC");

        std::env::set_var("CARGO_BUILD_RUSTC", "cargo-build-rustc");
        std::env::set_var("RUSTC", "plain-rustc");
        assert_eq!(
            rustc_command_from_env(),
            (
                "cargo-build-rustc".to_string(),
                RustcCommandSource::CargoBuildRustc
            )
        );

        std::env::remove_var("CARGO_BUILD_RUSTC");
        assert_eq!(
            rustc_command_from_env(),
            ("plain-rustc".to_string(), RustcCommandSource::RustcEnv)
        );

        std::env::remove_var("RUSTC");
        assert_eq!(
            rustc_command_from_env(),
            ("rustc".to_string(), RustcCommandSource::DefaultRustc)
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
