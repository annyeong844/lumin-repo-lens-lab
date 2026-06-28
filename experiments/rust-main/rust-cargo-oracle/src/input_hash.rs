use std::path::Path;

use lumin_rust_common::sha256_text;

use crate::environment::CompilationEnvironment;
use crate::metadata::{CargoMetadata, CargoPackage};
use crate::protocol::{CargoCheckMode, CargoTargetDirMode};
use crate::toolchain::Toolchain;

mod files;
mod identity;

use files::collect_input_file_hashes;
use identity::AnalysisInputIdentity;

pub(crate) struct AnalysisInputSet<'a> {
    pub(crate) root: &'a Path,
    pub(crate) metadata: Option<&'a CargoMetadata>,
    pub(crate) cargo_args: &'a [String],
    pub(crate) selected: &'a [CargoPackage],
    pub(crate) registry_hash: &'a str,
    pub(crate) compilation_environment: &'a CompilationEnvironment,
    pub(crate) features: Option<&'a str>,
    pub(crate) package_name: Option<&'a str>,
    pub(crate) toolchain: &'a Toolchain,
    pub(crate) cargo_check_mode: CargoCheckMode,
    pub(crate) cargo_target_dir_mode: CargoTargetDirMode,
    pub(crate) target_paths: &'a [String],
}

pub(crate) fn analysis_input_set_hash(input: AnalysisInputSet<'_>) -> serde_json::Result<String> {
    let file_hashes = collect_input_file_hashes(
        input.root,
        input.metadata,
        input.selected,
        input.compilation_environment,
    );
    let identity = AnalysisInputIdentity::new(&input, file_hashes);
    serde_json::to_string(&identity).map(|text| sha256_text(&text))
}
