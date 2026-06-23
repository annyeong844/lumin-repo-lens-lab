use anyhow::{Context, Result};
use lumin_rust_source_health::{
    analyze_root, protocol::DEFAULT_WORKER_STACK_BYTES, RustSourceHealthOptions,
};

use crate::support::prewrite::PreWriteRepo;

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
    let cue_cards = artifact["cueCards"].as_array().context("cue cards")?;
    let event_card = cue_cards
        .iter()
        .find(|card| card["candidate"]["identity"] == "src/lib.rs::Event")
        .context("Event shape cue card")?;
    assert_eq!(event_card["renderTier"], "SAFE_CUE");
    let event_shape_cue = event_card["cues"]
        .as_array()
        .context("Event cues")?
        .iter()
        .find(|cue| cue["evidenceLane"] == "shape-hash")
        .context("Event shape-hash cue")?;
    assert_eq!(event_shape_cue["cueTier"], "SAFE_CUE");
    assert_eq!(event_shape_cue["safeMeaning"], "claim-only");
    assert_eq!(event_shape_cue["claim"], "same normalized type shape");
    assert_eq!(
        event_shape_cue["notSafeFor"],
        serde_json::json!(["semantic-equivalence", "auto-reuse", "auto-fix"])
    );
    assert_eq!(
        event_shape_cue["evidence"][0]["artifact"],
        "rust-source-health"
    );
    assert_eq!(
        event_shape_cue["evidence"][0]["matchedField"],
        "files[].ast.shapeHashes[].hash"
    );
    assert_eq!(
        event_shape_cue["evidence"][0]["algorithmVersion"],
        "rust-shape-hash.normalized.v1"
    );
    assert_eq!(event_shape_cue["evidence"][0]["hash"], shape_hash);
    assert!(cue_cards
        .iter()
        .any(|card| card["candidate"]["identity"] == "src/lib.rs::EventMirror"));
    Ok(())
}

#[test]
fn prewrite_shape_and_signature_cues_preserve_path_policy_suppression() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "tests/helper.rs",
        br#"pub struct TestShape {
    pub id: u64,
}

pub fn parse_test(input: &str) -> usize {
    input.len()
}
"#,
    )?;
    let health = analyze_root(RustSourceHealthOptions {
        root: repo.root_path().to_path_buf(),
        source_commit: "test-source-commit".to_string(),
        thread_count: None,
        worker_stack_bytes: DEFAULT_WORKER_STACK_BYTES,
    })?;
    let test_file = health
        .files
        .get("tests/helper.rs")
        .context("tests/helper.rs health")?;
    let shape_hash = test_file
        .ast
        .shape_hashes
        .iter()
        .find(|fact| fact.name == "TestShape")
        .context("test shape hash")?
        .hash
        .clone();
    let signature_hash = test_file
        .ast
        .function_signatures
        .iter()
        .find(|fact| fact.name == "parse_test")
        .context("test function signature")?
        .hash
        .clone();
    let artifact = repo.run_json(&format!(
        r#"{{
  "names": [],
  "shapes": [{{"hash": "{shape_hash}"}}, {{"hash": "{signature_hash}"}}],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}}"#
    ))?;

    let cue_cards = artifact["cueCards"].as_array().context("cue cards")?;
    assert!(cue_cards.iter().all(|card| {
        card["candidate"]["identity"] != "tests/helper.rs::TestShape"
            && card["candidate"]["identity"] != "tests/helper.rs::parse_test"
    }));
    let suppressed = artifact["suppressedCues"]
        .as_array()
        .context("suppressed cues")?;
    for (identity, lane) in [
        ("tests/helper.rs::TestShape", "shape-hash"),
        ("tests/helper.rs::parse_test", "function-signature"),
    ] {
        let cue = suppressed
            .iter()
            .find(|cue| cue["candidate"]["identity"] == identity && cue["evidenceLane"] == lane)
            .with_context(|| format!("suppressed {lane} cue for {identity}"))?;
        assert_eq!(cue["reason"], "policy-excluded");
        assert_eq!(cue["pathClassifications"], serde_json::json!(["test"]));
    }
    Ok(())
}
