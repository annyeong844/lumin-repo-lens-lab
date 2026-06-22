use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_file_lane_reports_existing_new_and_unavailable_rust_files() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes("src/broken.rs", &[0xff])?;
    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": [
    "src/lib.rs",
    "src/new_module.rs",
    "src/broken.rs",
    "target/generated.rs",
    "README.md"
  ],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert_eq!(artifact["coverage"]["files"], "ran");
    let existing = file_lookup(&artifact, "src/lib.rs")?;
    assert_eq!(existing["kind"], "file");
    assert_eq!(existing["result"], "FILE_EXISTS");
    assert_eq!(existing["boundary"]["status"], "NOT_EVALUATED");
    assert!(citations(existing)
        .any(|citation| { citation.contains("rust-source-health.files['src/lib.rs'] present") }));
    let existing_card = cue_card(&artifact, "src/lib.rs::__file__")?;
    assert_eq!(existing_card["renderTier"], "SAFE_CUE");
    assert_eq!(existing_card["candidate"]["name"], "__file__");
    let exact_file_cue = existing_card["cues"]
        .as_array()
        .context("existing file cues")?
        .iter()
        .find(|cue| cue["evidenceLane"] == "exact-file")
        .context("exact file cue")?;
    assert_eq!(exact_file_cue["cueTier"], "SAFE_CUE");
    assert_eq!(exact_file_cue["safeMeaning"], "claim-only");
    assert_eq!(exact_file_cue["claim"], "exact file exists");
    assert_eq!(exact_file_cue["confidence"], "grounded");
    assert_eq!(
        exact_file_cue["notSafeFor"],
        serde_json::json!(["semantic-equivalence", "auto-reuse", "auto-fix"])
    );
    assert_eq!(
        exact_file_cue["evidence"][0]["artifact"],
        "rust-source-health"
    );
    assert_eq!(exact_file_cue["evidence"][0]["matchedField"], "files");
    assert_eq!(
        exact_file_cue["evidence"][0]["algorithmVersion"],
        "exact-file.v1"
    );
    assert_eq!(exact_file_cue["evidence"][0]["file"], "src/lib.rs");
    assert_eq!(
        exact_file_cue["evidence"][0]["fileLookupResult"],
        "FILE_EXISTS"
    );

    let new_file = file_lookup(&artifact, "src/new_module.rs")?;
    assert_eq!(new_file["result"], "NEW_FILE");
    assert!(citations(new_file).any(|citation| {
        citation.contains("rust-source-health.files does not contain 'src/new_module.rs'")
    }));

    let skipped = file_lookup(&artifact, "src/broken.rs")?;
    assert_eq!(skipped["result"], "FILE_STATUS_UNKNOWN");
    assert!(citations(skipped).any(|citation| citation.contains("invalid-utf8")));

    let excluded = file_lookup(&artifact, "target/generated.rs")?;
    assert_eq!(excluded["result"], "FILE_STATUS_UNKNOWN");
    assert!(citations(excluded).any(|citation| citation.contains("target/vendor excluded")));

    let non_rust = file_lookup(&artifact, "README.md")?;
    assert_eq!(non_rust["result"], "FILE_STATUS_UNKNOWN");
    assert!(citations(non_rust).any(|citation| citation.contains("Rust .rs files only")));
    assert!(cue_card(&artifact, "src/new_module.rs::__file__").is_err());
    Ok(())
}

#[test]
fn prewrite_file_lane_keeps_symlinked_rust_paths_unknown() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes("src/real.rs", b"pub fn real() {}\n")?;
    let target = repo.root_path().join("src").join("real.rs");
    let link = repo.root_path().join("src").join("linked.rs");
    if !create_file_symlink(&target, &link)? {
        return Ok(());
    }

    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": ["src/linked.rs"],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    let linked = file_lookup(&artifact, "src/linked.rs")?;
    assert_eq!(linked["result"], "FILE_STATUS_UNKNOWN");
    assert!(citations(linked).any(|citation| citation.contains("is a symlink")));
    assert!(cue_card(&artifact, "src/linked.rs::__file__").is_err());
    Ok(())
}

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

fn citations(lookup: &Value) -> impl Iterator<Item = &str> {
    lookup["citations"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

fn create_file_symlink(target: &Path, link: &Path) -> Result<bool> {
    match create_file_symlink_inner(target, link) {
        Ok(()) => Ok(true),
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::PermissionDenied | ErrorKind::Unsupported
            ) || error.raw_os_error() == Some(1314) =>
        {
            Ok(false)
        }
        Err(error) => Err(error).context("create symlinked Rust file fixture"),
    }
}

#[cfg(unix)]
fn create_file_symlink_inner(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_file_symlink_inner(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}
