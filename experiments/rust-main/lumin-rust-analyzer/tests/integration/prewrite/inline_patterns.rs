use anyhow::{Context, Result};

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_inline_patterns_emit_review_cue_for_repeated_statement_blocks() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "src/lib.rs",
        br#"pub struct Worker;

impl Worker {
    pub fn first(&self) {
        self.cleanup();
        self.close();
    }

    pub fn second(&self) {
        self.cleanup();
        self.close();
    }

    pub fn third(&self) {
        self.cleanup();
        self.close();
    }
}
"#,
    )?;

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
      "why": "extract repeated cleanup block"
    }
  ]
}"#,
    )?;

    assert_eq!(artifact["coverage"]["inlinePatterns"], "ran");
    let lookup = artifact["inlinePatternLookups"]
        .as_array()
        .and_then(|lookups| lookups.first())
        .context("inline pattern lookup")?;
    assert_eq!(lookup["result"], "INLINE_PATTERN_MATCH");
    assert_eq!(lookup["groups"][0]["kind"], "statement-sequence");
    assert_eq!(lookup["groups"][0]["size"], 3);
    assert_eq!(
        lookup["groups"][0]["normalizerVersion"],
        "rust-inline-statement-normalizer-v1"
    );

    let inline_cue = artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .flat_map(|card| card["cues"].as_array().into_iter().flatten())
        .find(|cue| cue["evidenceLane"] == "inline-extraction")
        .context("inline extraction cue")?;
    assert_eq!(inline_cue["cueTier"], "AGENT_REVIEW_CUE");
    assert_eq!(inline_cue["claim"], "repeated inline statement pattern");
    assert_eq!(
        inline_cue["notSafeFor"],
        serde_json::json!(["semantic-equivalence", "auto-reuse", "auto-fix"])
    );
    assert_eq!(
        inline_cue["evidence"][0]["matchedField"],
        "files[].ast.inlinePatterns[].patternHash"
    );
    Ok(())
}

#[test]
fn prewrite_inline_patterns_degrade_when_refactor_source_is_not_parsed() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes("src/lib.rs", b"pub fn broken( {\n")?;

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
      "lines": [1],
      "why": "extract repeated cleanup block"
    }
  ]
}"#,
    )?;

    assert_eq!(artifact["coverage"]["inlinePatterns"], "ran");
    assert_eq!(artifact["inlinePatternLookups"][0]["result"], "UNAVAILABLE");
    assert_eq!(
        artifact["inlinePatternLookups"][0]["reason"],
        "source-unavailable"
    );
    assert!(artifact["unavailableEvidence"]
        .as_array()
        .context("unavailable evidence")?
        .iter()
        .any(|entry| {
            entry["evidenceLane"] == "inline-extraction"
                && entry["reason"] == "source-unavailable"
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
            .all(|cue| cue["evidenceLane"] != "inline-extraction")));
    Ok(())
}
