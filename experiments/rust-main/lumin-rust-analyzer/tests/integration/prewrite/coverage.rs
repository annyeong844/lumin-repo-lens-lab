use anyhow::{Context, Result};
use lumin_rust_source_health::{
    analyze_root, protocol::DEFAULT_WORKER_STACK_BYTES, RustSourceHealthOptions,
};

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
    assert_eq!(artifact["coverage"]["shapes"], "ran");
    assert_eq!(artifact["coverage"]["files"], "ran");
    assert_eq!(artifact["coverage"]["dependencies"], "not-requested");
    assert_eq!(artifact["coverage"]["plannedTypeEscapes"], "ran");
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
            .is_some_and(|text| text.contains("does not yet make complete absence claims"))));

    let unavailable = artifact["unavailableEvidence"]
        .as_array()
        .context("unavailable evidence")?;
    assert_eq!(unavailable.len(), 2);
    assert!(unavailable.iter().all(|entry| {
        entry["evidenceLane"] == "shape-hash"
            && entry["status"] == "UNAVAILABLE"
            && entry["reason"] == "lookup-unavailable"
            && entry["artifact"] == "rust-source-health"
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
fn prewrite_shape_hash_matches_rust_source_health_record_struct() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "src/lib.rs",
        br#"pub struct Event {
    pub id: u64,
    name: String,
}

pub struct EventMirror {
    name: String,
    pub id: u64,
}
"#,
    )?;
    let health = analyze_root(RustSourceHealthOptions {
        root: repo.root_path().to_path_buf(),
        source_commit: "test-source-commit".to_string(),
        thread_count: None,
        worker_stack_bytes: DEFAULT_WORKER_STACK_BYTES,
    })?;
    let shape_hash = health
        .files
        .get("src/lib.rs")
        .context("source-health file")?
        .ast
        .shape_hashes
        .iter()
        .find(|fact| fact.name == "Event")
        .context("Event shape hash")?
        .hash
        .clone();
    let artifact = repo.run_json(&format!(
        r#"{{
  "names": [],
  "shapes": [{{"hash": "{shape_hash}"}}],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}}"#
    ))?;

    assert_eq!(artifact["coverage"]["shapes"], "ran");
    let shape_lookup = &artifact["shapeLookups"][0];
    assert_eq!(shape_lookup["result"], "SHAPE_MATCH");
    assert_eq!(shape_lookup["shapeHash"], shape_hash);
    assert_eq!(shape_lookup["shapeHashSource"], "hash");
    let matches = shape_lookup["matches"]
        .as_array()
        .context("shape matches")?;
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0]["identity"], "src/lib.rs::Event");
    assert_eq!(matches[0]["ownerFile"], "src/lib.rs");
    assert_eq!(matches[0]["name"], "Event");
    assert_eq!(matches[0]["shapeKind"], "record-struct");
    assert_eq!(matches[0]["fields"][0]["name"], "id");
    assert_eq!(matches[0]["fields"][0]["type"], "u64");
    assert_eq!(matches[0]["fields"][0]["visibility"], "public");
    assert_eq!(matches[0]["fields"][1]["name"], "name");
    assert_eq!(matches[0]["fields"][1]["type"], "String");
    assert_eq!(matches[0]["fields"][1]["visibility"], "private");
    assert_eq!(matches[1]["identity"], "src/lib.rs::EventMirror");
    assert_eq!(matches[1]["hash"], shape_hash);
    assert!(artifact["unavailableEvidence"]
        .as_array()
        .context("unavailable evidence")?
        .iter()
        .all(|entry| entry["evidenceLane"] != "shape-hash"));
    assert!(artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .all(|card| card["cues"]
            .as_array()
            .into_iter()
            .flatten()
            .all(|cue| cue["evidenceLane"] != "shape-hash")));
    Ok(())
}

#[test]
fn prewrite_planned_type_escapes_are_ran_and_preserved_like_js_ts() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": [
    {
      "escapeKind": "as-unknown-as-T",
      "locationHint": "src/vendor/wrapper.rs::adapt_response",
      "codeShape": "response as unknown as ThirdPartyShape",
      "reason": "upstream SDK lacks type exports",
      "alternativeConsidered": "unknown plus decoder"
    },
    {
      "escapeKind": "ts-expect-error",
      "locationHint": "unknown",
      "reason": "mirrors JS/TS migration declaration shape"
    }
  ]
}"#,
    )?;

    assert_eq!(artifact["coverage"]["plannedTypeEscapes"], "ran");
    let escapes = artifact["intent"]["plannedTypeEscapes"]
        .as_array()
        .context("planned type escapes")?;
    assert_eq!(escapes.len(), 2);
    assert_eq!(escapes[0]["escapeKind"], "as-unknown-as-T");
    assert_eq!(
        escapes[0]["codeShape"],
        "response as unknown as ThirdPartyShape"
    );
    assert_eq!(escapes[0]["alternativeConsidered"], "unknown plus decoder");
    assert_eq!(escapes[1]["escapeKind"], "ts-expect-error");
    assert_eq!(escapes[1]["locationHint"], "unknown");
    assert!(artifact["unavailableEvidence"]
        .as_array()
        .context("unavailable evidence")?
        .iter()
        .all(|entry| entry["evidenceLane"] != "planned-type-escapes"));
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
