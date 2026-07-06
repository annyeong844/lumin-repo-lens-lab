use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Number, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub const COMPARE_REPOS_REQUEST_SCHEMA_VERSION: &str = "lumin-compare-repos-producer-request.v1";

const TOOL_NAME: &str = "compare-repos.mjs";
const MISSING_ARTIFACT_NOTE: &str = "Deltas involving an artifact missing from either side will be null. Run the full pipeline on both sides for complete comparison.";
const KNOWN_ARTIFACTS: &[&str] = &[
    "triage.json",
    "topology.json",
    "discipline.json",
    "symbols.json",
    "dead-classify.json",
    "runtime-evidence.json",
    "staleness.json",
    "fix-plan.json",
    "call-graph.json",
    "barrels.json",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareReposRequest {
    pub schema_version: String,
    pub generated: String,
    pub left: PathBuf,
    pub right: PathBuf,
    pub left_label: String,
    pub right_label: String,
}

pub fn build_compare_repos_artifact(request: CompareReposRequest) -> Result<Value> {
    if request.schema_version != COMPARE_REPOS_REQUEST_SCHEMA_VERSION {
        bail!(
            "compare-repos-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let left = summarize_side(&request.left, &request.left_label);
    let right = summarize_side(&request.right, &request.right_label);
    let deltas = build_deltas(&left, &right);

    Ok(json!({
        "meta": {
            "generated": request.generated,
            "tool": TOOL_NAME,
            "left": path_string(&request.left),
            "right": path_string(&request.right),
        },
        "left": left.value,
        "right": right.value,
        "deltas": deltas,
        "missingArtifacts": {
            "left": missing_artifacts(&left.artifacts_found),
            "right": missing_artifacts(&right.artifacts_found),
            "note": MISSING_ARTIFACT_NOTE,
        },
    }))
}

struct SideSummary {
    value: Value,
    summaries: Map<String, Value>,
    artifacts_found: Vec<String>,
}

fn summarize_side(dir: &Path, label: &str) -> SideSummary {
    let mut summaries = Map::new();
    let mut artifacts_found = Vec::<String>::new();

    if let Some(triage) = load_if_exists(dir, "triage.json") {
        artifacts_found.push("triage.json".to_string());
        summaries.insert(
            "triage".to_string(),
            json!({
                "files": coalesce(&[
                    get_path(&triage, &["summary", "files"]),
                    get_path(&triage, &["files"]),
                ]),
                "loc": coalesce(&[
                    get_path(&triage, &["summary", "loc"]),
                    get_path(&triage, &["loc"]),
                ]),
                "buildSystem": coalesce(&[
                    get_path(&triage, &["buildSystem"]),
                    get_path(&triage, &["summary", "buildSystem"]),
                ]),
            }),
        );
    }

    if let Some(topology) = load_if_exists(dir, "topology.json") {
        artifacts_found.push("topology.json".to_string());
        let summary = get_path(&topology, &["summary"]).unwrap_or(&topology);
        summaries.insert(
            "topology".to_string(),
            json!({
                "files": value_or_null(get_path(summary, &["files"])),
                "edges": value_or_null(get_path(summary, &["edges"])),
                "sccCount": value_or_null(get_path(summary, &["sccCount"])),
                "typeOnlyEdges": value_or_null(get_path(summary, &["typeOnlyEdges"])),
            }),
        );
    }

    if let Some(symbols) = load_if_exists(dir, "symbols.json") {
        artifacts_found.push("symbols.json".to_string());
        summaries.insert(
            "symbols".to_string(),
            json!({
                "files": value_or_null(get_path(&symbols, &["files"])),
                "totalDefs": value_or_null(get_path(&symbols, &["totalDefs"])),
                "deadInProd": value_or_null(get_path(&symbols, &["deadInProd"])),
                "resolvedInternal": value_or_null(get_path(&symbols, &["uses", "resolvedInternal"])),
                "external": value_or_null(get_path(&symbols, &["uses", "external"])),
                "unresolvedInternal": value_or_null(get_path(&symbols, &["uses", "unresolvedInternal"])),
                "unresolvedInternalRatio": value_or_null(get_path(&symbols, &["uses", "unresolvedInternalRatio"])),
            }),
        );
    }

    if let Some(dead_classify) = load_if_exists(dir, "dead-classify.json") {
        artifacts_found.push("dead-classify.json".to_string());
        summaries.insert(
            "deadClassify".to_string(),
            json!({
                "categoryC": value_or_null(get_path(&dead_classify, &["summary", "category_C"])),
                "categoryA": value_or_null(get_path(&dead_classify, &["summary", "category_A"])),
                "categoryB": value_or_null(get_path(&dead_classify, &["summary", "category_B"])),
                "excluded": value_or_null(get_path(&dead_classify, &["summary", "excluded"])),
            }),
        );
    }

    if let Some(fix_plan) = load_if_exists(dir, "fix-plan.json") {
        artifacts_found.push("fix-plan.json".to_string());
        summaries.insert(
            "fixPlan".to_string(),
            json!({
                "SAFE_FIX": value_or_null(get_path(&fix_plan, &["summary", "SAFE_FIX"])),
                "REVIEW_FIX": value_or_null(get_path(&fix_plan, &["summary", "REVIEW_FIX"])),
                "DEGRADED": value_or_null(get_path(&fix_plan, &["summary", "DEGRADED"])),
                "MUTED": value_or_null(get_path(&fix_plan, &["summary", "MUTED"])),
                "total": value_or_null(get_path(&fix_plan, &["summary", "total"])),
                "resolverBlindnessGate": value_or_null(get_path(&fix_plan, &["meta", "resolverBlindness", "gate"])),
            }),
        );
    }

    for name in [
        "runtime-evidence.json",
        "staleness.json",
        "discipline.json",
        "call-graph.json",
        "barrels.json",
    ] {
        if dir.join(name).exists() {
            artifacts_found.push(name.to_string());
        }
    }

    let value = json!({
        "label": label,
        "artifactsFound": artifacts_found,
        "summaries": summaries,
    });

    SideSummary {
        value,
        summaries,
        artifacts_found,
    }
}

fn build_deltas(left: &SideSummary, right: &SideSummary) -> Value {
    json!({
        "files": delta(summary_field(left, "triage", "files"), summary_field(right, "triage", "files")),
        "loc": delta(summary_field(left, "triage", "loc"), summary_field(right, "triage", "loc")),
        "totalDefs": delta(summary_field(left, "symbols", "totalDefs"), summary_field(right, "symbols", "totalDefs")),
        "deadInProd": delta(summary_field(left, "symbols", "deadInProd"), summary_field(right, "symbols", "deadInProd")),
        "runtimeSccs": delta(summary_field(left, "topology", "sccCount"), summary_field(right, "topology", "sccCount")),
        "typeOnlyEdges": delta(summary_field(left, "topology", "typeOnlyEdges"), summary_field(right, "topology", "typeOnlyEdges")),
        "safeFixes": delta(summary_field(left, "fixPlan", "SAFE_FIX"), summary_field(right, "fixPlan", "SAFE_FIX")),
        "reviewFixes": delta(summary_field(left, "fixPlan", "REVIEW_FIX"), summary_field(right, "fixPlan", "REVIEW_FIX")),
        "degraded": delta(summary_field(left, "fixPlan", "DEGRADED"), summary_field(right, "fixPlan", "DEGRADED")),
        "muted": delta(summary_field(left, "fixPlan", "MUTED"), summary_field(right, "fixPlan", "MUTED")),
        "unresolvedInternalRatio": delta(summary_field(left, "symbols", "unresolvedInternalRatio"), summary_field(right, "symbols", "unresolvedInternalRatio")),
    })
}

fn summary_field<'a>(side: &'a SideSummary, summary: &str, field: &str) -> Option<&'a Value> {
    side.summaries
        .get(summary)
        .and_then(|value| get_path(value, &[field]))
}

fn delta(left: Option<&Value>, right: Option<&Value>) -> Value {
    let (Some(left), Some(right)) = (left, right) else {
        return Value::Null;
    };
    if !left.is_number() || !right.is_number() {
        return Value::Null;
    }

    if let (Some(left_int), Some(right_int)) = (left.as_i64(), right.as_i64()) {
        return json!(right_int - left_int);
    }
    if let (Some(left_uint), Some(right_uint)) = (left.as_u64(), right.as_u64()) {
        if right_uint >= left_uint {
            return json!(right_uint - left_uint);
        }
        let Some(left_int) = i128::from(left_uint).checked_neg() else {
            return Value::Null;
        };
        let diff = i128::from(right_uint) + left_int;
        return json!(diff);
    }
    match (left.as_f64(), right.as_f64()) {
        (Some(left_float), Some(right_float)) => Number::from_f64(right_float - left_float)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        _ => Value::Null,
    }
}

fn missing_artifacts(found: &[String]) -> Vec<String> {
    KNOWN_ARTIFACTS
        .iter()
        .filter(|artifact| !found.iter().any(|found| found == **artifact))
        .map(|artifact| (*artifact).to_string())
        .collect()
}

fn load_if_exists(dir: &Path, name: &str) -> Option<Value> {
    let path = dir.join(name);
    if !path.exists() {
        return None;
    }
    match fs::read_to_string(&path) {
        Ok(text) => {
            let text = text.trim_start_matches('\u{feff}');
            match serde_json::from_str::<Value>(text) {
                Ok(value) => Some(value),
                Err(error) => {
                    let path_display = path.display();
                    eprintln!("[compare] failed to parse {path_display}: {error}");
                    None
                }
            }
        }
        Err(error) => {
            let path_display = path.display();
            eprintln!("[compare] failed to parse {path_display}: {error}");
            None
        }
    }
}

fn get_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn value_or_null(value: Option<&Value>) -> Value {
    value.cloned().unwrap_or(Value::Null)
}

fn coalesce(values: &[Option<&Value>]) -> Value {
    values
        .iter()
        .find_map(|value| value.as_ref().copied())
        .cloned()
        .unwrap_or(Value::Null)
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
