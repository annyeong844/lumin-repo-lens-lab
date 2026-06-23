use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_file_lane_reports_domain_cluster_watch_cues_like_js_ts() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes("src/user_loader.rs", b"pub fn load_user() {}\n")?;
    repo.write_bytes("src/user_store.rs", b"pub fn store_user() {}\n")?;
    repo.write_bytes("src/other.rs", b"pub fn other() {}\n")?;
    repo.write_bytes("src/artifacts.rs", b"pub fn artifacts() {}\n")?;
    repo.write_bytes(
        "src/post_write_artifact.rs",
        b"pub fn post_write_artifact() {}\n",
    )?;
    repo.write_bytes(
        "src/pre_write_artifact.rs",
        b"pub fn pre_write_artifact() {}\n",
    )?;
    repo.write_bytes(
        "src/shape_index_artifact.rs",
        b"pub fn shape_index_artifact() {}\n",
    )?;
    repo.write_bytes(
        "src/merge_with_values.rs",
        b"pub fn merge_with_values() {}\n",
    )?;
    repo.write_bytes("src/deep_merge.rs", b"pub fn deep_merge() {}\n")?;

    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": [
    "src/user_service.rs",
    "src/artifact_loader.rs",
    "src/merge_with_defaults.rs",
    "src/other_new.rs"
  ],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    let user_service = domain_cluster(file_lookup(&artifact, "src/user_service.rs")?)?;
    assert_eq!(user_service["kind"], "DOMAIN_CLUSTER_DETECTED");
    assert_eq!(user_service["directory"], "src");
    assert_eq!(user_service["basenamePrefix"], "user");
    assert_eq!(user_service["matchKind"], "prefix");
    assert_eq!(user_service["prefixPath"], "src/user");
    assert_eq!(user_service["matchCount"], 2);
    assert_eq!(user_service["totalLoc"], Value::Null);
    assert_eq!(
        example_files(user_service),
        vec!["src/user_loader.rs", "src/user_store.rs"]
    );
    let user_service_cue = cue_card(&artifact, "src/user_service.rs::__file__")?;
    assert_eq!(user_service_cue["renderTier"], "AGENT_REVIEW_CUE");
    let domain_cue = user_service_cue["cues"]
        .as_array()
        .context("file domain cluster cues")?
        .iter()
        .find(|cue| cue["evidenceLane"] == "file-domain-cluster")
        .context("file domain cluster cue")?;
    assert_eq!(domain_cue["cueTier"], "AGENT_REVIEW_CUE");
    assert_eq!(domain_cue["claim"], "related Rust file domain cluster");
    assert_eq!(
        domain_cue["evidence"][0]["matchedField"],
        "fileLookups[].domainCluster"
    );
    assert_eq!(domain_cue["evidence"][0]["file"], "src/user_service.rs");
    assert_eq!(domain_cue["evidence"][0]["fileLookupResult"], "NEW_FILE");

    let artifact_loader = domain_cluster(file_lookup(&artifact, "src/artifact_loader.rs")?)?;
    assert_eq!(artifact_loader["kind"], "DOMAIN_CLUSTER_DETECTED");
    assert_eq!(artifact_loader["basenamePrefix"], "artifact");
    assert_eq!(artifact_loader["matchKind"], "domain-token");
    assert_eq!(artifact_loader["prefixPath"], "src/artifact");
    assert_eq!(artifact_loader["matchCount"], 4);
    let artifact_examples = example_files(artifact_loader);
    assert!(artifact_examples.contains(&"src/artifacts.rs"));
    assert!(artifact_examples.contains(&"src/post_write_artifact.rs"));

    let merge_defaults = domain_cluster(file_lookup(&artifact, "src/merge_with_defaults.rs")?)?;
    assert_eq!(merge_defaults["kind"], "DOMAIN_CLUSTER_DETECTED");
    assert_eq!(merge_defaults["basenamePrefix"], "mergeWith");
    assert_eq!(merge_defaults["matchKind"], "prefix");
    assert_eq!(merge_defaults["prefixPath"], "src/mergeWith");
    assert_eq!(merge_defaults["matchCount"], 1);
    assert_eq!(
        example_files(merge_defaults),
        vec!["src/merge_with_values.rs"]
    );

    let unrelated = file_lookup(&artifact, "src/other_new.rs")?;
    assert_eq!(unrelated["result"], "NEW_FILE");
    assert_eq!(unrelated["domainCluster"], Value::Null);
    assert!(cue_card(&artifact, "src/other_new.rs::__file__").is_err());
    Ok(())
}

fn file_lookup<'a>(artifact: &'a Value, intent_file: &str) -> Result<&'a Value> {
    artifact["fileLookups"]
        .as_array()
        .context("fileLookups array")?
        .iter()
        .find(|lookup| lookup["intentFile"] == intent_file)
        .with_context(|| format!("lookup for {intent_file}"))
}

fn domain_cluster(lookup: &Value) -> Result<&Value> {
    let cluster = &lookup["domainCluster"];
    if cluster.is_object() {
        Ok(cluster)
    } else {
        anyhow::bail!("domainCluster missing for {}", lookup["intentFile"]);
    }
}

fn cue_card<'a>(artifact: &'a Value, identity: &str) -> Result<&'a Value> {
    artifact["cueCards"]
        .as_array()
        .context("cueCards array")?
        .iter()
        .find(|card| card["candidate"]["identity"] == identity)
        .with_context(|| format!("cue card {identity}"))
}

fn example_files(cluster: &Value) -> Vec<&str> {
    cluster["examples"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry["file"].as_str())
        .collect()
}
