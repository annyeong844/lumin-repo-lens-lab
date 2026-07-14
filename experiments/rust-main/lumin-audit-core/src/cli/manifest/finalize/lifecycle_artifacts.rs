use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use lumin_audit_core::manifest_final::ManifestCloseoutCompanionInput;

pub(super) fn current_lifecycle_artifacts(
    output: &Path,
    manifest: &serde_json::Value,
    companion: &ManifestCloseoutCompanionInput,
    include_producer_performance: bool,
) -> Vec<String> {
    let mut artifacts = BTreeSet::new();

    for pointer in [
        "/preWrite/advisoryPath",
        "/preWrite/latestAdvisoryPath",
        "/preWrite/rustEvidencePath",
        "/preWrite/anyInventoryPath",
        "/preWrite/rustNativeArtifactPath",
        "/preWrite/rustNativeLatestPath",
        "/postWrite/deltaPath",
    ] {
        add_current_output_path(
            &mut artifacts,
            output,
            manifest
                .pointer(pointer)
                .and_then(serde_json::Value::as_str),
        );
    }

    let fresh_rust_evidence = manifest
        .pointer("/preWrite/rustEvidencePath")
        .and_then(serde_json::Value::as_str)
        .is_some();
    if fresh_rust_evidence {
        add_current_output_file(&mut artifacts, output, "pre-write-evidence.latest.json");
    }

    if manifest
        .pointer("/postWrite/ran")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        let pre_write_id = manifest
            .pointer("/postWrite/preWriteInvocationId")
            .and_then(serde_json::Value::as_str);
        let delta_id = manifest
            .pointer("/postWrite/deltaInvocationId")
            .and_then(serde_json::Value::as_str);
        if let (Some(pre_write_id), Some(delta_id)) = (pre_write_id, delta_id) {
            add_current_output_file(
                &mut artifacts,
                output,
                &format!("post-write-delta.{pre_write_id}.{delta_id}.json"),
            );
        }
    }

    if let Some(paths) = manifest
        .pointer("/canonDraft/draftPaths")
        .and_then(serde_json::Value::as_array)
    {
        for path in paths {
            add_current_output_path(&mut artifacts, output, path.as_str());
        }
    }

    if manifest
        .pointer("/checkCanon/ran")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        add_current_output_file(&mut artifacts, output, "canon-drift.json");
        if let Some(per_source) = manifest
            .pointer("/checkCanon/perSource")
            .and_then(serde_json::Value::as_object)
        {
            for entry in per_source.values() {
                add_current_output_path(
                    &mut artifacts,
                    output,
                    entry.get("reportPath").and_then(serde_json::Value::as_str),
                );
            }
        }
    }

    for path in [
        companion.topology_mermaid_path.as_deref(),
        companion.audit_summary_path.as_deref(),
        companion.review_pack_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        add_current_output_path(&mut artifacts, output, Some(path));
    }
    if include_producer_performance {
        add_current_output_file(&mut artifacts, output, "producer-performance.json");
    }

    artifacts.into_iter().collect()
}

fn add_current_output_file(artifacts: &mut BTreeSet<String>, output: &Path, name: &str) {
    add_current_output_path(artifacts, output, Some(name));
}

fn add_current_output_path(artifacts: &mut BTreeSet<String>, output: &Path, value: Option<&str>) {
    let Some(value) = value else {
        return;
    };
    let path = PathBuf::from(value);
    let path = if path.is_absolute() {
        path
    } else {
        output.join(path)
    };
    if !path.is_file() {
        return;
    }
    let Ok(relative) = path.strip_prefix(output) else {
        return;
    };
    if relative.components().count() != 1 {
        return;
    }
    artifacts.insert(relative.to_string_lossy().to_string());
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn lifecycle_artifact_scope_excludes_reused_base_outputs() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let output = temp.path();
        for name in [
            "symbols.json",
            "producer-performance.json",
            "pre-write-evidence.PRE.json",
            "pre-write-evidence.latest.json",
            "pre-write-advisory.latest.json",
            "pre-write-advisory.PRE.json",
            "any-inventory.pre.PRE.json",
        ] {
            std::fs::write(output.join(name), "{}")?;
        }
        let manifest = serde_json::json!({
            "preWrite": {
                "requested": true,
                "ran": true,
                "advisoryPath": output.join("pre-write-advisory.PRE.json"),
                "latestAdvisoryPath": output.join("pre-write-advisory.latest.json"),
                "advisoryInvocationId": "PRE",
                "rustEvidencePath": "pre-write-evidence.PRE.json",
                "anyInventoryPath": "any-inventory.pre.PRE.json"
            }
        });
        let artifacts = current_lifecycle_artifacts(
            output,
            &manifest,
            &ManifestCloseoutCompanionInput::default(),
            false,
        );
        assert_eq!(
            artifacts,
            vec![
                "any-inventory.pre.PRE.json",
                "pre-write-advisory.PRE.json",
                "pre-write-advisory.latest.json",
                "pre-write-evidence.PRE.json",
                "pre-write-evidence.latest.json",
            ]
        );
        Ok(())
    }

    #[test]
    fn lifecycle_artifact_scope_excludes_stale_pre_write_evidence() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let output = temp.path();
        for name in [
            "pre-write-advisory.latest.json",
            "pre-write-advisory.PRE.json",
            "pre-write-evidence.latest.json",
            "any-inventory.pre.PRE.json",
        ] {
            std::fs::write(output.join(name), "{}")?;
        }
        let manifest = serde_json::json!({
            "preWrite": {
                "requested": true,
                "ran": true,
                "advisoryPath": output.join("pre-write-advisory.PRE.json"),
                "latestAdvisoryPath": output.join("pre-write-advisory.latest.json"),
                "advisoryInvocationId": "PRE"
            }
        });

        let artifacts = current_lifecycle_artifacts(
            output,
            &manifest,
            &ManifestCloseoutCompanionInput::default(),
            false,
        );
        assert_eq!(
            artifacts,
            vec![
                "pre-write-advisory.PRE.json",
                "pre-write-advisory.latest.json",
            ]
        );
        Ok(())
    }
}
