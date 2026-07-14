use serde_json::Value;

use super::review_checks::{
    format_any_contamination_review_check, format_blocked_candidate_hint_distribution,
    format_blocked_candidate_hints, format_dependency_hygiene_review_check,
    format_framework_resource_surface_counts, format_sfc_evidence_review_check,
    format_unreachable_scc_review_check,
};
use super::support::{
    arr, format_rust_scope, get, lane, n, n_or, object, render_lane_prompt, value_to_string, yes_no,
};

pub(super) fn topology_lane(topology: &Value, call_graph: &Value, barrels: &Value) -> String {
    let scc_count = n_or(
        get(get(topology, "summary").unwrap_or(&Value::Null), "sccCount"),
        arr(get(topology, "sccs")).len() as i64,
    );
    let semi_dead = n_or(
        get(
            get(call_graph, "summary").unwrap_or(&Value::Null),
            "semiDead",
        ),
        arr(get(call_graph, "semiDeadList")).len() as i64,
    );
    let barrel_keys = object(barrels)
        .map(|object| {
            object
                .keys()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "unknown".to_string());
    lane(
        "Lane 1 — Topology And Flow Review",
        render_lane_prompt(
            "Topology reviewer",
            "Find cross-file structure risks the short summary might hide: runtime cycles, one-way boundary breaks, barrel amplification, and semi-dead import clusters.".to_string(),
            vec![
                "manifest.json".to_string(),
                "topology.json".to_string(),
                "call-graph.json".to_string(),
                "barrels.json".to_string(),
            ],
            vec![
                format!("Runtime SCC count from topology: {scc_count}. If non-zero, inspect the largest SCC before any local cleanup."),
                format!("Semi-dead import count from call graph: {semi_dead}. Screen framework/test conventions before calling an import removable."),
                format!("Barrel evidence present: {} ({}). Treat barrel findings as review cues, not automatic refactors.", yes_no(barrels.is_object()), barrel_keys),
            ],
            "Already stable boundary facts, top one or two cross-file risks, and the smallest verification command after a fix.",
        ),
    )
}

pub(super) fn type_lane(
    discipline: &Value,
    checklist_facts: &Value,
    shape_index: &Value,
    function_clones: &Value,
    symbols: &Value,
    manifest: &Value,
) -> String {
    let totals = get(discipline, "totals").unwrap_or(&Value::Null);
    let escape_count = n(get(totals, ":any"))
        + n(get(totals, "as any"))
        + n(get(totals, "as unknown as"))
        + n(get(totals, "@ts-ignore"))
        + n(get(totals, "@ts-expect-error"))
        + n(get(totals, "@ts-nocheck"))
        + n(get(totals, "jsdoc-any"));
    let shape_drift = get(checklist_facts, "B1B2_shape_drift").unwrap_or(&Value::Null);
    let duplicate = get(checklist_facts, "B1_duplicate_implementation").unwrap_or(&Value::Null);
    let function_meta = get(function_clones, "meta").unwrap_or(&Value::Null);
    let rust_analysis = get(manifest, "rustAnalysis").unwrap_or(&Value::Null);
    let rust_artifact_available = get(rust_analysis, "status").and_then(Value::as_str)
        == Some("complete")
        && get(rust_analysis, "available").and_then(Value::as_bool) == Some(true);
    let mut artifacts = vec![
        "discipline.json".to_string(),
        "shape-index.json".to_string(),
        "function-clones.json".to_string(),
        "checklist-facts.json".to_string(),
        "symbols.json".to_string(),
    ];
    if rust_artifact_available {
        artifacts.push("rust-analyzer-health.latest.json".to_string());
    }
    let rust_evidence_mission = if rust_artifact_available {
        "Use rust-analyzer-health.latest.json, not JS/TS clone or shape artifacts, for Rust files."
    } else {
        "Rust analyzer evidence is not available for this run; JS/TS clone and shape artifacts are not Rust evidence."
    };
    lane(
        "Lane 2 — Types, Shapes, And Contract Review",
        render_lane_prompt(
            "Type and shape reviewer",
            format!("Look for JS/TS type-boundary and helper-shape drift that requires semantic judgment: repeated exported shapes, same-structure and near-function clone cues, and concentrated any/ignore-style escapes. {rust_evidence_mission}"),
            artifacts,
            vec![
                format!("Type escape total to screen: {escape_count}. Prioritize clusters over scattered one-offs."),
                format_any_contamination_review_check(symbols),
                format!("JS/TS exact exported shape groups: {}; near-shape review cues: {}; raw shape facts: {}. Do not use shape-index.json as Rust shape evidence.",
                    n(get(shape_drift, "exactDuplicateGroups")),
                    n(get(shape_drift, "nearShapeCandidateCount")),
                    arr(get(shape_index, "facts")).len(),
                ),
                format!("JS/TS function clone cues: exact body groups {}; same-structure groups {}; same-signature groups {}; near-function cues {}. Read source before calling them semantic duplicates.",
                    n_or(get(duplicate, "exactBodyGroups"), n(get(function_meta, "exactBodyGroupCount"))),
                    n_or(get(duplicate, "structureGroupCandidates"), n(get(function_meta, "structureGroupCount"))),
                    n_or(get(duplicate, "signatureGroupCandidates"), n(get(function_meta, "signatureGroupCount"))),
                    n_or(get(duplicate, "nearFunctionCandidates"), n(get(function_meta, "nearFunctionCandidateCount"))),
                ),
                if rust_artifact_available {
                    format!("Rust analyzer artifact available for {} file(s){}. Use rust-analyzer-health.latest.json for Rust shape, signature, clone, and syntax review cues.",
                        n(get(rust_analysis, "files")),
                        format_rust_scope(rust_analysis)
                    )
                } else {
                    let requested_status = if get(rust_analysis, "requested").and_then(Value::as_bool) == Some(true) {
                        format!(" ({})", get(rust_analysis, "status").and_then(Value::as_str).unwrap_or("not-run"))
                    } else {
                        String::new()
                    };
                    format!("Rust analyzer artifact not available in this run{requested_status}; keep Rust shape/clone claims limited to manifest blind-zone evidence.")
                },
                "For near-shape or semantic duplication, read the cited declarations before recommending a merge.".to_string(),
            ],
            "One type/shape theme worth smoothing, anything likely intentional, and what evidence is still missing.",
        ),
    )
}

pub(super) fn dead_surface_lane(
    fix_plan: &Value,
    dead_classify: &Value,
    manifest: &Value,
    module_reachability: &Value,
) -> String {
    let summary = get(fix_plan, "summary").unwrap_or(&Value::Null);
    let excluded = get(
        get(dead_classify, "summary").unwrap_or(&Value::Null),
        "excluded",
    )
    .and_then(Value::as_object);
    let excluded_text = excluded
        .map(|object| {
            object
                .iter()
                .take(4)
                .map(|(key, value)| format!("{key}: {}", value_to_string(Some(value), "0")))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "none recorded".to_string());
    let resolver = get(manifest, "resolverDiagnostics").unwrap_or(&Value::Null);
    let blocked_count = n(get(resolver, "blockedCandidateHintCount"));
    let sample_limit = n(get(resolver, "blockedCandidateHintSampleLimit"));
    let blocked_hints = format_blocked_candidate_hints(get(resolver, "blockedCandidateHints"));
    let blocked_distribution = format_blocked_candidate_hint_distribution(resolver);
    let resolver_blocked_hint = (blocked_count > 0).then(|| {
        format!(
            "Resolver blocked absence hints: {blocked_count}{}{}. Read manifest.json.resolverDiagnostics.blockedCandidateHints and resolver-diagnostics.json.blockedCandidateHints before treating affected exports as absent.",
            if sample_limit > 0 {
                format!("; manifest sample limit {sample_limit}")
            } else {
                String::new()
            },
            blocked_hints
                .map(|text| format!("; examples: {text}"))
                .unwrap_or_default()
        )
    });
    let resolver_blocked_distribution = blocked_distribution.map(|text| {
        format!("Resolver blocked absence distribution: {text}. Read manifest.json.resolverDiagnostics.blockedCandidateHintReasonCounts and manifest.json.resolverDiagnostics.blockedCandidateHintFamilyCounts before opening the full hint list.")
    });
    let dependency_check =
        format_dependency_hygiene_review_check(get(manifest, "unusedDependencies"));
    let mut artifacts = vec![
        "fix-plan.json".to_string(),
        "dead-classify.json".to_string(),
        "symbols.json".to_string(),
        "manifest.json".to_string(),
        "module-reachability.json".to_string(),
    ];
    if dependency_check.is_some() {
        artifacts.push("unused-deps.json".to_string());
    }
    let mut checks = vec![
        format!("Tier summary: SAFE_FIX {}, REVIEW_FIX {}, DEGRADED {}, MUTED {}. Do not present REVIEW_FIX as removable without screening.",
            n(get(summary, "SAFE_FIX")),
            n(get(summary, "REVIEW_FIX")),
            n(get(summary, "DEGRADED")),
            n(get(summary, "MUTED")),
        ),
        format!("Muted/excluded families observed: {excluded_text}. Translate them into plain language for the user."),
    ];
    checks.extend(resolver_blocked_distribution);
    checks.extend(resolver_blocked_hint);
    checks.extend(format_framework_resource_surface_counts(get(
        manifest,
        "frameworkResourceSurfaces",
    )));
    checks.extend(dependency_check);
    checks.extend(format_sfc_evidence_review_check(get(
        manifest,
        "sfcEvidence",
    )));
    checks.extend(format_unreachable_scc_review_check(module_reachability));
    checks.push("For each visible cleanup candidate, check whether it is exported through package/API/declaration/test-only surfaces before recommending a change.".to_string());
    lane(
        "Lane 3 — Dead Export And Public Surface Review",
        render_lane_prompt(
            "Dead-export/public-surface reviewer",
            "Separate real cleanup from public surface, declaration/type-surface, framework, generated, config, and test-consumer false positives.".to_string(),
            artifacts,
            checks,
            "Which candidates are safe to leave alone, which need review together, and at most one action-ready cleanup slice.",
        ),
    )
}

pub(super) fn failure_lane(checklist_facts: &Value, manifest: &Value) -> String {
    let e2 = get(checklist_facts, "E2_silent_catch").unwrap_or(&Value::Null);
    let blind_zones = arr(get(manifest, "blindZones")).len();
    let rust_analysis = get(manifest, "rustAnalysis").unwrap_or(&Value::Null);
    let rust_artifact_available = get(rust_analysis, "status").and_then(Value::as_str)
        == Some("complete")
        && get(rust_analysis, "available").and_then(Value::as_bool) == Some(true);
    let mut artifacts = vec![
        "checklist-facts.json".to_string(),
        "manifest.json".to_string(),
        "discipline.json".to_string(),
    ];
    if rust_artifact_available {
        artifacts.push("rust-analyzer-health.latest.json".to_string());
    }
    lane(
        "Lane 4 — Failure Handling And Blind-Zone Review",
        render_lane_prompt(
            "Failure-handling reviewer",
            "Check whether error-handling and measurement blind zones could make the main summary too optimistic.".to_string(),
            artifacts,
            vec![
                format!("Silent catch count: {}; non-empty anonymous catches: {}; unused catch params: {}.",
                    n(get(e2, "count")),
                    n(get(e2, "nonEmptyAnonymousCount")),
                    n(get(e2, "unusedParamCount")),
                ),
                format!("Blind zones recorded in manifest: {blind_zones}. Treat any blind zone as a limit on absence/removal claims."),
                if rust_artifact_available {
                    format!("Rust analyzer artifact available for {} file(s){}. Read rust-analyzer-health.latest.json before making Rust syntax, clone, dead-definition, or absence claims.",
                        n(get(rust_analysis, "files")),
                        format_rust_scope(rust_analysis),
                    )
                } else {
                    let requested_status = if get(rust_analysis, "requested").and_then(Value::as_bool) == Some(true) {
                        format!(" ({})", get(rust_analysis, "status").and_then(Value::as_str).unwrap_or("not-run"))
                    } else {
                        String::new()
                    };
                    format!("Rust analyzer artifact not available in this run{requested_status}; keep Rust findings limited to manifest blind-zone evidence.")
                },
                "If a catch pattern is intentional, recommend documenting the intent rather than changing behavior blindly.".to_string(),
            ],
            "Failure-handling strengths, one watch item if present, and exact limits on what this audit could not prove.",
        ),
    )
}
