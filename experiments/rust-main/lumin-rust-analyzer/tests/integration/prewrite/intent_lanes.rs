use anyhow::{Context, Result};

use crate::support::prewrite::PreWriteRepo;

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
fn prewrite_refactor_sources_run_inline_pattern_lookup_without_match() -> Result<()> {
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

    assert_eq!(artifact["coverage"]["inlinePatterns"], "ran");
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
    assert_eq!(inline_lookups[0]["result"], "NO_INLINE_PATTERN_MATCH");
    assert_eq!(inline_lookups[0]["groups"], serde_json::json!([]));

    let unavailable = artifact["unavailableEvidence"]
        .as_array()
        .context("unavailable evidence")?;
    assert!(unavailable
        .iter()
        .all(|entry| entry["evidenceLane"] != "inline-extraction"));
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
