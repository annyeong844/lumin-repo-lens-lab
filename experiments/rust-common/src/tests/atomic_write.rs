use super::support::TestDir;
use crate::{atomic_write_json, atomic_write_json_pretty};
use serde_json::json;
use std::fs;
use std::io;

#[test]
fn atomic_write_json_pretty_creates_parent_and_trailing_newline() -> io::Result<()> {
    let temp = TestDir::new("atomic-write-pretty")?;
    let path = temp.path().join("artifacts").join("result.json");

    atomic_write_json_pretty(&path, &json!({ "ok": true }))?;

    let written = fs::read_to_string(path)?;
    assert!(written.contains("\"ok\": true"));
    assert!(written.ends_with('\n'));
    Ok(())
}

#[test]
fn atomic_write_json_keeps_compact_shape_for_size_sensitive_artifacts() -> io::Result<()> {
    let temp = TestDir::new("atomic-write-compact")?;
    let path = temp.path().join("result.json");

    atomic_write_json(&path, &json!({ "ok": true }))?;

    assert_eq!(fs::read_to_string(path)?, "{\"ok\":true}\n");
    Ok(())
}
