use anyhow::{Context, Result};

use crate::support::prewrite::PreWriteRepo;

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
    assert_eq!(shape_lookups[1]["result"], "NOT_OBSERVED");
    assert!(shape_lookups[1]["citations"]
        .as_array()
        .context("hash shape citations")?
        .iter()
        .any(|citation| citation
            .as_str()
            .is_some_and(|text| text.contains("complete rust-source-health"))));

    let unavailable = artifact["unavailableEvidence"]
        .as_array()
        .context("unavailable evidence")?;
    assert_eq!(unavailable.len(), 1);
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
