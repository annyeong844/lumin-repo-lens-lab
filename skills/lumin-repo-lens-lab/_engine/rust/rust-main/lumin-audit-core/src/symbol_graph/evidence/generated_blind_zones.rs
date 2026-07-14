use super::ordering::value_string;
use super::paths::normalize_path_segments;
use crate::scan_scope::{scan_scope_status_for_path, ScanScopeOptions};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use std::path::Path;

pub(in crate::symbol_graph) fn build_generated_consumer_blind_zones(
    root: &str,
    unresolved_records: &[Value],
    include_tests: bool,
    exclude: &[String],
    mode: &str,
) -> Vec<Value> {
    let root_path = Path::new(root);
    let mut zones = Vec::new();
    let mut seen = BTreeSet::new();

    let scan_options = ScanScopeOptions {
        include_tests,
        exclude: exclude.to_vec(),
        ..ScanScopeOptions::default()
    };

    for record in unresolved_records {
        if !is_generated_artifact_missing_record(record) {
            continue;
        }
        let Some(artifact) = record.get("generatedArtifact").and_then(Value::as_object) else {
            continue;
        };
        for candidate in target_candidates(record) {
            let Some(candidate_path) = generated_candidate_repo_relative(root_path, &candidate)
            else {
                continue;
            };
            let Some(scope_package_root) =
                consumer_zone_scope_root(record, artifact, &candidate_path)
            else {
                continue;
            };

            let abs_candidate = root_path.join(&candidate_path);
            let mut status = "missing";
            let mut scan_scope_reason = None;
            if abs_candidate.exists() {
                let scope = scan_scope_status_for_path(root_path, &abs_candidate, &scan_options);
                if scope.included {
                    continue;
                }
                status = "present-but-out-of-scope";
                scan_scope_reason = scope.reason.or(Some("excluded"));
            }

            let mut object = Map::new();
            object.insert("reason".to_string(), json!("generated-consumer-blind-zone"));
            object.insert(
                "sourceReason".to_string(),
                json!(value_string(record, "reason")),
            );
            object.insert(
                "specifier".to_string(),
                json!(nullable_string(record, "specifier")),
            );
            object.insert(
                "consumerFile".to_string(),
                json!(nullable_string(record, "consumerFile")
                    .or_else(|| nullable_string(record, "fromHint"))),
            );
            object.insert(
                "matchedPackage".to_string(),
                json!(nullable_string_from_map(artifact, "matchedPackage")),
            );
            object.insert(
                "targetSubpath".to_string(),
                json!(nullable_string_from_map(artifact, "targetSubpath")),
            );
            object.insert(
                "generatorFamily".to_string(),
                json!(nullable_string_from_map(artifact, "generatorFamily")),
            );
            object.insert(
                "confidence".to_string(),
                json!(nullable_string_from_map(artifact, "confidence")),
            );
            object.insert("candidatePath".to_string(), json!(candidate_path));
            object.insert("status".to_string(), json!(status));
            object.insert("scopePackageRoot".to_string(), json!(scope_package_root));
            object.insert("mode".to_string(), json!(mode));
            if let Some(reason) = scan_scope_reason {
                object.insert("scanScopeReason".to_string(), json!(reason));
            }
            if mode == "prepared" {
                object.insert("staleStatus".to_string(), json!("unknown"));
                object.insert(
                    "staleReason".to_string(),
                    json!("generator-input-hash-not-recorded"),
                );
            }
            let zone = Value::Object(object);
            let key = generated_consumer_zone_dedupe_key(&zone);
            if seen.insert(key) {
                zones.push(zone);
            }
        }
    }

    zones
}

fn is_generated_artifact_missing_record(record: &Value) -> bool {
    value_string(record, "reason") == "workspace-generated-artifact-missing"
        && record
            .get("generatedArtifact")
            .is_some_and(Value::is_object)
}

fn target_candidates(record: &Value) -> Vec<String> {
    record
        .get("targetCandidates")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn generated_candidate_repo_relative(root: &Path, candidate: &str) -> Option<String> {
    let root_text = normalize_path_segments(&root.to_string_lossy());
    let root_text = root_text.trim_end_matches('/');
    let candidate_path = Path::new(candidate);
    let candidate_text = if candidate_path.is_absolute() {
        normalize_path_segments(&candidate_path.to_string_lossy())
    } else {
        normalize_path_segments(&format!("{root_text}/{candidate}"))
    };
    let prefix = format!("{root_text}/");
    candidate_text
        .strip_prefix(&prefix)
        .filter(|relative| !relative.is_empty() && *relative != "..")
        .filter(|relative| !relative.starts_with("../"))
        .map(ToString::to_string)
}

fn generated_package_root(artifact: &Map<String, Value>) -> Option<String> {
    nullable_string_from_map(artifact, "packageRoot")
        .or_else(|| nullable_string_from_map(artifact, "packageDir"))
        .or_else(|| nullable_string_from_map(artifact, "workspaceRoot"))
}

fn package_root_from_candidate(candidate_path: &str) -> Option<String> {
    let parts = candidate_path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if matches!(parts.first(), Some(&"apps" | &"packages")) && parts.len() >= 2 {
        return Some(format!("{}/{}", parts[0], parts[1]));
    }
    None
}

fn consumer_zone_scope_root(
    _record: &Value,
    artifact: &Map<String, Value>,
    candidate_path: &str,
) -> Option<String> {
    generated_package_root(artifact).or_else(|| package_root_from_candidate(candidate_path))
}

fn nullable_string(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn nullable_string_from_map(object: &Map<String, Value>, field: &str) -> Option<String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn generated_consumer_zone_dedupe_key(zone: &Value) -> String {
    [
        value_string(zone, "specifier"),
        value_string(zone, "consumerFile"),
        value_string(zone, "candidatePath"),
        value_string(zone, "mode"),
    ]
    .join("|")
}
