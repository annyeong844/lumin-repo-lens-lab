use anyhow::{bail, Result as AnyResult};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const ARTIFACT_NAME: &str = "rust-analyzer-health.latest.json";

#[derive(Debug, Clone, PartialEq)]
pub struct RustAnalysisSummary {
    pub artifact: &'static str,
    pub status: RustAnalysisStatus,
    pub available: bool,
    pub root: Option<String>,
    pub schema_version: Option<String>,
    pub policy_version: Option<String>,
    pub producer: Option<String>,
    pub mode: Option<String>,
    pub source_health_profile: Option<Value>,
    pub semantic_mode: Option<Value>,
    pub scan_scope: Option<ScanScope>,
    pub files: u64,
    pub syntax_review_signals: u64,
    pub syntax_review_opaque_surfaces: u64,
    pub syntax_function_clone_exact_body_groups: u64,
    pub syntax_function_clone_structure_groups: u64,
    pub syntax_function_clone_signature_groups: u64,
    pub syntax_function_clone_near_candidates: u64,
    pub action_tier_summary: Option<Value>,
    pub oracle_bridge_status: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustAnalysisStatus {
    RootMismatch,
    InvalidShape,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanScope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_tests: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_policy: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustAnalysisRunMergeInput {
    #[serde(default)]
    pub evidence: Option<Value>,
    #[serde(default)]
    pub run: RustAnalysisRunObservation,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustAnalysisRunObservation {
    pub requested: bool,
    pub ran: bool,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analyzer_invocation: Option<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for RustAnalysisRunObservation {
    fn default() -> Self {
        Self {
            requested: false,
            ran: false,
            status: "not-requested".to_string(),
            rust_files: None,
            reason: None,
            artifact: None,
            path: None,
            source_commit: None,
            producer: None,
            analyzer_invocation: None,
            extra: BTreeMap::new(),
        }
    }
}

impl Serialize for RustAnalysisSummary {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.status {
            RustAnalysisStatus::RootMismatch | RustAnalysisStatus::InvalidShape => {
                let mut out = serializer.serialize_struct("RustAnalysisSummary", 4)?;
                out.serialize_field("artifact", &self.artifact)?;
                out.serialize_field("status", &self.status)?;
                out.serialize_field("available", &self.available)?;
                out.serialize_field("root", &self.root)?;
                out.end()
            }
            RustAnalysisStatus::Complete => {
                let field_count = if self.scan_scope.is_some() { 19 } else { 18 };
                let mut out = serializer.serialize_struct("RustAnalysisSummary", field_count)?;
                out.serialize_field("artifact", &self.artifact)?;
                out.serialize_field("status", &self.status)?;
                out.serialize_field("available", &self.available)?;
                out.serialize_field("schemaVersion", &self.schema_version)?;
                out.serialize_field("policyVersion", &self.policy_version)?;
                out.serialize_field("producer", &self.producer)?;
                out.serialize_field("mode", &self.mode)?;
                out.serialize_field("sourceHealthProfile", &self.source_health_profile)?;
                out.serialize_field("semanticMode", &self.semantic_mode)?;
                if let Some(scan_scope) = &self.scan_scope {
                    out.serialize_field("scanScope", scan_scope)?;
                }
                out.serialize_field("files", &self.files)?;
                out.serialize_field("syntaxReviewSignals", &self.syntax_review_signals)?;
                out.serialize_field(
                    "syntaxReviewOpaqueSurfaces",
                    &self.syntax_review_opaque_surfaces,
                )?;
                out.serialize_field(
                    "syntaxFunctionCloneExactBodyGroups",
                    &self.syntax_function_clone_exact_body_groups,
                )?;
                out.serialize_field(
                    "syntaxFunctionCloneStructureGroups",
                    &self.syntax_function_clone_structure_groups,
                )?;
                out.serialize_field(
                    "syntaxFunctionCloneSignatureGroups",
                    &self.syntax_function_clone_signature_groups,
                )?;
                out.serialize_field(
                    "syntaxFunctionCloneNearCandidates",
                    &self.syntax_function_clone_near_candidates,
                )?;
                out.serialize_field("actionTierSummary", &self.action_tier_summary)?;
                out.serialize_field("oracleBridgeStatus", &self.oracle_bridge_status)?;
                out.end()
            }
        }
    }
}

pub fn merge_rust_analysis_run(input: RustAnalysisRunMergeInput) -> AnyResult<Value> {
    validate_rust_analysis_run(&input.run)?;
    if input.run.requested {
        if input.run.ran {
            if status_field(input.evidence.as_ref()) != Some("complete") {
                let mut output = run_object(input.run)?;
                output.insert(
                    "status".to_string(),
                    Value::String("artifact-unavailable".to_string()),
                );
                output.insert("available".to_string(), Value::Bool(false));
                output.insert(
                    "artifactStatus".to_string(),
                    Value::String(
                        status_field(input.evidence.as_ref())
                            .unwrap_or("missing")
                            .to_string(),
                    ),
                );
                if let Some(artifact) = string_field_from_value(input.evidence.as_ref(), "artifact")
                {
                    output.insert("artifact".to_string(), Value::String(artifact.to_string()));
                }
                return Ok(Value::Object(output));
            }

            let mut output = object_field_map(input.evidence.as_ref());
            output.extend(run_object(input.run)?);
            output.insert("status".to_string(), Value::String("complete".to_string()));
            output.insert("available".to_string(), Value::Bool(true));
            return Ok(Value::Object(output));
        }
        return serde_json::to_value(input.run).map_err(Into::into);
    }

    let mut output = serde_json::Map::new();
    output.insert("requested".to_string(), Value::Bool(false));
    output.insert("ran".to_string(), Value::Bool(false));
    output.insert(
        "status".to_string(),
        Value::String("not-requested".to_string()),
    );
    output.insert(
        "rustFiles".to_string(),
        Value::Number(input.run.rust_files.unwrap_or(0).into()),
    );
    if let Some(artifact) = string_field_from_value(input.evidence.as_ref(), "artifact") {
        output.insert("artifact".to_string(), Value::String(artifact.to_string()));
    }
    if let Some(status) = status_field(input.evidence.as_ref()) {
        output.insert(
            "artifactStatus".to_string(),
            Value::String(status.to_string()),
        );
    }
    Ok(Value::Object(output))
}

pub fn summarize_rust_analysis_artifact(
    root: &Path,
    artifact: &Value,
) -> Option<RustAnalysisSummary> {
    if !artifact.is_object() {
        return None;
    }

    let artifact_root = artifact.pointer("/meta/input/root").and_then(Value::as_str);
    if !same_resolved_path(artifact_root, root) {
        return Some(unavailable_summary(
            RustAnalysisStatus::RootMismatch,
            artifact_root.map(ToOwned::to_owned),
        ));
    }

    let Some(summary) = artifact.get("summary").and_then(Value::as_object) else {
        return Some(unavailable_summary(
            RustAnalysisStatus::InvalidShape,
            artifact_root.map(ToOwned::to_owned),
        ));
    };

    if artifact
        .get("schemaVersion")
        .and_then(Value::as_str)
        .is_none()
        || artifact
            .get("policyVersion")
            .and_then(Value::as_str)
            .is_none()
        || artifact.pointer("/meta/producer").and_then(Value::as_str) != Some("lumin-rust-analyzer")
        || artifact.pointer("/meta/mode").and_then(Value::as_str) != Some("rust-main")
        || summary.get("files").and_then(Value::as_u64).is_none()
    {
        return Some(unavailable_summary(
            RustAnalysisStatus::InvalidShape,
            artifact_root.map(ToOwned::to_owned),
        ));
    }

    Some(RustAnalysisSummary {
        artifact: ARTIFACT_NAME,
        status: RustAnalysisStatus::Complete,
        available: true,
        root: None,
        schema_version: string_field(artifact, "/schemaVersion"),
        policy_version: string_field(artifact, "/policyVersion"),
        producer: string_field(artifact, "/meta/producer"),
        mode: string_field(artifact, "/meta/mode"),
        source_health_profile: artifact
            .pointer("/meta/input/effectiveSourceHealthProfile")
            .or_else(|| artifact.pointer("/meta/input/sourceHealthProfile"))
            .cloned(),
        semantic_mode: artifact.pointer("/meta/input/semanticMode").cloned(),
        scan_scope: scan_scope_from_artifact(artifact),
        files: number_or_zero(summary.get("files")),
        syntax_review_signals: number_or_zero(summary.get("syntaxReviewSignals")),
        syntax_review_opaque_surfaces: number_or_zero(summary.get("syntaxReviewOpaqueSurfaces")),
        syntax_function_clone_exact_body_groups: number_or_zero(
            summary.get("syntaxFunctionCloneExactBodyGroups"),
        ),
        syntax_function_clone_structure_groups: number_or_zero(
            summary.get("syntaxFunctionCloneStructureGroups"),
        ),
        syntax_function_clone_signature_groups: number_or_zero(
            summary.get("syntaxFunctionCloneSignatureGroups"),
        ),
        syntax_function_clone_near_candidates: number_or_zero(
            summary.get("syntaxFunctionCloneNearCandidates"),
        ),
        action_tier_summary: summary.get("actionTierSummary").cloned(),
        oracle_bridge_status: summary.get("oracleBridgeStatus").cloned(),
    })
}

fn validate_rust_analysis_run(run: &RustAnalysisRunObservation) -> AnyResult<()> {
    if run.status.trim().is_empty() {
        bail!("rust-analysis-run-merge: run.status must be a non-empty string");
    }
    Ok(())
}

fn run_object(run: RustAnalysisRunObservation) -> AnyResult<serde_json::Map<String, Value>> {
    let Value::Object(map) = serde_json::to_value(run)? else {
        bail!("rust-analysis-run-merge: invalid run shape");
    };
    Ok(map)
}

fn object_field_map(value: Option<&Value>) -> serde_json::Map<String, Value> {
    value
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

fn status_field(value: Option<&Value>) -> Option<&str> {
    string_field_from_value(value, "status")
}

fn string_field_from_value<'a>(value: Option<&'a Value>, field: &str) -> Option<&'a str> {
    value?.get(field)?.as_str()
}

fn unavailable_summary(status: RustAnalysisStatus, root: Option<String>) -> RustAnalysisSummary {
    RustAnalysisSummary {
        artifact: ARTIFACT_NAME,
        status,
        available: false,
        root,
        schema_version: None,
        policy_version: None,
        producer: None,
        mode: None,
        source_health_profile: None,
        semantic_mode: None,
        scan_scope: None,
        files: 0,
        syntax_review_signals: 0,
        syntax_review_opaque_surfaces: 0,
        syntax_function_clone_exact_body_groups: 0,
        syntax_function_clone_structure_groups: 0,
        syntax_function_clone_signature_groups: 0,
        syntax_function_clone_near_candidates: 0,
        action_tier_summary: None,
        oracle_bridge_status: None,
    }
}

fn same_resolved_path(artifact_root: Option<&str>, root: &Path) -> bool {
    let Some(artifact_root) = artifact_root else {
        return false;
    };

    let artifact_root = Path::new(artifact_root);
    match (
        std::fs::canonicalize(artifact_root),
        std::fs::canonicalize(root),
    ) {
        (Ok(left), Ok(right)) => left == right,
        _ => absolute_path(artifact_root) == absolute_path(root),
    }
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    match std::env::current_dir() {
        Ok(current_dir) => current_dir.join(path),
        Err(_) => path.to_path_buf(),
    }
}

fn string_field(value: &Value, pointer: &str) -> Option<String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn number_or_zero(value: Option<&Value>) -> u64 {
    value.and_then(Value::as_u64).unwrap_or(0)
}

fn scan_scope_from_artifact(artifact: &Value) -> Option<ScanScope> {
    let input = artifact.pointer("/meta/input").and_then(Value::as_object);
    let syntax_input = artifact
        .pointer("/phases/syntax/meta/input")
        .and_then(Value::as_object);

    let include_tests =
        bool_field(input, "includeTests").or_else(|| bool_field(syntax_input, "includeTests"));
    let exclude = string_array_from(input.and_then(|input| input.get("exclude")))
        .or_else(|| string_array_from(syntax_input.and_then(|input| input.get("exclude"))));
    let path_policy = object_field(syntax_input, "pathPolicy")
        .or_else(|| object_field(input, "pathPolicy"))
        .cloned();

    if include_tests.is_none() && exclude.is_none() && path_policy.is_none() {
        return None;
    }

    Some(ScanScope {
        include_tests,
        exclude,
        path_policy,
    })
}

fn bool_field(input: Option<&serde_json::Map<String, Value>>, field: &str) -> Option<bool> {
    input
        .and_then(|input| input.get(field))
        .and_then(Value::as_bool)
}

fn object_field<'a>(
    input: Option<&'a serde_json::Map<String, Value>>,
    field: &str,
) -> Option<&'a Value> {
    input
        .and_then(|input| input.get(field))
        .filter(|value| value.is_object())
}

fn string_array_from(value: Option<&Value>) -> Option<Vec<String>> {
    let values = value?.as_array()?;
    Some(
        values
            .iter()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect(),
    )
}
