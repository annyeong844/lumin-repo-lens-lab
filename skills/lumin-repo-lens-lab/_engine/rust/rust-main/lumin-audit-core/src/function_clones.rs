use anyhow::{bail, Result};
use serde_json::{json, Map, Value};

mod facts;
mod groups;
mod near;
mod projection;
mod protocol;
#[cfg(test)]
mod tests;

use facts::{compare_facts, stamp_observed_at, FunctionFact};
use groups::{exact_body_groups, signature_groups, structure_groups};
use near::{build_near_function_candidates, function_clone_near_policy_summary};
use projection::{non_generated_count, sort_diagnostics};
pub use protocol::{FunctionClonesRequest, FUNCTION_CLONES_REQUEST_SCHEMA_VERSION};

const FUNCTION_CLONE_SCHEMA_VERSION: &str = "function-clones.v3";
const FUNCTION_CLONE_NORMALIZED_VERSION: &str = "function-body.normalized.v1";
const FUNCTION_SIGNATURE_NORMALIZED_VERSION: &str = "function-signature.normalized.v1";

pub fn build_function_clones_artifact(request: FunctionClonesRequest) -> Result<Value> {
    if request.schema_version != FUNCTION_CLONES_REQUEST_SCHEMA_VERSION {
        bail!(
            "function-clones-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let observed_at = request
        .observed_at
        .clone()
        .unwrap_or_else(|| request.generated.clone());
    let mut stamped_facts = request
        .facts
        .into_iter()
        .map(|fact| FunctionFact::from_value(stamp_observed_at(fact, &observed_at)))
        .collect::<Vec<_>>();

    stamped_facts.sort_by(compare_facts);
    let sorted_diagnostics = sort_diagnostics(request.diagnostics);

    let exact_body_groups = exact_body_groups(&stamped_facts);
    let structure_groups = structure_groups(&stamped_facts);
    let signature_groups = signature_groups(&stamped_facts);
    let near_function_candidates =
        build_near_function_candidates(&stamped_facts, &exact_body_groups, &structure_groups);
    let generated_file_fact_count = stamped_facts
        .iter()
        .filter(|fact| fact.generated_file)
        .count();

    let mut meta = Map::new();
    meta.insert("tool".to_string(), json!("build-function-clone-index.mjs"));
    meta.insert("generated".to_string(), json!(request.generated));
    meta.insert("root".to_string(), json!(request.root));
    meta.insert("source".to_string(), json!("fresh-ast-pass"));
    meta.insert("scope".to_string(), json!(request.scope));
    meta.insert("observedAt".to_string(), json!(observed_at));
    meta.insert(
        "complete".to_string(),
        json!(
            request.files_with_read_errors.is_empty() && request.files_with_parse_errors.is_empty()
        ),
    );
    meta.insert("includeTests".to_string(), json!(request.include_tests));
    meta.insert("exclude".to_string(), Value::Array(request.exclude));
    meta.insert("fileCount".to_string(), json!(request.file_count));
    meta.insert("factCount".to_string(), json!(stamped_facts.len()));
    meta.insert(
        "generatedFileFactCount".to_string(),
        json!(generated_file_fact_count),
    );
    meta.insert(
        "exactBodyGroupCount".to_string(),
        json!(non_generated_count(&exact_body_groups)),
    );
    meta.insert(
        "structureGroupCount".to_string(),
        json!(non_generated_count(&structure_groups)),
    );
    meta.insert(
        "signatureGroupCount".to_string(),
        json!(non_generated_count(&signature_groups)),
    );
    meta.insert(
        "nearFunctionCandidateCount".to_string(),
        json!(non_generated_count(&near_function_candidates)),
    );
    meta.insert(
        "diagnosticCount".to_string(),
        json!(sorted_diagnostics.len()),
    );
    meta.insert(
        "filesWithParseErrors".to_string(),
        Value::Array(request.files_with_parse_errors),
    );
    meta.insert(
        "filesWithReadErrors".to_string(),
        Value::Array(request.files_with_read_errors),
    );
    meta.insert(
        "thresholdPolicies".to_string(),
        Value::Array(vec![function_clone_near_policy_summary()]),
    );
    if let Some(incremental) = request.incremental {
        meta.insert("incremental".to_string(), incremental);
    }
    meta.insert(
        "supports".to_string(),
        json!({
            "exportedTopLevelFunctions": true,
            "fileLocalTopLevelFunctions": true,
            "functionFactVisibility": true,
            "exportedConstArrowFunctions": true,
            "defaultFunctionExports": true,
            "exactBodyHash": true,
            "normalizedExactHash": true,
            "normalizedStructureHash": true,
            "normalizedVersion": FUNCTION_CLONE_NORMALIZED_VERSION,
            "normalizedFunctionSignatureHash": true,
            "functionSignatureGroups": true,
            "functionSignatureNormalizedVersion": FUNCTION_SIGNATURE_NORMALIZED_VERSION,
            "nearFunctionCandidates": true,
            "generatedFileEvidence": true,
            "semanticEquivalence": false,
        }),
    );
    meta.insert(
        "caveat".to_string(),
        json!("Function clone groups and near candidates are deterministic review cues. They do not prove semantic equivalence or justify automatic merging."),
    );

    Ok(json!({
        "schemaVersion": FUNCTION_CLONE_SCHEMA_VERSION,
        "meta": meta,
        "facts": stamped_facts.into_iter().map(|fact| fact.value).collect::<Vec<_>>(),
        "exactBodyGroups": exact_body_groups,
        "structureGroups": structure_groups,
        "signatureGroups": signature_groups,
        "nearFunctionCandidates": near_function_candidates,
        "diagnostics": sorted_diagnostics,
    }))
}
