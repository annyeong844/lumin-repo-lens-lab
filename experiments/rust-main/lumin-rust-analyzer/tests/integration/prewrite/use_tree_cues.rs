use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_public_reexport_alias_is_claim_only_safe_like_ts_js_export_alias() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "src/lib.rs",
        br#"mod model {
    pub struct Thing;
}

pub use model::Thing as PublicThing;
"#,
    )?;
    let artifact = repo.run_json(
        r#"{
  "names": ["PublicThing"],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    let lookup = lookup(&artifact, "PublicThing")?;
    assert_eq!(lookup["result"], "EXISTS");
    assert_eq!(
        lookup["identities"][0]["identity"],
        "src/lib.rs::PublicThing"
    );
    assert_eq!(lookup["identities"][0]["matchedField"], "useTreeIndex");
    assert_eq!(lookup["identities"][0]["visibility"], "public");
    assert!(lookup["citations"]
        .as_array()
        .context("re-export alias citations")?
        .iter()
        .any(|citation| citation
            .as_str()
            .is_some_and(|text| text.contains(".ast.useTrees contains 'PublicThing'"))));

    let card = card(&artifact, "src/lib.rs::PublicThing")?;
    assert_eq!(card["renderTier"], "SAFE_CUE");
    let cue = &card["cues"][0];
    assert_eq!(cue["cueTier"], "SAFE_CUE");
    assert_eq!(cue["safeMeaning"], "claim-only");
    assert_eq!(cue["evidenceLane"], "exact-symbol");
    assert_eq!(cue["claim"], "exact Rust use-tree name exists");
    assert_eq!(
        cue["notSafeFor"],
        serde_json::json!(["semantic-equivalence", "auto-reuse", "auto-fix"])
    );
    assert_eq!(cue["evidence"][0]["matchedField"], "files[].ast.useTrees[]");
    assert_eq!(
        cue["evidence"][0]["candidateIdentity"],
        "src/lib.rs::PublicThing"
    );
    Ok(())
}

#[test]
fn prewrite_use_tree_names_are_exact_only_and_ignore_anonymous_renames() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "src/lib.rs",
        br#"pub use external::Thing as PublicThing;
pub use hidden::Hidden as _;
pub use external::{self};
"#,
    )?;
    let artifact = repo.run_json(
        r#"{
  "names": ["PublicThingV2", "Hidden", "external"],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    let near_lookup = lookup(&artifact, "PublicThingV2")?;
    assert_eq!(near_lookup["result"], "NOT_OBSERVED");
    assert!(near_lookup["nearNames"]
        .as_array()
        .context("near names")?
        .iter()
        .all(|entry| entry["matchedField"] != "useTreeIndex"));
    assert!(near_lookup["semanticHints"]
        .as_array()
        .context("semantic hints")?
        .iter()
        .all(|entry| entry["matchedField"] != "useTreeIndex"));

    let anonymous = lookup(&artifact, "Hidden")?;
    assert_eq!(anonymous["result"], "NOT_OBSERVED");

    let self_export = lookup(&artifact, "external")?;
    assert_eq!(self_export["result"], "EXISTS");
    assert_eq!(self_export["identities"][0]["matchedField"], "useTreeIndex");
    let card = card(&artifact, "src/lib.rs::external")?;
    assert_eq!(card["renderTier"], "SAFE_CUE");
    assert_eq!(card["cues"][0]["claim"], "exact Rust use-tree name exists");
    Ok(())
}

fn card<'a>(artifact: &'a Value, identity: &str) -> Result<&'a Value> {
    artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .find(|card| card["candidate"]["identity"] == identity)
        .with_context(|| format!("cue card {identity}"))
}

fn lookup<'a>(artifact: &'a Value, name: &str) -> Result<&'a Value> {
    artifact["lookups"]
        .as_array()
        .context("lookups")?
        .iter()
        .find(|lookup| lookup["intentName"] == name)
        .with_context(|| format!("lookup {name}"))
}
