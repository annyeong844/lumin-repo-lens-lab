use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;
use tempfile::TempDir;

use crate::cli::run_cli;

#[test]
fn compact_cli_reuses_unchanged_file_facts_and_invalidates_changed_content() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("src/lib.rs"),
        r#"
pub fn alpha() -> Option<u8> {
    Some(1)
}

fn private_helper() -> u8 {
    alpha().unwrap_or(0)
}
"#,
    )?;
    fs::write(
        root.join("src/extra.rs"),
        r#"
pub(crate) fn beta() -> Result<u8, ()> {
    Ok(2)
}
"#,
    )?;
    let cache_root = temp.path().join("cache root with spaces");

    let cold = run_compact(&root, &cache_root, &temp.path().join("cold.json"))?;
    assert_eq!(cold["meta"]["incremental"]["enabled"], true);
    assert_eq!(
        cold["meta"]["incremental"]["identityMode"],
        "strict-content-hash"
    );
    assert_eq!(cold["meta"]["incremental"]["changedFiles"], 2);
    assert_eq!(cold["meta"]["incremental"]["reusedFiles"], 0);

    let warm = run_compact(&root, &cache_root, &temp.path().join("warm.json"))?;
    assert_eq!(warm["summary"]["files"], cold["summary"]["files"]);
    assert_eq!(warm["summary"]["signals"], cold["summary"]["signals"]);
    assert_eq!(warm["meta"]["incremental"]["enabled"], true);
    assert_eq!(warm["meta"]["incremental"]["changedFiles"], 0);
    assert_eq!(warm["meta"]["incremental"]["reusedFiles"], 2);
    assert_eq!(warm["meta"]["incremental"]["droppedFiles"], 0);

    fs::write(
        root.join("src/extra.rs"),
        r#"
pub(crate) fn beta() -> Result<u8, ()> {
    Ok(3)
}

fn gamma() {
    let _ = beta();
}
"#,
    )?;
    let changed = run_compact(&root, &cache_root, &temp.path().join("changed.json"))?;
    assert_eq!(changed["meta"]["incremental"]["changedFiles"], 1);
    assert_eq!(changed["meta"]["incremental"]["reusedFiles"], 1);
    assert_eq!(changed["meta"]["incremental"]["invalidatedFiles"], 1);
    assert_eq!(changed["summary"]["files"], 2);
    assert!(
        changed["summary"]["definitions"]
            .as_u64()
            .context("changed definitions")?
            > warm["summary"]["definitions"]
                .as_u64()
                .context("warm definitions")?
    );

    Ok(())
}

#[test]
fn compact_cli_streams_warm_clone_cache_without_changing_clone_outputs() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("src/alpha.rs"),
        r#"
fn duplicate_alpha() -> Option<u8> {
    Some(9)
}

pub fn exported_alpha() -> Option<u8> {
    duplicate_alpha()
}
"#,
    )?;
    fs::write(
        root.join("src/beta.rs"),
        r#"
fn duplicate_beta() -> Option<u8> {
    Some(9)
}

pub fn exported_beta() -> Option<u8> {
    duplicate_beta()
}
"#,
    )?;
    fs::write(
        root.join("src/near_a.rs"),
        r#"
fn normalize_domain_value(input: &str) -> String {
    input.trim().to_string()
}

fn render_domain_value(input: &str) -> String {
    format!("render:{input}")
}

pub fn convert_search_flag(input: &str) -> String {
    let normalized = normalize_domain_value(input);
    render_domain_value(&normalized)
}
"#,
    )?;
    fs::write(
        root.join("src/near_b.rs"),
        r#"
fn publish_domain_value(input: &str) -> String {
    format!("publish:{input}")
}

pub fn convert_output_flag(input: &str) -> String {
    let normalized = normalize_domain_value(input);
    publish_domain_value(&normalized)
}
"#,
    )?;
    let cache_root = temp.path().join("cache");

    let cold = run_compact(&root, &cache_root, &temp.path().join("cold.json"))?;
    assert!(
        cold["functionCloneGroups"]["exactBodyGroupCount"]
            .as_u64()
            .context("cold exact body groups")?
            > 0
    );
    let warm = run_compact(&root, &cache_root, &temp.path().join("warm.json"))?;

    assert_eq!(warm["meta"]["incremental"]["changedFiles"], 0);
    assert_eq!(warm["meta"]["incremental"]["reusedFiles"], 4);
    assert_eq!(warm["functionCloneGroups"], cold["functionCloneGroups"]);

    Ok(())
}

#[test]
fn compact_cli_stores_incremental_cache_in_bounded_shards() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("src"))?;
    for index in 0..24 {
        fs::write(
            root.join("src").join(format!("module_{index}.rs")),
            format!(
                r#"
pub fn exported_{index}() -> usize {{
    {index}
}}
"#
            ),
        )?;
    }
    let cache_root = temp.path().join("cache");

    let artifact = run_compact(&root, &cache_root, &temp.path().join("cold.json"))?;
    assert_eq!(artifact["meta"]["incremental"]["changedFiles"], 24);
    assert_eq!(artifact["summary"]["files"], 24);
    let lane_counts = lane_json_file_counts(&cache_root)?;
    let cache_json_files: usize = lane_counts.iter().map(|(_, count)| *count).sum();
    assert!(
        lane_counts
            .iter()
            .all(|(_, count)| (1..=16).contains(count)),
        "expected each cache lane to use bounded shard files, got {lane_counts:?}"
    );
    assert_eq!(
        lane_counts
            .iter()
            .map(|(lane, _)| lane.as_str())
            .collect::<Vec<_>>(),
        ["clone", "dead", "summary"]
    );

    let warm = run_compact(&root, &cache_root, &temp.path().join("warm.json"))?;
    assert_eq!(warm["meta"]["incremental"]["changedFiles"], 0);
    assert_eq!(warm["meta"]["incremental"]["reusedFiles"], 24);
    assert_eq!(count_json_files(&cache_root)?, cache_json_files);

    Ok(())
}

#[test]
fn compact_cli_rebuilds_missing_lane_shards_without_zeroing_phase_outputs() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("src/alpha.rs"),
        r#"
fn duplicate_alpha() -> Option<u8> {
    Some(7)
}

pub fn exported_alpha() -> Option<u8> {
    duplicate_alpha()
}
"#,
    )?;
    fs::write(
        root.join("src/beta.rs"),
        r#"
fn duplicate_beta() -> Option<u8> {
    Some(7)
}

pub fn exported_beta() -> Option<u8> {
    duplicate_beta()
}
"#,
    )?;
    let cache_root = temp.path().join("cache");

    let cold = run_compact(&root, &cache_root, &temp.path().join("cold.json"))?;
    assert!(
        cold["functionCloneGroups"]["exactBodyGroupCount"]
            .as_u64()
            .context("cold exact body groups")?
            > 0
    );
    let cold_definition_count = cold["unusedDefinitionAnalysis"]["summary"]["definitionCount"]
        .as_u64()
        .context("cold definition count")?;
    assert!(cold_definition_count > 0);

    remove_lane_json_files(&cache_root, "clone")?;
    let clone_rebuilt = run_compact(&root, &cache_root, &temp.path().join("clone-rebuilt.json"))?;
    assert!(
        clone_rebuilt["meta"]["incremental"]["changedFiles"]
            .as_u64()
            .context("clone rebuilt changed files")?
            > 0
    );
    assert_eq!(
        clone_rebuilt["functionCloneGroups"]["exactBodyGroupCount"],
        cold["functionCloneGroups"]["exactBodyGroupCount"]
    );
    assert_eq!(
        clone_rebuilt["unusedDefinitionAnalysis"]["summary"]["definitionCount"],
        cold["unusedDefinitionAnalysis"]["summary"]["definitionCount"]
    );

    remove_lane_json_files(&cache_root, "dead")?;
    let dead_rebuilt = run_compact(&root, &cache_root, &temp.path().join("dead-rebuilt.json"))?;
    assert!(
        dead_rebuilt["meta"]["incremental"]["changedFiles"]
            .as_u64()
            .context("dead rebuilt changed files")?
            > 0
    );
    assert_eq!(
        dead_rebuilt["functionCloneGroups"]["exactBodyGroupCount"],
        cold["functionCloneGroups"]["exactBodyGroupCount"]
    );
    assert_eq!(
        dead_rebuilt["unusedDefinitionAnalysis"]["summary"]["definitionCount"],
        cold["unusedDefinitionAnalysis"]["summary"]["definitionCount"]
    );

    Ok(())
}

fn run_compact(root: &Path, cache_root: &Path, output_path: &Path) -> Result<Value> {
    let output = run_cli(&[
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--output".to_string(),
        output_path.to_string_lossy().to_string(),
        "--source-commit".to_string(),
        "test-source-commit".to_string(),
        "--cache-root".to_string(),
        cache_root.to_string_lossy().to_string(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&fs::read(output_path)?).context("parse compact rust-health artifact")
}

fn count_json_files(root: &Path) -> Result<usize> {
    let mut count = 0usize;
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(read_dir) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in read_dir {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("json")
            {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn lane_json_file_counts(root: &Path) -> Result<Vec<(String, usize)>> {
    let producer_dir = find_cache_producer_dir(root)?;
    let mut counts = Vec::new();
    for lane in ["clone", "dead", "summary"] {
        counts.push((
            lane.to_string(),
            count_json_files(&producer_dir.join(lane))?,
        ));
    }
    Ok(counts)
}

fn remove_lane_json_files(root: &Path, lane: &str) -> Result<()> {
    let lane_dir = find_cache_producer_dir(root)?.join(lane);
    let Ok(read_dir) = fs::read_dir(&lane_dir) else {
        return Ok(());
    };
    for entry in read_dir {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("json")
        {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

fn find_cache_producer_dir(root: &Path) -> Result<std::path::PathBuf> {
    let incremental = root.join("incremental");
    let mut pending = vec![incremental];
    while let Some(dir) = pending.pop() {
        let Ok(read_dir) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in read_dir {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if entry.file_name() == "rust-source-health-compact" {
                return Ok(entry.path());
            }
            pending.push(entry.path());
        }
    }
    anyhow::bail!(
        "missing rust-source-health compact cache producer directory under {}",
        root.display()
    )
}
