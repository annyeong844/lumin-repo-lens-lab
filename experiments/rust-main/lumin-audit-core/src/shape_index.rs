use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;

pub const SHAPE_INDEX_REQUEST_SCHEMA_VERSION: &str = "lumin-shape-index-producer-request.v1";

const TOOL_NAME: &str = "build-shape-index.mjs";
const ARTIFACT_SCHEMA_VERSION: &str = "shape-index.v1";
const SHAPE_HASH_NORMALIZED_VERSION: &str = "shape-hash.normalized.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeIndexRequest {
    pub schema_version: String,
    pub generated: String,
    pub root: Value,
    #[serde(default)]
    pub include_tests: bool,
    #[serde(default)]
    pub exclude: Vec<Value>,
    pub scope: String,
    pub observed_at: String,
    #[serde(default)]
    pub file_count: usize,
    #[serde(default)]
    pub facts: Vec<Value>,
    #[serde(default)]
    pub diagnostics: Vec<Value>,
    #[serde(default)]
    pub files_with_parse_errors: Vec<Value>,
    #[serde(default)]
    pub files_with_read_errors: Vec<Value>,
    #[serde(default)]
    pub incremental: Option<Value>,
}

pub fn build_shape_index_artifact(request: ShapeIndexRequest) -> Result<Value> {
    if request.schema_version != SHAPE_INDEX_REQUEST_SCHEMA_VERSION {
        bail!(
            "shape-index-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let facts = stamped_and_sorted_facts(request.facts, &request.observed_at);
    let diagnostics = sorted_diagnostics(request.diagnostics);
    let parse_errors = sorted_file_errors(request.files_with_parse_errors);
    let read_errors = sorted_file_errors(request.files_with_read_errors);
    let groups_by_hash = groups_by_hash(&facts);
    let generated_file_fact_count = facts
        .iter()
        .filter(|fact| fact.get("generatedFile").is_some())
        .count();

    let mut meta = Map::new();
    meta.insert("tool".to_string(), json!(TOOL_NAME));
    meta.insert("generated".to_string(), json!(request.generated));
    meta.insert("root".to_string(), request.root);
    meta.insert("source".to_string(), json!("fresh-ast-pass"));
    meta.insert("scope".to_string(), json!(request.scope));
    meta.insert("observedAt".to_string(), json!(request.observed_at));
    meta.insert(
        "complete".to_string(),
        json!(read_errors.is_empty() && parse_errors.is_empty()),
    );
    meta.insert("includeTests".to_string(), json!(request.include_tests));
    meta.insert("exclude".to_string(), Value::Array(request.exclude));
    meta.insert("fileCount".to_string(), json!(request.file_count));
    meta.insert("factCount".to_string(), json!(facts.len()));
    meta.insert(
        "generatedFileFactCount".to_string(),
        json!(generated_file_fact_count),
    );
    meta.insert("hashGroupCount".to_string(), json!(groups_by_hash.len()));
    meta.insert("diagnosticCount".to_string(), json!(diagnostics.len()));
    meta.insert(
        "filesWithParseErrors".to_string(),
        Value::Array(parse_errors),
    );
    meta.insert("filesWithReadErrors".to_string(), Value::Array(read_errors));
    if let Some(incremental) = request.incremental {
        meta.insert("incremental".to_string(), incremental);
    }
    meta.insert(
        "supports".to_string(),
        json!({
            "shapeHash": true,
            "normalizedVersion": SHAPE_HASH_NORMALIZED_VERSION,
            "exportedInterfaces": true,
            "exportedObjectTypeAliases": true,
            "exportedUnionLiteralTypeAliases": true,
            "unsupportedShapesAsDiagnostics": true,
            "generatedFileEvidence": true,
        }),
    );

    Ok(json!({
        "schemaVersion": ARTIFACT_SCHEMA_VERSION,
        "meta": Value::Object(meta),
        "facts": facts,
        "groupsByHash": groups_by_hash,
        "diagnostics": diagnostics,
    }))
}

fn stamped_and_sorted_facts(facts: Vec<Value>, observed_at: &str) -> Vec<Value> {
    let mut facts = facts
        .into_iter()
        .map(|fact| stamp_observed_at(fact, observed_at))
        .collect::<Vec<_>>();
    facts.sort_by(compare_facts);
    facts
}

fn stamp_observed_at(fact: Value, observed_at: &str) -> Value {
    match fact {
        Value::Object(mut object) => {
            object.insert("observedAt".to_string(), json!(observed_at));
            Value::Object(object)
        }
        other => other,
    }
}

fn compare_facts(left: &Value, right: &Value) -> Ordering {
    value_str(left, "ownerFile")
        .cmp(value_str(right, "ownerFile"))
        .then_with(|| value_usize(left, "line").cmp(&value_usize(right, "line")))
        .then_with(|| value_str(left, "exportedName").cmp(value_str(right, "exportedName")))
}

fn sorted_diagnostics(mut diagnostics: Vec<Value>) -> Vec<Value> {
    diagnostics.sort_by(|left, right| {
        diagnostic_file(left)
            .cmp(diagnostic_file(right))
            .then_with(|| value_str(left, "exportedName").cmp(value_str(right, "exportedName")))
            .then_with(|| value_str(left, "code").cmp(value_str(right, "code")))
    });
    diagnostics
}

fn sorted_file_errors(mut errors: Vec<Value>) -> Vec<Value> {
    errors.sort_by(|left, right| {
        value_str(left, "file")
            .cmp(value_str(right, "file"))
            .then_with(|| value_str(left, "message").cmp(value_str(right, "message")))
    });
    errors
}

fn diagnostic_file(value: &Value) -> &str {
    let file = value_str(value, "file");
    if file.is_empty() {
        value_str(value, "ownerFile")
    } else {
        file
    }
}

fn groups_by_hash(facts: &[Value]) -> BTreeMap<String, Vec<String>> {
    let mut groups = BTreeMap::<String, Vec<String>>::new();
    for fact in facts {
        let Some(hash) = fact.get("hash").and_then(Value::as_str) else {
            continue;
        };
        let identities = fact
            .get("identities")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str);
        let group = groups.entry(hash.to_string()).or_default();
        group.extend(identities.map(str::to_string));
    }
    for identities in groups.values_mut() {
        identities.sort();
    }
    groups
}

fn value_str<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn value_usize(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn base_request(facts: Vec<Value>) -> ShapeIndexRequest {
        ShapeIndexRequest {
            schema_version: SHAPE_INDEX_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-04T00:00:00.000Z".to_string(),
            root: json!("C:/repo"),
            include_tests: false,
            exclude: vec![json!("node_modules")],
            scope: "TS/JS production files, exported types only".to_string(),
            observed_at: "2026-07-04T00:00:00.000Z".to_string(),
            file_count: 2,
            facts,
            diagnostics: vec![],
            files_with_parse_errors: vec![],
            files_with_read_errors: vec![],
            incremental: Some(json!({
                "enabled": false,
                "reason": "disabled-by-flag",
            })),
        }
    }

    #[test]
    fn builds_shape_index_projection_from_js_extracted_facts() -> Result<()> {
        let hash = format!("sha256:{}", "a".repeat(64));
        let artifact = build_shape_index_artifact(base_request(vec![
            json!({
                "kind": "shape-hash",
                "hash": hash,
                "identities": ["b.ts::B"],
                "identity": "b.ts::B",
                "exportedName": "B",
                "ownerFile": "b.ts",
                "line": 3,
            }),
            json!({
                "kind": "shape-hash",
                "hash": hash,
                "identities": ["a.ts::A"],
                "identity": "a.ts::A",
                "exportedName": "A",
                "ownerFile": "a.ts",
                "line": 1,
                "generatedFile": {
                    "kind": "generated-file",
                    "source": "path",
                    "evidence": "path:generated-suffix"
                },
            }),
        ]))?;

        assert_eq!(artifact["schemaVersion"], "shape-index.v1");
        assert_eq!(artifact["meta"]["tool"], "build-shape-index.mjs");
        assert_eq!(artifact["meta"]["factCount"], 2);
        assert_eq!(artifact["meta"]["generatedFileFactCount"], 1);
        assert_eq!(
            artifact["meta"]["supports"]["normalizedVersion"],
            "shape-hash.normalized.v1"
        );
        assert_eq!(artifact["facts"][0]["identity"], "a.ts::A");
        assert_eq!(artifact["facts"][1]["identity"], "b.ts::B");
        assert_eq!(
            artifact["groupsByHash"][hash.as_str()],
            json!(["a.ts::A", "b.ts::B"])
        );
        Ok(())
    }

    #[test]
    fn rejects_unknown_request_schema() {
        let mut request = base_request(vec![]);
        request.schema_version = "shape-index.future".to_string();
        let error = match build_shape_index_artifact(request) {
            Ok(_) => panic!("schema should reject"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("unsupported schemaVersion"));
    }
}
