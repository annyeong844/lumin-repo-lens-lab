use anyhow::{Context, Result};

use super::support::{cue_card, dependency_lookup, run_dependency_fixture};

#[test]
fn prewrite_dependency_lane_promotes_grounded_dependency_hub_cue() -> Result<()> {
    let artifact = run_dependency_fixture()?;
    let anyhow = dependency_lookup(&artifact, "anyhow")?;
    let anyhow_observed_count = anyhow["existingImports"]["observedImportCount"]
        .as_u64()
        .context("anyhow observed import count")?;
    assert!(anyhow_observed_count >= 10);

    let hub_card = cue_card(&artifact, "Cargo.toml::dependency::anyhow")?;
    assert_eq!(hub_card["renderTier"], "AGENT_REVIEW_CUE");
    assert_eq!(hub_card["candidate"]["ownerFile"], "Cargo.toml");
    assert_eq!(hub_card["candidate"]["name"], "anyhow");
    let hub_cue = hub_card["cues"]
        .as_array()
        .context("dependency hub cues")?
        .iter()
        .find(|cue| cue["evidenceLane"] == "dependency-hub")
        .context("dependency hub cue")?;
    assert_eq!(hub_cue["cueTier"], "AGENT_REVIEW_CUE");
    assert_eq!(hub_cue["claim"], "Rust dependency hub");
    assert_eq!(hub_cue["confidence"], "grounded");
    assert_eq!(
        hub_cue["evidence"][0]["matchedField"],
        "dependencyLookups[].existingImports"
    );
    assert_eq!(
        hub_cue["evidence"][0]["dependencyLookupResult"],
        anyhow["result"]
    );
    assert_eq!(
        hub_cue["evidence"][0]["observedImportCount"],
        anyhow_observed_count
    );
    assert_eq!(hub_cue["evidence"][0]["consumerThreshold"], 10);
    Ok(())
}
