use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

pub const CHECKLIST_FACTS_REQUEST_SCHEMA_VERSION: &str =
    "lumin-checklist-facts-producer-request.v1";

const TOOL_NAME: &str = "checklist-facts.mjs";
const ARTIFACT_SCHEMA_VERSION: usize = 9;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistFactsRequest {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    #[serde(default)]
    pub files_scanned: usize,
    #[serde(default)]
    pub inputs: ChecklistInputArtifacts,
    pub ast_facts: ChecklistAstFacts,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistInputArtifacts {
    #[serde(default)]
    pub topology: Option<Value>,
    #[serde(default)]
    pub dead_classify: Option<Value>,
    #[serde(default)]
    pub fix_plan: Option<Value>,
    #[serde(default)]
    pub barrels: Option<Value>,
    #[serde(default)]
    pub triage: Option<Value>,
    #[serde(default)]
    pub shape_index: Option<Value>,
    #[serde(default)]
    pub function_clones: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistAstFacts {
    pub function_size: FunctionSizeFacts,
    pub silent_catch: SilentCatchFacts,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionSizeFacts {
    #[serde(default)]
    pub entries: Vec<FunctionSizeEntry>,
    #[serde(default)]
    pub parse_errors: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionSizeEntry {
    pub file: String,
    #[serde(default)]
    pub line: usize,
    pub name: String,
    #[serde(default)]
    pub loc: usize,
    #[serde(default)]
    pub file_role: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SilentCatchFacts {
    #[serde(default = "default_silent_catch_analysis")]
    pub analysis: String,
    #[serde(default)]
    pub parse_errors: usize,
    #[serde(default)]
    pub sites: Vec<Value>,
    #[serde(default)]
    pub documented_sites: Vec<Value>,
    #[serde(default)]
    pub anonymous_sites: Vec<Value>,
    #[serde(default)]
    pub non_empty_anonymous_sites: Vec<Value>,
    #[serde(default)]
    pub unused_param_sites: Vec<Value>,
}

#[derive(Debug, Clone)]
struct CrossEdge {
    from: String,
    to: String,
    count: usize,
}

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
            a2_function_size(&request.ast_facts.function_size),
            true,
        ),
    );
    artifact.insert(
        "A5_decoupling_ratio".to_string(),
        annotate(
            "A5_decoupling_ratio",
            a5_decoupling_ratio(request.inputs.topology.as_ref()),
            true,
        ),
    );
    artifact.insert(
        "A6_circular_deps".to_string(),
        annotate(
            "A6_circular_deps",
            a6_cycles(request.inputs.topology.as_ref()),
            false,
        ),
    );
    artifact.insert(
        "B1_duplicate_implementation".to_string(),
        annotate(
            "B1_duplicate_implementation",
            b1_duplicate_implementation(request.inputs.function_clones.as_ref()),
            true,
        ),
    );
    artifact.insert(
        "B3_dead_code".to_string(),
        annotate(
            "B3_dead_code",
            b3_dead_code(request.inputs.fix_plan.as_ref()),
            true,
        ),
    );
    artifact.insert(
        "B1B2_shape_drift".to_string(),
        annotate(
            "B1B2_shape_drift",
            b1b2_shape_drift(request.inputs.shape_index.as_ref()),
            true,
        ),
    );
    artifact.insert(
        "C5_lint_enforcement".to_string(),
        annotate(
            "C5_lint_enforcement",
            c5_lint_enforcement(request.inputs.triage.as_ref()),
            true,
        ),
    );
    artifact.insert(
        "C7_barrel_amplification".to_string(),
        annotate(
            "C7_barrel_amplification",
            c7_barrel_amplification(request.inputs.barrels.as_ref()),
            true,
        ),
    );
    artifact.insert(
        "E2_silent_catch".to_string(),
        annotate(
            "E2_silent_catch",
            e2_silent_catch(&request.ast_facts.silent_catch),
            true,
        ),
    );
    artifact.insert("_not_computed".to_string(), not_computed_items());

    Ok(Value::Object(artifact))
}

fn default_silent_catch_analysis() -> String {
    "oxc-ast-catch-clause".to_string()
}

fn a2_function_size(facts: &FunctionSizeFacts) -> Value {
    let mut buckets = RoleBucket::default();
    let mut role_buckets = BTreeMap::<String, RoleBucket>::from([
        ("production".to_string(), RoleBucket::default()),
        ("test".to_string(), RoleBucket::default()),
        ("script".to_string(), RoleBucket::default()),
    ]);
    let mut all_loc = Vec::<usize>::new();
    let mut oversized = Vec::<(usize, FunctionSizeEntry)>::new();
    let mut watch = Vec::<(usize, FunctionSizeEntry)>::new();

    for (index, entry) in facts.entries.iter().cloned().enumerate() {
        let loc = entry.loc.max(1);
        all_loc.push(loc);
        let role = normalized_role(&entry.file_role);
        let role_bucket = role_buckets.entry(role).or_default();
        role_bucket.total += 1;
        if loc > 150 {
            buckets.big += 1;
            role_bucket.big += 1;
            oversized.push((index, entry));
        } else if loc > 100 {
            buckets.medium += 1;
            role_bucket.medium += 1;
            watch.push((index, entry));
        } else {
            buckets.small += 1;
            role_bucket.small += 1;
        }
    }

    all_loc.sort_unstable();
    let p95 = if all_loc.is_empty() {
        0
    } else {
        all_loc[all_loc.len() * 95 / 100]
    };
    let sort_by_loc = |items: &mut Vec<(usize, FunctionSizeEntry)>| {
        items.sort_by(|(left_index, left), (right_index, right)| {
            right
                .loc
                .cmp(&left.loc)
                .then_with(|| left_index.cmp(right_index))
        });
    };
    sort_by_loc(&mut oversized);
    sort_by_loc(&mut watch);

    let gate = if buckets.big >= 3 {
        "fix"
    } else if buckets.big >= 1 {
        "watch"
    } else {
        "ok"
    };

    json!({
        "gate": gate,
        "buckets": buckets.to_value(),
        "roleBuckets": role_buckets_to_value(&role_buckets),
        "p95Loc": p95,
        "total": all_loc.len(),
        "parseErrors": facts.parse_errors,
        "oversized": entries_to_values(&oversized, 20),
        "watch": entries_to_values(&watch, 20),
        "oversizedByRole": entries_by_role(&oversized),
        "watchByRole": entries_by_role(&watch),
    })
}

fn a5_decoupling_ratio(topology: Option<&Value>) -> Value {
    let Some(topology) = topology else {
        return unavailable("topology.json missing — run measure-topology.mjs first");
    };
    if !topology.get("summary").is_some_and(Value::is_object) {
        return unavailable("topology.json missing — run measure-topology.mjs first");
    }

    let total = value_at(topology, &["summary", "internalEdges"])
        .and_then(as_usize)
        .unwrap_or(0);
    let (source, mut edges) = normalize_cross_submodule_edges(topology);
    let cross_sum: usize = edges.iter().map(|edge| edge.count).sum();
    let layered_sum: usize = edges
        .iter()
        .filter(|edge| is_healthy_layered_cross_edge(edge))
        .map(|edge| edge.count)
        .sum();
    let reviewed_sum = cross_sum.saturating_sub(layered_sum);
    let ratio = if total > 0 {
        cross_sum as f64 / total as f64
    } else {
        0.0
    };
    let raw_gate = if ratio > 0.5 {
        "fix"
    } else if ratio > 0.3 {
        "watch"
    } else {
        "ok"
    };
    let gate = if raw_gate != "ok" && cross_sum > 0 && reviewed_sum == 0 {
        "ok"
    } else {
        raw_gate
    };
    edges.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.from.cmp(&b.from))
            .then_with(|| a.to.cmp(&b.to))
    });
    let note = if source == "full-list" {
        "ratio is exact from topology.json.crossSubmoduleEdges. Healthy layered flows (root/scripts/tests → _lib, tests → production) are visible but do not trip the gate by themselves."
    } else {
        "ratio is a LOWER bound from topology.json.crossSubmoduleTop; the true ratio may be slightly higher."
    };

    json!({
        "gate": gate,
        "rawGate": raw_gate,
        "crossSubmoduleEdgeSource": source,
        "crossSubmoduleEdgesSum": cross_sum,
        "crossSubmoduleEdgesTop30Sum": if source == "top-30" { json!(cross_sum) } else { Value::Null },
        "healthyLayeredEdgesSum": layered_sum,
        "reviewedEdgesSum": reviewed_sum,
        "totalInternalEdges": total,
        "ratioLowerBound": round3(ratio),
        "topCrossSubmoduleEdges": edges.iter().take(10).map(cross_edge_value).collect::<Vec<_>>(),
        "note": note,
    })
}

fn a6_cycles(topology: Option<&Value>) -> Value {
    let Some(topology) = topology else {
        return unavailable("topology.json missing");
    };
    let Some(sccs) = topology.get("sccs").and_then(Value::as_array) else {
        return unavailable("topology.json missing");
    };
    let nontrivial = sccs
        .iter()
        .filter(|scc| scc.get("size").and_then(as_usize).unwrap_or(0) >= 2)
        .cloned()
        .collect::<Vec<_>>();
    json!({
        "gate": if nontrivial.is_empty() { "ok" } else { "fix" },
        "sccCount": value_at(topology, &["summary", "sccCount"]).and_then(as_usize).unwrap_or(nontrivial.len()),
        "maxSccSize": value_at(topology, &["summary", "maxSccSize"]).and_then(as_usize).unwrap_or(0),
        "lens": value_at(topology, &["summary", "lens"]).and_then(Value::as_str).unwrap_or("unknown"),
        "topSccs": nontrivial.into_iter().take(5).collect::<Vec<_>>(),
    })
}

fn b3_dead_code(fix_plan: Option<&Value>) -> Value {
    let Some(summary) = fix_plan.and_then(|artifact| artifact.get("summary")) else {
        return unavailable(
            "fix-plan.json missing — run rank-fixes.mjs after classify-dead-exports.mjs",
        );
    };
    let safe_fix = summary.get("SAFE_FIX").and_then(as_usize).unwrap_or(0);
    json!({
        "gate": if safe_fix >= 10 { "fix" } else if safe_fix > 0 { "watch" } else { "ok" },
        "safeFix": safe_fix,
        "reviewFix": summary.get("REVIEW_FIX").and_then(as_usize).unwrap_or(0),
        "degraded": summary.get("DEGRADED").and_then(as_usize).unwrap_or(0),
        "muted": summary.get("MUTED").and_then(as_usize).unwrap_or(0),
        "total": summary.get("total").and_then(as_usize).unwrap_or(0),
    })
}

fn b1b2_shape_drift(shape_index: Option<&Value>) -> Value {
    let Some(shape_index) = shape_index else {
        return unavailable(
            "shape-index.json missing — run full profile or build-shape-index.mjs first",
        );
    };
    let facts = shape_index
        .get("facts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let facts_by_identity = facts
        .iter()
        .filter_map(|fact| {
            let identity = fact.get("identity")?.as_str()?;
            Some((identity.to_string(), fact.clone()))
        })
        .collect::<BTreeMap<_, _>>();

    let mut groups = Vec::<Value>::new();
    if let Some(groups_by_hash) = shape_index.get("groupsByHash").and_then(Value::as_object) {
        for (hash, identities) in groups_by_hash {
            let Some(identities) = identities.as_array() else {
                continue;
            };
            if identities.len() < 2 {
                continue;
            }
            let identity_strings = identities
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>();
            if let Some(group) = summarize_shape_group(hash, &identity_strings, &facts_by_identity)
            {
                groups.push(group);
            }
        }
    }

    groups.sort_by(shape_group_rank);
    let non_generated_groups = groups
        .iter()
        .filter(|group| {
            !group
                .get("generatedOnly")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();
    let near_shape_candidates = collect_near_shape_candidates(&facts);
    let gate = if !non_generated_groups.is_empty() || !near_shape_candidates.is_empty() {
        "watch"
    } else {
        "ok"
    };
    let duplicate_identity_count = non_generated_groups
        .iter()
        .map(|group| group.get("size").and_then(as_usize).unwrap_or(0))
        .sum::<usize>();

    json!({
        "gate": gate,
        "available": true,
        "exactDuplicateGroups": non_generated_groups.len(),
        "nearShapeCandidateCount": near_shape_candidates.len(),
        "generatedOnlyGroups": groups.len().saturating_sub(non_generated_groups.len()),
        "duplicateIdentityCount": duplicate_identity_count,
        "totalShapeFacts": facts.len(),
        "shapeIndexComplete": value_at(shape_index, &["meta", "complete"]).and_then(Value::as_bool).unwrap_or(true),
        "topGroups": non_generated_groups.into_iter().take(10).collect::<Vec<_>>(),
        "nearShapeCandidates": near_shape_candidates.iter().take(10).cloned().collect::<Vec<_>>(),
        "generatedOnlySummary": groups
            .iter()
            .filter(|group| group.get("generatedOnly").and_then(Value::as_bool).unwrap_or(false))
            .take(5)
            .map(|group| json!({
                "hash": group.get("hash").cloned().unwrap_or(Value::Null),
                "size": group.get("size").cloned().unwrap_or(Value::Null),
                "ownerFiles": group.get("ownerFiles").cloned().unwrap_or_else(|| json!([])),
            }))
            .collect::<Vec<_>>(),
        "note": "Exact and near exported type-shape matches only. Treat as review cues, not proof of duplicated implementation or an automatic refactor.",
    })
}

fn b1_duplicate_implementation(function_clones: Option<&Value>) -> Value {
    let Some(function_clones) = function_clones else {
        return unavailable("function-clones.json missing — run full profile or build-function-clone-index.mjs first");
    };
    let exact_groups = non_generated_array(function_clones, "exactBodyGroups");
    let structure_groups = non_generated_array(function_clones, "structureGroups");
    let signature_groups = non_generated_array(function_clones, "signatureGroups");
    let near_function_candidates = non_generated_array(function_clones, "nearFunctionCandidates");
    let mut candidate_identities = BTreeSet::<String>::new();
    for group in structure_groups
        .iter()
        .chain(signature_groups.iter())
        .chain(near_function_candidates.iter())
    {
        if let Some(identities) = group.get("identities").and_then(Value::as_array) {
            for identity in identities.iter().filter_map(Value::as_str) {
                candidate_identities.insert(identity.to_string());
            }
        }
    }
    let has_candidates = !exact_groups.is_empty()
        || !structure_groups.is_empty()
        || !signature_groups.is_empty()
        || !near_function_candidates.is_empty();

    json!({
        "gate": if has_candidates { "watch" } else { "ok" },
        "available": true,
        "exactBodyGroups": exact_groups.len(),
        "structureGroupCandidates": structure_groups.len(),
        "signatureGroupCandidates": signature_groups.len(),
        "nearFunctionCandidates": near_function_candidates.len(),
        "generatedOnlyExactGroups": generated_only_count(function_clones, "exactBodyGroups"),
        "generatedOnlyStructureGroups": generated_only_count(function_clones, "structureGroups"),
        "generatedOnlySignatureGroups": generated_only_count(function_clones, "signatureGroups"),
        "generatedOnlyNearFunctionCandidates": generated_only_count(function_clones, "nearFunctionCandidates"),
        "candidateIdentityCount": candidate_identities.len(),
        "totalFunctionFacts": function_clones.get("facts").and_then(Value::as_array).map_or(0, Vec::len),
        "functionCloneIndexComplete": value_at(function_clones, &["meta", "complete"]).and_then(Value::as_bool).unwrap_or(true),
        "topExactGroups": exact_groups.into_iter().take(10).collect::<Vec<_>>(),
        "topStructureGroups": structure_groups.into_iter().take(10).collect::<Vec<_>>(),
        "topSignatureGroups": signature_groups.into_iter().take(10).collect::<Vec<_>>(),
        "topNearFunctionCandidates": near_function_candidates.into_iter().take(10).collect::<Vec<_>>(),
        "note": "Exact body, same-structure, same-signature, and near exported function cues only. Treat as review cues, not proof of semantic equivalence or an automatic merge.",
    })
}

fn c5_lint_enforcement(triage: Option<&Value>) -> Value {
    let Some(rules) = triage
        .and_then(|artifact| artifact.get("boundaries"))
        .and_then(Value::as_array)
    else {
        return unavailable("triage.json missing — run triage-repo.mjs first");
    };
    let has_boundary_rule = rules.iter().any(|rule| {
        matches!(
            rule.get("rule").and_then(Value::as_str),
            Some("no-restricted-imports")
                | Some("no-restricted-paths")
                | Some("eslint-plugin-boundaries")
        )
    });
    json!({
        "gate": if has_boundary_rule { "ok" } else { "watch" },
        "rulesDetected": rules.len(),
        "boundaryRulePresent": has_boundary_rule,
        "rules": rules,
    })
}

fn c7_barrel_amplification(barrels: Option<&Value>) -> Value {
    let Some(barrels) = barrels else {
        return unavailable("barrels.json missing — run check-barrel-discipline.mjs first");
    };
    if barrels.get("mode").and_then(Value::as_str) == Some("single-package") {
        return json!({
            "gate": "ok",
            "reason": "single-package repo — no workspace barrels to discipline",
        });
    }
    let mut by_package = Vec::<Value>::new();
    let mut worst_compliance = 1.0f64;
    if let Some(packages) = barrels.get("byPackage").and_then(Value::as_object) {
        for (pkg, data) in packages {
            let compliance = data
                .get("policyCompliance")
                .and_then(Value::as_str)
                .unwrap_or("");
            let compliance_num = parse_percent(compliance);
            if let Some(pct) = compliance_num {
                worst_compliance = worst_compliance.min(pct);
            }
            by_package.push(json!({
                "pkg": pkg,
                "rootImports": data.get("rootImports").cloned().unwrap_or(Value::Null),
                "subpathImports": data.get("subpathImports").cloned().unwrap_or(Value::Null),
                "total": data.get("total").cloned().unwrap_or(Value::Null),
                "compliance": data.get("policyCompliance").cloned().unwrap_or(Value::Null),
                "complianceNum": compliance_num.map_or(Value::Null, |value| json!(value)),
            }));
        }
    }
    let gate = if worst_compliance < 0.5 {
        "fix"
    } else if worst_compliance < 0.8 {
        "watch"
    } else {
        "ok"
    };
    json!({
        "gate": gate,
        "worstCompliance": round3(worst_compliance),
        "byPackage": by_package,
    })
}

fn e2_silent_catch(facts: &SilentCatchFacts) -> Value {
    let watch_count = facts.non_empty_anonymous_sites.len() + facts.unused_param_sites.len();
    let gate = if facts.sites.len() > 3 {
        "fix"
    } else if !facts.sites.is_empty() || watch_count > 0 {
        "watch"
    } else {
        "ok"
    };
    json!({
        "gate": gate,
        "analysis": facts.analysis,
        "count": facts.sites.len(),
        "emptyUndocumentedCount": facts.sites.len(),
        "parseErrors": facts.parse_errors,
        "sites": facts.sites,
        "documentedCount": facts.documented_sites.len(),
        "emptyDocumentedCount": facts.documented_sites.len(),
        "documentedSites": facts.documented_sites,
        "anonymousCount": facts.anonymous_sites.len(),
        "anonymousSites": facts.anonymous_sites,
        "nonEmptyAnonymousCount": facts.non_empty_anonymous_sites.len(),
        "nonEmptyAnonymousSites": facts.non_empty_anonymous_sites,
        "unusedParamCount": facts.unused_param_sites.len(),
        "unusedParamSites": facts.unused_param_sites,
    })
}

fn unavailable(reason: &str) -> Value {
    json!({
        "gate": "unknown",
        "available": false,
        "reason": reason,
    })
}

fn annotate(section_key: &str, result: Value, context_check: bool) -> Value {
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

fn not_computed_items() -> Value {
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

#[derive(Debug, Clone, Copy, Default)]
struct RoleBucket {
    big: usize,
    medium: usize,
    small: usize,
    total: usize,
}

impl RoleBucket {
    fn to_value(self) -> Value {
        json!({
            "big": self.big,
            "medium": self.medium,
            "small": self.small,
        })
    }

    fn to_role_value(self) -> Value {
        json!({
            "big": self.big,
            "medium": self.medium,
            "small": self.small,
            "total": self.total,
        })
    }
}

fn normalized_role(role: &str) -> String {
    match role {
        "test" | "script" | "production" => role.to_string(),
        _ => "production".to_string(),
    }
}

fn role_buckets_to_value(role_buckets: &BTreeMap<String, RoleBucket>) -> Value {
    let mut object = Map::new();
    for role in ["production", "test", "script"] {
        object.insert(
            role.to_string(),
            role_buckets
                .get(role)
                .copied()
                .unwrap_or_default()
                .to_role_value(),
        );
    }
    Value::Object(object)
}

fn function_entry_value(entry: &FunctionSizeEntry) -> Value {
    json!({
        "file": entry.file,
        "line": entry.line,
        "name": entry.name,
        "loc": entry.loc.max(1),
        "fileRole": normalized_role(&entry.file_role),
    })
}

fn entries_to_values(items: &[(usize, FunctionSizeEntry)], limit: usize) -> Vec<Value> {
    items
        .iter()
        .take(limit)
        .map(|(_, entry)| function_entry_value(entry))
        .collect()
}

fn entries_by_role(items: &[(usize, FunctionSizeEntry)]) -> Value {
    let mut object = Map::new();
    for role in ["production", "test", "script"] {
        object.insert(
            role.to_string(),
            Value::Array(
                items
                    .iter()
                    .filter(|(_, entry)| normalized_role(&entry.file_role) == role)
                    .take(10)
                    .map(|(_, entry)| function_entry_value(entry))
                    .collect(),
            ),
        );
    }
    Value::Object(object)
}

fn normalize_cross_submodule_edges(topology: &Value) -> (&'static str, Vec<CrossEdge>) {
    if let Some(edges) = topology
        .get("crossSubmoduleEdges")
        .and_then(Value::as_array)
    {
        return (
            "full-list",
            edges
                .iter()
                .filter_map(|edge| {
                    Some(CrossEdge {
                        from: edge.get("from")?.as_str()?.to_string(),
                        to: edge.get("to")?.as_str()?.to_string(),
                        count: edge.get("count").and_then(as_usize).unwrap_or(0),
                    })
                })
                .collect(),
        );
    }
    if let Some(edges) = topology.get("crossSubmoduleTop").and_then(Value::as_array) {
        return (
            "top-30",
            edges
                .iter()
                .filter_map(|edge| {
                    let text = edge.get("edge")?.as_str()?;
                    let (from, to) = text.split_once(" → ")?;
                    Some(CrossEdge {
                        from: from.to_string(),
                        to: to.to_string(),
                        count: edge.get("count").and_then(as_usize).unwrap_or(0),
                    })
                })
                .collect(),
        );
    }
    ("absent", Vec::new())
}

fn is_healthy_layered_cross_edge(edge: &CrossEdge) -> bool {
    (edge.to == "_lib" && matches!(edge.from.as_str(), "root" | "scripts" | "tests"))
        || (edge.from == "tests" && edge.to != "tests" && edge.to != "root")
}

fn cross_edge_value(edge: &CrossEdge) -> Value {
    json!({
        "from": edge.from,
        "to": edge.to,
        "count": edge.count,
    })
}

fn summarize_shape_group(
    hash: &str,
    identities: &[String],
    facts_by_identity: &BTreeMap<String, Value>,
) -> Option<Value> {
    let mut members = identities
        .iter()
        .filter_map(|identity| facts_by_identity.get(identity))
        .cloned()
        .collect::<Vec<_>>();
    if members.len() < 2 {
        return None;
    }
    members.sort_by(|a, b| {
        text_field(a, "ownerFile")
            .cmp(&text_field(b, "ownerFile"))
            .then_with(|| text_field(a, "exportedName").cmp(&text_field(b, "exportedName")))
    });
    let owner_files = unique_sorted_strings(members.iter().map(|m| text_field(m, "ownerFile")));
    let exported_names =
        unique_sorted_strings(members.iter().map(|m| text_field(m, "exportedName")));
    let generated_members = members
        .iter()
        .filter(|member| {
            member
                .get("generatedFile")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    let field_names = members.first().map(shape_field_names).unwrap_or_default();
    Some(json!({
        "hash": hash,
        "size": members.len(),
        "ownerFiles": owner_files,
        "exportedNames": exported_names,
        "generatedMembers": generated_members,
        "generatedOnly": generated_members == members.len(),
        "fieldNames": field_names,
        "identities": members.iter().filter_map(|m| m.get("identity").and_then(Value::as_str)).collect::<Vec<_>>(),
    }))
}

fn shape_group_rank(a: &Value, b: &Value) -> Ordering {
    let a_non_generated = !a
        .get("generatedOnly")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let b_non_generated = !b
        .get("generatedOnly")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    b_non_generated
        .cmp(&a_non_generated)
        .then_with(|| {
            b.get("size")
                .and_then(as_usize)
                .unwrap_or(0)
                .cmp(&a.get("size").and_then(as_usize).unwrap_or(0))
        })
        .then_with(|| text_field(a, "hash").cmp(&text_field(b, "hash")))
}

fn collect_near_shape_candidates(facts: &[Value]) -> Vec<Value> {
    let mut usable = facts
        .iter()
        .filter(|fact| {
            fact.get("identity").and_then(Value::as_str).is_some()
                && !fact
                    .get("generatedFile")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                && shape_field_names(fact).len() >= 2
        })
        .cloned()
        .collect::<Vec<_>>();
    usable.sort_by_key(|fact| text_field(fact, "identity"));

    let mut candidates = Vec::<Value>::new();
    for i in 0..usable.len() {
        for j in (i + 1)..usable.len() {
            if let Some(candidate) = summarize_near_shape_candidate(&usable[i], &usable[j]) {
                candidates.push(candidate);
            }
        }
    }
    candidates.sort_by(|a, b| {
        number_field(b, "score")
            .partial_cmp(&number_field(a, "score"))
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                number_field(b, "fieldJaccard")
                    .partial_cmp(&number_field(a, "fieldJaccard"))
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| identities_key(a).cmp(&identities_key(b)))
    });
    candidates.truncate(20);
    candidates
}

fn summarize_near_shape_candidate(a: &Value, b: &Value) -> Option<Value> {
    let a_fields = shape_field_names(a);
    let b_fields = shape_field_names(b);
    let shared_fields = set_intersection(&a_fields, &b_fields);
    if same_hash_pair(a, b) || shared_fields.len() < 2 {
        return None;
    }
    let field_jaccard = jaccard(&a_fields, &b_fields);
    let a_name_tokens = tokenize_shape_name(&text_field(a, "exportedName"));
    let b_name_tokens = tokenize_shape_name(&text_field(b, "exportedName"));
    let shared_name_tokens = set_intersection(&a_name_tokens, &b_name_tokens);
    let name_token_jaccard = jaccard(&a_name_tokens, &b_name_tokens);
    let same_directory =
        owner_dir(&text_field(a, "ownerFile")) == owner_dir(&text_field(b, "ownerFile"));
    let domain_cue = same_directory || !shared_name_tokens.is_empty();
    if !domain_cue {
        return None;
    }
    let nearly_same_fields = field_jaccard >= 0.5 && shared_fields.len() >= 2;
    let same_named_concept = !shared_name_tokens.is_empty() && field_jaccard >= 0.4;
    if !nearly_same_fields && !same_named_concept {
        return None;
    }
    let score = round3(
        (field_jaccard * 0.75)
            + (name_token_jaccard * 0.2)
            + if same_directory { 0.05 } else { 0.0 },
    );
    Some(json!({
        "score": score,
        "fieldJaccard": round3(field_jaccard),
        "nameTokenJaccard": round3(name_token_jaccard),
        "sameDirectory": same_directory,
        "identities": [text_field(a, "identity"), text_field(b, "identity")],
        "ownerFiles": [text_field(a, "ownerFile"), text_field(b, "ownerFile")],
        "exportedNames": [text_field(a, "exportedName"), text_field(b, "exportedName")],
        "sharedFieldNames": shared_fields,
        "leftOnlyFieldNames": set_diff(&a_fields, &b_fields),
        "rightOnlyFieldNames": set_diff(&b_fields, &a_fields),
        "sharedNameTokens": shared_name_tokens,
        "reason": "near exported type-shape review cue only; field/name overlap is not proof of duplication",
    }))
}

fn shape_field_names(fact: &Value) -> Vec<String> {
    let mut fields = fact
        .get("fields")
        .and_then(Value::as_array)
        .map(|fields| {
            fields
                .iter()
                .filter_map(|field| field.get("name").and_then(Value::as_str))
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    fields.sort();
    fields
}

fn tokenize_shape_name(name: &str) -> Vec<String> {
    let mut expanded = String::new();
    let mut previous: Option<char> = None;
    for ch in name.chars() {
        if let Some(prev) = previous {
            if (prev.is_ascii_lowercase() || prev.is_ascii_digit()) && ch.is_ascii_uppercase() {
                expanded.push(' ');
            }
        }
        expanded.push(ch);
        previous = Some(ch);
    }
    expanded
        .replace(['_', '-', '.'], " ")
        .to_ascii_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| token.len() >= 3 && !SHAPE_NAME_STOP_TOKENS.contains(token))
        .map(str::to_string)
        .collect()
}

const SHAPE_NAME_STOP_TOKENS: &[&str] = &[
    "type",
    "types",
    "interface",
    "interfaces",
    "model",
    "models",
    "state",
    "view",
    "data",
    "dto",
    "payload",
    "props",
    "options",
    "config",
    "request",
    "response",
    "result",
    "event",
    "item",
];

fn owner_dir(file: &str) -> String {
    let slash = file.replace('\\', "/");
    slash
        .rsplit_once('/')
        .map_or(String::new(), |(dir, _)| dir.to_string())
}

fn same_hash_pair(a: &Value, b: &Value) -> bool {
    a.get("hash").and_then(Value::as_str).is_some()
        && a.get("hash").and_then(Value::as_str) == b.get("hash").and_then(Value::as_str)
}

fn set_intersection(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().collect::<BTreeSet<_>>();
    left.iter()
        .filter(|item| right.contains(item))
        .cloned()
        .collect()
}

fn set_diff(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().collect::<BTreeSet<_>>();
    left.iter()
        .filter(|item| !right.contains(item))
        .cloned()
        .collect()
}

fn jaccard(left: &[String], right: &[String]) -> f64 {
    let left_set = left.iter().collect::<BTreeSet<_>>();
    let right_set = right.iter().collect::<BTreeSet<_>>();
    let union_len = left_set.union(&right_set).count();
    if union_len == 0 {
        return 0.0;
    }
    let intersection_len = left_set.intersection(&right_set).count();
    intersection_len as f64 / union_len as f64
}

fn non_generated_array(artifact: &Value, key: &str) -> Vec<Value> {
    artifact
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    !item
                        .get("generatedOnly")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn generated_only_count(artifact: &Value, key: &str) -> usize {
    artifact
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    item.get("generatedOnly")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

fn unique_sorted_strings<I>(items: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    items
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn text_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn number_field(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn identities_key(value: &Value) -> String {
    value
        .get("identities")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("|")
        })
        .unwrap_or_default()
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    Some(cursor)
}

fn as_usize(value: &Value) -> Option<usize> {
    value
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .or_else(|| {
            let number = value.as_f64()?;
            if number.is_finite() && number >= 0.0 {
                Some(number.floor() as usize)
            } else {
                None
            }
        })
}

fn parse_percent(value: &str) -> Option<f64> {
    let prefix = value.split_once('%').map_or(value, |(prefix, _)| prefix);
    prefix.parse::<f64>().ok().map(|number| number / 100.0)
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
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
