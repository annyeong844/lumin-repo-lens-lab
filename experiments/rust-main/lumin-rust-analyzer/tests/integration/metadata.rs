use anyhow::{Context, Result};

use crate::support::metadata_mode::assert_metadata_only_without_cargo_check;
use crate::support::scenarios::single_package::{
    analyze_metadata_only_single_package,
    analyze_metadata_only_single_package_with_invalid_utf8_file,
};

#[test]
fn unified_cli_defaults_to_metadata_only_without_running_cargo_check() -> Result<()> {
    let artifact = analyze_metadata_only_single_package("pub fn demo() {}\n")?;
    assert_metadata_only_without_cargo_check(&artifact)
}

#[test]
fn unified_cli_prioritizes_strong_review_signal_examples() -> Result<()> {
    let artifact = analyze_metadata_only_single_package(
        "pub fn demo() { let value = String::from(\"x\"); let _copy = value.clone(); panic!(\"one\"); panic!(\"two\"); unsafe { let _ = 1; } }\n",
    )?;

    assert_eq!(
        artifact["summary"]["syntaxReviewSignalExamples"][0]["kind"],
        "panic-macro"
    );
    assert_eq!(
        artifact["summary"]["syntaxReviewSignalExamples"][1]["kind"],
        "unsafe-block"
    );
    assert_eq!(
        artifact["summary"]["syntaxReviewSignalExamples"][2]["kind"],
        "clone-call"
    );
    Ok(())
}

#[test]
fn unified_cli_projects_parse_errors_as_capped_examples() -> Result<()> {
    let artifact = analyze_metadata_only_single_package("pub fn broken( {\n")?;
    let syntax = &artifact["files"]["src/lib.rs"]["syntax"];
    let parse = &syntax["parse"];

    assert_eq!(parse["ok"], false);
    assert!(parse["errorCount"].as_u64().context("parse error count")? > 0);
    assert_eq!(parse["sampleLimit"], 3);
    assert!(parse["errors"].is_null());
    let examples = parse["errorExamples"]
        .as_array()
        .context("parse error examples")?;
    assert!(!examples.is_empty());
    assert!(examples.len() <= 3);
    assert_eq!(examples[0]["claim"], "syntax-only");
    assert_eq!(artifact["summary"]["syntaxParseErrorFiles"], 1);
    assert!(
        artifact["summary"]["syntaxParseErrors"]
            .as_u64()
            .context("syntax parse errors")?
            > 0
    );
    Ok(())
}

#[test]
fn unified_cli_projects_skipped_files_as_capped_phase_examples() -> Result<()> {
    let artifact =
        analyze_metadata_only_single_package_with_invalid_utf8_file("pub fn demo() {}\n")?;
    let syntax_phase = &artifact["phases"]["syntax"];
    let skipped_examples = syntax_phase["skippedFileExamples"]
        .as_array()
        .context("syntax skipped file examples")?;

    assert_eq!(artifact["summary"]["files"], 1);
    assert_eq!(syntax_phase["summary"]["skippedFiles"], 1);
    assert_eq!(syntax_phase["skippedFileCount"], 1);
    assert_eq!(skipped_examples.len(), 1);
    assert_eq!(skipped_examples[0]["path"], "src/bad.rs");
    assert_eq!(skipped_examples[0]["reason"], "invalid-utf8");
    assert!(syntax_phase.get("skippedFiles").is_none());
    assert!(artifact.get("skippedFiles").is_none());
    Ok(())
}
