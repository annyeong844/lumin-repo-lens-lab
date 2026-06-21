use std::ffi::OsStr;
use std::fs;

use anyhow::Result;
use serde_json::Value;
use tempfile::TempDir;

use crate::support::fixtures::package;
use crate::support::scenarios::run;

pub fn analyze_cargo_check_single_package(lib_rs: &str) -> Result<Value> {
    analyze_single_package(lib_rs, Some("cargo-check"))
}

pub fn analyze_cargo_check_single_package_with_adjudication(
    lib_rs: &str,
    adjudication_json: &str,
) -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    let adjudication_path = temp.path().join("safe-fix-adjudication.json");
    package::write_single_package_crate(&root, "app", lib_rs)?;
    fs::write(&adjudication_path, adjudication_json)?;
    let extra_args = [
        OsStr::new("--calibration-adjudication"),
        adjudication_path.as_os_str(),
    ];
    run::run_analyzer_with_args(&root, Some("cargo-check"), &extra_args)
}

pub fn analyze_metadata_only_single_package(lib_rs: &str) -> Result<Value> {
    analyze_single_package(lib_rs, None)
}

pub fn analyze_metadata_only_single_package_with_invalid_utf8_file(lib_rs: &str) -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    package::write_single_package_crate(&root, "app", lib_rs)?;
    fs::write(root.join("src").join("bad.rs"), [0xff, 0xfe, b'\n'])?;
    run::run_analyzer(&root, None)
}

pub fn analyze_targeted_single_package(lib_rs: &str) -> Result<Value> {
    analyze_single_package(lib_rs, Some("targeted-cargo-check"))
}

pub fn analyze_targeted_single_package_with_integration(
    lib_rs: &str,
    integration_rs: &str,
) -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    package::write_single_package_crate(&root, "app", lib_rs)?;
    fs::create_dir_all(root.join("tests"))?;
    fs::write(root.join("tests").join("integration.rs"), integration_rs)?;
    run::run_analyzer(&root, Some("targeted-cargo-check"))
}

fn analyze_single_package(lib_rs: &str, semantic_mode: Option<&str>) -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    package::write_single_package_crate(&root, "app", lib_rs)?;
    run::run_analyzer(&root, semantic_mode)
}
