use anyhow::{Context, Result};
use std::fs;
use tempfile::TempDir;

use crate::rustc_span::RustcSpan;

use super::{OwnershipResolver, SpanClass};

#[test]
fn metadata_unavailable_fallback_keeps_root_src_diagnostic_user_code() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("crate");
    fs::create_dir_all(root.join("src"))?;

    let resolver = OwnershipResolver::new(&root, None, &[]);
    let span =
        rustc_span(r#"{"file_name":"src/lib.rs","is_primary":true}"#).context("rustc span")?;

    assert_eq!(
        resolver.classify_span_for_package(&span, Some("unknown-package")),
        SpanClass::UserCode
    );
    Ok(())
}

#[test]
fn metadata_unavailable_fallback_does_not_mark_dependency_shaped_path_user_code() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("crate");
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("bad_dep").join("src"))?;

    let resolver = OwnershipResolver::new(&root, None, &[]);
    let span = rustc_span(r#"{"file_name":"bad_dep/src/lib.rs","is_primary":true}"#)
        .context("rustc span")?;

    assert_eq!(
        resolver.classify_span_for_package(&span, Some("unknown-package")),
        SpanClass::Unknown
    );
    Ok(())
}

fn rustc_span(raw: &str) -> Result<RustcSpan, serde_json::Error> {
    serde_json::from_str(raw)
}
