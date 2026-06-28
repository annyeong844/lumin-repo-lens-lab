use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_file_lane_keeps_symlinked_rust_paths_unknown() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes("src/real.rs", b"pub fn real() {}\n")?;
    let target = repo.root_path().join("src").join("real.rs");
    let link = repo.root_path().join("src").join("linked.rs");
    if !create_file_symlink(&target, &link)? {
        return Ok(());
    }
    repo.write_bytes("real_dir/nested.rs", b"pub fn nested() {}\n")?;
    let target_dir = repo.root_path().join("real_dir");
    let link_dir = repo.root_path().join("src").join("linked_dir");
    if !create_dir_symlink(&target_dir, &link_dir)? {
        return Ok(());
    }

    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": ["src/linked.rs", "src/linked_dir/nested.rs"],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    let linked = file_lookup(&artifact, "src/linked.rs")?;
    assert_eq!(linked["result"], "FILE_STATUS_UNKNOWN");
    assert!(citations(linked).any(|citation| citation.contains("is a symlink")));
    assert!(cue_card(&artifact, "src/linked.rs::__file__").is_err());
    let linked_dir_file = file_lookup(&artifact, "src/linked_dir/nested.rs")?;
    assert_eq!(linked_dir_file["result"], "FILE_STATUS_UNKNOWN");
    assert!(citations(linked_dir_file)
        .any(|citation| citation.contains("'src/linked_dir' is a symlink")));
    assert!(cue_card(&artifact, "src/linked_dir/nested.rs::__file__").is_err());
    Ok(())
}

fn file_lookup<'a>(artifact: &'a Value, intent_file: &str) -> Result<&'a Value> {
    artifact["fileLookups"]
        .as_array()
        .context("fileLookups array")?
        .iter()
        .find(|lookup| lookup["intentFile"] == intent_file)
        .with_context(|| format!("lookup for {intent_file}"))
}

fn cue_card<'a>(artifact: &'a Value, identity: &str) -> Result<&'a Value> {
    artifact["cueCards"]
        .as_array()
        .context("cueCards array")?
        .iter()
        .find(|card| card["candidate"]["identity"] == identity)
        .with_context(|| format!("cue card {identity}"))
}

fn citations(lookup: &Value) -> impl Iterator<Item = &str> {
    lookup["citations"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

fn create_file_symlink(target: &Path, link: &Path) -> Result<bool> {
    create_symlink(
        || create_file_symlink_inner(target, link),
        "create symlinked Rust file fixture",
    )
}

fn create_dir_symlink(target: &Path, link: &Path) -> Result<bool> {
    create_symlink(
        || create_dir_symlink_inner(target, link),
        "create symlinked Rust directory fixture",
    )
}

fn create_symlink(
    create: impl FnOnce() -> std::io::Result<()>,
    context: &'static str,
) -> Result<bool> {
    match create() {
        Ok(()) => Ok(true),
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::PermissionDenied | ErrorKind::Unsupported
            ) || error.raw_os_error() == Some(1314) =>
        {
            Ok(false)
        }
        Err(error) => Err(error).context(context),
    }
}

#[cfg(unix)]
fn create_file_symlink_inner(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_file_symlink_inner(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(unix)]
fn create_dir_symlink_inner(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_dir_symlink_inner(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}
