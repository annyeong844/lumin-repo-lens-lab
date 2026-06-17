use serde_json::{json, Value};
use std::path::Path;

use crate::command::run_command;
use crate::protocol::ToolchainMeta;

#[derive(Debug, Clone)]
pub(crate) struct Toolchain {
    pub(crate) cargo_version: Option<String>,
    pub(crate) rustc_version_verbose: Option<String>,
    pub(crate) host: Option<String>,
}

pub(crate) fn collect_toolchain(root: &Path, cargo_bin: &str, timeout_ms: u64) -> Toolchain {
    let cargo_version = run_command(cargo_bin, &["--version".to_string()], root, timeout_ms)
        .ok()
        .filter(|output| output.status == Some(0))
        .map(|output| output.stdout.trim().to_string());
    let rustc_version_verbose = run_command("rustc", &["-vV".to_string()], root, timeout_ms)
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
        host,
    }
}

pub(crate) fn toolchain_meta(toolchain: &Toolchain) -> ToolchainMeta {
    ToolchainMeta {
        cargo_version: toolchain.cargo_version.clone(),
        rustc_version_verbose: toolchain.rustc_version_verbose.clone(),
        host: toolchain.host.clone(),
        profile: "dev",
    }
}

pub(crate) fn toolchain_json(toolchain: &Toolchain) -> Value {
    json!({
        "cargoVersion": toolchain.cargo_version,
        "rustcVersionVerbose": toolchain.rustc_version_verbose,
        "host": toolchain.host,
        "profile": "dev",
    })
}
