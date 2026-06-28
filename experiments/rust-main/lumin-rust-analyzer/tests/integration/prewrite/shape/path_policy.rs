use anyhow::{Context, Result};

use crate::support::prewrite::PreWriteRepo;

use super::support::{shape_hash, signature_hash, source_health};

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
    let health = source_health(&repo)?;
    let shape_hash = shape_hash(&health, "tests/helper.rs", "TestShape")?;
    let signature_hash = signature_hash(&health, "tests/helper.rs", "parse_test")?;
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
