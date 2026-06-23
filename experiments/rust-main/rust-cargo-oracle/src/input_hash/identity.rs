use std::collections::BTreeMap;

use serde::Serialize;

use super::AnalysisInputSet;
use crate::protocol::{ArtifactProfile, CargoCheckMode, CargoTargetDirMode, RustcCommandSource};
use crate::toolchain::Toolchain;
use crate::DIAGNOSTIC_POLICY_VERSION;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AnalysisInputIdentity<'a> {
    policy_version: &'static str,
    registry_hash: &'a str,
    cargo_args: &'a [String],
    selected_package_ids: Vec<&'a str>,
    features: Option<&'a str>,
    package_name: Option<&'a str>,
    cargo_check_mode: CargoCheckMode,
    cargo_target_dir_mode: CargoTargetDirMode,
    target_paths: &'a [String],
    toolchain: ToolchainIdentity<'a>,
    compilation_environment: &'a BTreeMap<String, String>,
    file_hashes: BTreeMap<String, String>,
}

impl<'a> AnalysisInputIdentity<'a> {
    pub(super) fn new(
        input: &'a AnalysisInputSet<'a>,
        file_hashes: BTreeMap<String, String>,
    ) -> Self {
        Self {
            policy_version: DIAGNOSTIC_POLICY_VERSION,
            registry_hash: input.registry_hash,
            cargo_args: input.cargo_args,
            selected_package_ids: input.selected.iter().map(|pkg| pkg.id.as_str()).collect(),
            features: input.features,
            package_name: input.package_name,
            cargo_check_mode: input.cargo_check_mode,
            cargo_target_dir_mode: input.cargo_target_dir_mode,
            target_paths: input.target_paths,
            toolchain: ToolchainIdentity::from(input.toolchain),
            compilation_environment: input.compilation_environment.values(),
            file_hashes,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolchainIdentity<'a> {
    cargo_version: Option<&'a str>,
    rustc_version_verbose: Option<&'a str>,
    rustc_bin: &'a str,
    rustc_source: RustcCommandSource,
    host: Option<&'a str>,
    profile: ArtifactProfile,
}

impl<'a> From<&'a Toolchain> for ToolchainIdentity<'a> {
    fn from(toolchain: &'a Toolchain) -> Self {
        Self {
            cargo_version: toolchain.cargo_version.as_deref(),
            rustc_version_verbose: toolchain.rustc_version_verbose.as_deref(),
            rustc_bin: &toolchain.rustc_bin,
            rustc_source: toolchain.rustc_source,
            host: toolchain.host.as_deref(),
            profile: ArtifactProfile::Dev,
        }
    }
}
