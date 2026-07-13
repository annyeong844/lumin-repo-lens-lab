mod function_size;
mod presentation;
mod projections;
mod protocol;
mod shape_drift;
mod topology;
mod value_support;

use anyhow::{bail, Result};
use serde_json::{json, Map, Value};

use function_size::function_size;
use presentation::{annotate, not_computed_items, unavailable};
use projections::{
    barrel_amplification, dead_code, duplicate_implementation, lint_enforcement, silent_catch,
};
pub use protocol::{
    ChecklistAstFacts, ChecklistFactsRequest, ChecklistInputArtifacts, FunctionSizeEntry,
    FunctionSizeFacts, SilentCatchFacts,
};
use shape_drift::shape_drift;
use topology::{cycles, decoupling_ratio};
use value_support::{
    as_usize, generated_only_count, identities_key, non_generated_array, number_field,
    parse_percent, round3, text_field, unique_sorted_strings, value_at,
};

pub const CHECKLIST_FACTS_REQUEST_SCHEMA_VERSION: &str =
    "lumin-checklist-facts-producer-request.v1";

const TOOL_NAME: &str = "checklist-facts.mjs";
const ARTIFACT_SCHEMA_VERSION: usize = 9;

pub fn build_checklist_facts_artifact(request: ChecklistFactsRequest) -> Result<Value> {
    if request.schema_version != CHECKLIST_FACTS_REQUEST_SCHEMA_VERSION {
        bail!(
            "checklist-facts-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let mut artifact = Map::new();
    artifact.insert(
        "meta".to_string(),
        json!({
            "generated": request.generated,
            "root": request.root,
            "tool": TOOL_NAME,
            "schemaVersion": ARTIFACT_SCHEMA_VERSION,
            "filesScanned": request.files_scanned,
            "inputsPresent": {
                "topology.json": request.inputs.topology.is_some(),
                "dead-classify.json": request.inputs.dead_classify.is_some(),
                "fix-plan.json": request.inputs.fix_plan.is_some(),
                "barrels.json": request.inputs.barrels.is_some(),
                "triage.json": request.inputs.triage.is_some(),
                "shape-index.json": request.inputs.shape_index.is_some(),
                "function-clones.json": request.inputs.function_clones.is_some(),
            }
        }),
    );

    artifact.insert(
        "A2_function_size".to_string(),
        annotate(
            "A2_function_size",
            function_size(&request.ast_facts.function_size),
            true,
        ),
    );
    artifact.insert(
        "A5_decoupling_ratio".to_string(),
        annotate(
            "A5_decoupling_ratio",
            decoupling_ratio(request.inputs.topology.as_ref()),
            true,
        ),
    );
    artifact.insert(
        "A6_circular_deps".to_string(),
        annotate(
            "A6_circular_deps",
            cycles(request.inputs.topology.as_ref()),
            false,
        ),
    );
    artifact.insert(
        "B1_duplicate_implementation".to_string(),
        annotate(
            "B1_duplicate_implementation",
            duplicate_implementation(request.inputs.function_clones.as_ref()),
            true,
        ),
    );
    artifact.insert(
        "B3_dead_code".to_string(),
        annotate(
            "B3_dead_code",
            dead_code(request.inputs.fix_plan.as_ref()),
            true,
        ),
    );
    artifact.insert(
        "B1B2_shape_drift".to_string(),
        annotate(
            "B1B2_shape_drift",
            shape_drift(request.inputs.shape_index.as_ref()),
            true,
        ),
    );
    artifact.insert(
        "C5_lint_enforcement".to_string(),
        annotate(
            "C5_lint_enforcement",
            lint_enforcement(request.inputs.triage.as_ref()),
            true,
        ),
    );
    artifact.insert(
        "C7_barrel_amplification".to_string(),
        annotate(
            "C7_barrel_amplification",
            barrel_amplification(request.inputs.barrels.as_ref()),
            true,
        ),
    );
    artifact.insert(
        "E2_silent_catch".to_string(),
        annotate(
            "E2_silent_catch",
            silent_catch(&request.ast_facts.silent_catch),
            true,
        ),
    );
    artifact.insert("_not_computed".to_string(), not_computed_items());

    Ok(Value::Object(artifact))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_checklist_artifact_from_ast_and_optional_inputs() -> Result<()> {
        let artifact = build_checklist_facts_artifact(ChecklistFactsRequest {
            schema_version: CHECKLIST_FACTS_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-04T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            files_scanned: 2,
            inputs: ChecklistInputArtifacts {
                topology: Some(json!({
                    "summary": { "internalEdges": 100, "sccCount": 0, "maxSccSize": 0, "lens": "runtime" },
                    "crossSubmoduleEdges": [
                        { "from": "root", "to": "_lib", "count": 60 },
                        { "from": "tests", "to": "_lib", "count": 20 }
                    ],
                    "sccs": []
                })),
                fix_plan: Some(
                    json!({ "summary": { "SAFE_FIX": 1, "REVIEW_FIX": 2, "DEGRADED": 3, "MUTED": 4, "total": 10 } }),
                ),
                triage: Some(json!({ "boundaries": [{ "rule": "no-restricted-imports" }] })),
                barrels: Some(json!({ "mode": "single-package" })),
                ..ChecklistInputArtifacts::default()
            },
            ast_facts: ChecklistAstFacts {
                function_size: FunctionSizeFacts {
                    parse_errors: 0,
                    entries: vec![FunctionSizeEntry {
                        file: "src/huge.ts".to_string(),
                        line: 1,
                        name: "huge".to_string(),
                        loc: 160,
                        file_role: "production".to_string(),
                    }],
                },
                silent_catch: SilentCatchFacts::default(),
            },
        })?;

        assert_eq!(artifact["meta"]["tool"], TOOL_NAME);
        assert_eq!(artifact["A2_function_size"]["gate"], "watch");
        assert_eq!(artifact["A5_decoupling_ratio"]["rawGate"], "fix");
        assert_eq!(artifact["A5_decoupling_ratio"]["gate"], "ok");
        assert_eq!(artifact["B3_dead_code"]["gate"], "watch");
        assert_eq!(artifact["C5_lint_enforcement"]["gate"], "ok");
        assert!(artifact["_not_computed"]
            .as_array()
            .is_some_and(|entries| entries.len() >= 20));
        Ok(())
    }
}
