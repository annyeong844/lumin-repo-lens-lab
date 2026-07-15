use serde_json::{json, Map, Value};

pub(super) fn unavailable(reason: &str) -> Value {
    json!({
        "gate": "unknown",
        "available": false,
        "reason": reason,
    })
}

pub(super) fn annotate(section_key: &str, result: Value, context_check: bool) -> Value {
    let mut object = match result {
        Value::Object(object) => object,
        other => {
            let mut object = Map::new();
            object.insert("value".to_string(), other);
            object
        }
    };
    let snapshot = Value::Object(object.clone());
    object.insert(
        "_citation_hint".to_string(),
        Value::String(citation_for(section_key, &snapshot)),
    );
    object.insert(
        "_context_check_required".to_string(),
        Value::Bool(context_check),
    );
    Value::Object(object)
}

fn citation_for(section_key: &str, result: &Value) -> String {
    if result.get("available").and_then(Value::as_bool) == Some(false) {
        let reason = result
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("run pipeline prerequisites");
        return format!("[확인 불가, scan range: {section_key} input artifact missing — {reason}]");
    }
    match section_key {
        "A2_function_size" => format!(
            "[grounded, checklist-facts.json.A2_function_size.buckets = {}, roleBuckets = {}]",
            compact_json(result.get("buckets")),
            compact_json(result.get("roleBuckets"))
        ),
        "A5_decoupling_ratio" => format!(
            "[grounded, checklist-facts.json.A5_decoupling_ratio.ratioLowerBound = {}]",
            display_value(result.get("ratioLowerBound"))
        ),
        "A6_circular_deps" => format!(
            "[grounded, checklist-facts.json.A6_circular_deps.sccCount = {}, lens = {}]",
            display_value(result.get("sccCount")),
            display_value(result.get("lens"))
        ),
        "B3_dead_code" => format!(
            "[grounded, checklist-facts.json.B3_dead_code = {{safeFix: {}, reviewFix: {}, degraded: {}, muted: {}, total: {}}}]",
            display_value(result.get("safeFix")),
            display_value(result.get("reviewFix")),
            display_value(result.get("degraded")),
            display_value(result.get("muted")),
            display_value(result.get("total"))
        ),
        "B1B2_shape_drift" => format!(
            "[grounded, checklist-facts.json.B1B2_shape_drift.exactDuplicateGroups = {}, nearShapeCandidateCount = {}, duplicateIdentityCount = {}]",
            display_value(result.get("exactDuplicateGroups")),
            display_value(result.get("nearShapeCandidateCount")),
            display_value(result.get("duplicateIdentityCount"))
        ),
        "B1_duplicate_implementation" => format!(
            "[grounded, checklist-facts.json.B1_duplicate_implementation.exactBodyGroups = {}, structureGroupCandidates = {}, signatureGroupCandidates = {}, nearFunctionCandidates = {}]",
            display_value(result.get("exactBodyGroups")),
            display_value(result.get("structureGroupCandidates")),
            display_value(result.get("signatureGroupCandidates")),
            display_value(result.get("nearFunctionCandidates"))
        ),
        "C5_lint_enforcement" if result.get("gate").and_then(Value::as_str) == Some("unknown") => format!(
            "[unknown, checklist-facts.json.C5_lint_enforcement.lintEvidenceStatus = {}, reason = {}]",
            display_value(result.get("lintEvidenceStatus")),
            display_value(result.get("reason"))
        ),
        "C5_lint_enforcement" => format!(
            "[grounded, checklist-facts.json.C5_lint_enforcement.boundaryRulePresent = {}, rulesDetected = {}]",
            display_value(result.get("boundaryRulePresent")),
            display_value(result.get("rulesDetected"))
        ),
        "C7_barrel_amplification" => format!(
            "[grounded, checklist-facts.json.C7_barrel_amplification.worstCompliance = {}]",
            result
                .get("worstCompliance")
                .map(display_json_value)
                .unwrap_or_else(|| "n/a".to_string())
        ),
        "E2_silent_catch" => format!(
            "[grounded, checklist-facts.json.E2_silent_catch.count = {}, nonEmptyAnonymousCount = {}, unusedParamCount = {}, analysis = {}]",
            display_value(result.get("count")),
            display_value(result.get("nonEmptyAnonymousCount")),
            display_value(result.get("unusedParamCount")),
            display_value(result.get("analysis"))
        ),
        _ => format!("[grounded, checklist-facts.json.{section_key}]"),
    }
}

pub(super) fn not_computed_items() -> Value {
    json!([
        { "item": "A1", "reason": "summary of A2-A6 — synthesize after reading sub-items" },
        { "item": "A3", "reason": "helper zoo — needs per-file export fan-in map; symbols.json currently emits only topSymbolFanIn (top 50)" },
        { "item": "A4", "reason": "over-split — needs per-file fanIn/fanOut; topology.json currently emits only top lists" },
        { "item": "B1", "reason": "broader duplicate implementation still requires LLM review; B1_duplicate_implementation covers top-level exported and file-local exact body, same-structure, same-signature, and near function clone cues only" },
        { "item": "B2", "reason": "broader shared-shape drift still requires domain/vocab judgment; nearShapeCandidates are artifact-backed review cues only" },
        { "item": "B4", "reason": "pipeline duplication — semantic comparison across script entry points" },
        { "item": "C1", "reason": "cohesion / SRP — LLM reads file name vs body alignment" },
        { "item": "C2", "reason": "boundary health — LLM reads cross-submodule direction for inversion patterns" },
        { "item": "C3", "reason": "crosscut concerns — LLM identifies validation / normalization / error patterns" },
        { "item": "C4", "reason": "single state-mutation entry — dataflow analysis" },
        { "item": "C6", "reason": "file hierarchy health — LLM reads triage.topDirs shape" },
        { "item": "D1", "reason": "type tightness — JS has limited static info; discipline.json gives counts" },
        { "item": "D2", "reason": "interface/generic appropriateness — LLM judgment" },
        { "item": "D3", "reason": "naming consistency — LLM scan of sibling exports" },
        { "item": "D4", "reason": "implicit coupling — LLM identifies side-effect-only imports, init-order deps" },
        { "item": "D5", "reason": "discriminated-union candidates — LLM judgment" },
        { "item": "E1", "reason": "defensive-code density — LLM judgment across sites" },
        { "item": "E3", "reason": "fallback hiding bugs — LLM reads catch-then-return-null patterns" },
        { "item": "E4", "reason": "catch re-classification — LLM inspects rethrown error types" },
        { "item": "E5", "reason": "resource cleanup — AST possible but heuristic; not implemented" },
        { "item": "E6", "reason": "fire-and-forget Promise — AST possible but heuristic; not implemented" },
        { "item": "F1", "reason": "abstraction level — LLM judgment" },
        { "item": "F2", "reason": "test coverage of edge/failure cases — merge with c8 report when available" },
        { "item": "F3", "reason": "test-to-contract coupling — LLM reads assertions" },
        { "item": "F4", "reason": "mock boundary depth — LLM judgment" },
    ])
}

fn compact_json(value: Option<&Value>) -> String {
    value
        .map(display_json_value)
        .unwrap_or_else(|| "null".to_string())
}

fn display_value(value: Option<&Value>) -> String {
    value
        .map(display_json_value)
        .unwrap_or_else(|| "unknown".to_string())
}

fn display_json_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        other => serde_json::to_string(other).unwrap_or_else(|_| "unknown".to_string()),
    }
}
