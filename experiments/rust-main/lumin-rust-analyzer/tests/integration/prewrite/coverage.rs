use anyhow::{Context, Result};

use crate::support::prewrite::PreWriteRepo;
use crate::support::scenarios::single_package::analyze_metadata_only_single_package;

const SHAPE_HASH: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn prewrite_not_observed_keeps_opaque_taint_and_file_lane_visible() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    let artifact = repo.run_json(&format!(
        r#"{{
  "taskId": "TASK-42",
  "names": ["totally_missing_name"],
  "shapes": [{{"fields": ["id"]}}, {{"hash": "{SHAPE_HASH}"}}],
  "files": ["src/new.rs"]
}}"#
    ))?;

    assert_eq!(artifact["intent"]["taskId"], "TASK-42");
    assert_eq!(artifact["coverage"]["names"], "ran");
    assert_eq!(artifact["coverage"]["shapes"], "unsupported");
    assert_eq!(artifact["coverage"]["files"], "ran");
    assert_eq!(artifact["coverage"]["dependencies"], "not-requested");
    assert_eq!(artifact["coverage"]["plannedTypeEscapes"], "not-requested");
    assert_eq!(artifact["lookups"][0]["result"], "NOT_OBSERVED");
    let shape_lookups = artifact["shapeLookups"]
        .as_array()
        .context("shape lookups")?;
    assert_eq!(shape_lookups.len(), 2);
    assert_eq!(shape_lookups[0]["kind"], "shape");
    assert_eq!(shape_lookups[0]["result"], "UNAVAILABLE");
    assert_eq!(
        shape_lookups[0]["shape"]["fields"],
        serde_json::json!(["id"])
    );
    assert!(shape_lookups[0]["citations"]
        .as_array()
        .context("fields-only shape citations")?
        .iter()
        .any(|citation| citation.as_str().is_some_and(|text| {
            text.contains("field names alone are not structural equality evidence")
        })));
    assert_eq!(shape_lookups[1]["shapeHash"], SHAPE_HASH);
    assert!(shape_lookups[1]["citations"]
        .as_array()
        .context("hash shape citations")?
        .iter()
        .any(|citation| citation
            .as_str()
            .is_some_and(|text| text.contains("Rust pre-write shape lookup is unsupported"))));

    let unavailable = artifact["unavailableEvidence"]
        .as_array()
        .context("unavailable evidence")?;
    assert_eq!(unavailable.len(), 2);
    assert!(unavailable.iter().all(|entry| {
        entry["evidenceLane"] == "shape-hash"
            && entry["status"] == "UNAVAILABLE"
            && entry["reason"] == "lookup-unavailable"
            && entry["artifact"] == "shape-index.json"
    }));
    assert!(artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .all(|card| card["cues"]
            .as_array()
            .into_iter()
            .flatten()
            .all(|cue| cue["evidenceLane"] != "shape-hash")));
    assert_eq!(artifact["fileLookups"][0]["intentFile"], "src/new.rs");
    assert_eq!(artifact["fileLookups"][0]["result"], "NEW_FILE");
    assert_eq!(
        artifact["fileLookups"][0]["boundary"]["status"],
        "NOT_EVALUATED"
    );
    assert!(
        artifact["lookups"][0]["taintedBy"]["reviewOpaqueSurfaces"]
            .as_u64()
            .context("review opaque surfaces")?
            > 0
    );
    assert!(artifact["lookups"][0]["citations"]
        .as_array()
        .context("citations")?
        .iter()
        .any(|citation| citation
            .as_str()
            .is_some_and(|text| text.contains("not an absence claim"))));
    assert_eq!(
        artifact["intentWarnings"]
            .as_array()
            .context("warnings")?
            .len(),
        2
    );
    Ok(())
}

#[test]
fn prewrite_output_is_deterministic_and_does_not_change_legacy_artifact_shape() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    let intent = r#"{
  "names": ["load_task"],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#;
    let first = repo.run_json(intent)?;
    let second = repo.run_json(intent)?;

    assert_eq!(first, second);
    assert!(first.get("generated").is_none());
    assert!(first["meta"].get("generated").is_none());
    assert!(first.get("definitionIndex").is_none());
    assert!(first.get("implMethodIndex").is_none());

    let legacy = analyze_metadata_only_single_package("pub fn demo() {}\n")?;
    assert!(legacy.get("preWrite").is_none());
    assert!(legacy.get("cueCards").is_none());
    assert!(legacy.get("lookups").is_none());
    Ok(())
}

#[test]
fn prewrite_refactor_sources_report_inline_pattern_unavailable_without_cues() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": [],
  "refactorSources": [
    {
      "file": "src/lib.rs",
      "lines": [3, 4],
      "why": "extract repeated inline handling"
    }
  ]
}"#,
    )?;

    assert_eq!(artifact["coverage"]["inlinePatterns"], "unsupported");
    assert_eq!(
        artifact["intent"]["refactorSources"][0]["file"],
        "src/lib.rs"
    );
    assert_eq!(
        artifact["intent"]["refactorSources"][0]["lines"],
        serde_json::json!([3, 4])
    );

    let inline_lookups = artifact["inlinePatternLookups"]
        .as_array()
        .context("inline pattern lookups")?;
    assert_eq!(inline_lookups.len(), 1);
    assert_eq!(inline_lookups[0]["kind"], "inline-pattern");
    assert_eq!(inline_lookups[0]["result"], "UNAVAILABLE");
    assert_eq!(inline_lookups[0]["reason"], "missing-artifact");
    assert_eq!(inline_lookups[0]["artifact"], "inline-patterns.json");

    let unavailable = artifact["unavailableEvidence"]
        .as_array()
        .context("unavailable evidence")?;
    assert!(unavailable.iter().any(|entry| {
        entry["evidenceLane"] == "inline-extraction"
            && entry["status"] == "UNAVAILABLE"
            && entry["reason"] == "missing-artifact"
            && entry["artifact"] == "inline-patterns.json"
    }));
    assert!(artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .all(|card| card["cues"]
            .as_array()
            .into_iter()
            .flatten()
            .all(|cue| cue["evidenceLane"] != "inline-extraction")));
    assert!(artifact["suppressedCues"]
        .as_array()
        .context("suppressed cues")?
        .iter()
        .all(|cue| cue["evidenceLane"] != "inline-extraction"));
    Ok(())
}
