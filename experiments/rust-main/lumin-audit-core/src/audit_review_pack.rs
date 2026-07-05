use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const AUDIT_REVIEW_PACK_RENDER_REQUEST_SCHEMA_VERSION: &str =
    "lumin-audit-review-pack-render-request.v1";
pub const AUDIT_REVIEW_PACK_RENDER_RESULT_SCHEMA_VERSION: &str =
    "lumin-audit-review-pack-render-result.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditReviewPackRenderRequest {
    pub schema_version: String,
    #[serde(default)]
    pub manifest: Value,
    #[serde(default)]
    pub checklist_facts: Value,
    #[serde(default)]
    pub fix_plan: Value,
    #[serde(default)]
    pub topology: Value,
    #[serde(default)]
    pub discipline: Value,
    #[serde(default)]
    pub call_graph: Value,
    #[serde(default)]
    pub barrels: Value,
    #[serde(default)]
    pub shape_index: Value,
    #[serde(default)]
    pub function_clones: Value,
    #[serde(default)]
    pub dead_classify: Value,
    #[serde(default)]
    pub symbols: Value,
    #[serde(default)]
    pub module_reachability: Value,
    pub output_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditReviewPackRenderResult {
    pub schema_version: &'static str,
    pub path: String,
    pub bytes: usize,
}

#[derive(Debug, Default)]
struct OwnerSummary {
    annotated: i64,
    severe: i64,
    any_contaminated: i64,
    severe_examples: Vec<String>,
}

#[derive(Debug, Default)]
struct AnyContaminationSummary {
    present: bool,
    supported: bool,
    helper: OwnerSummary,
    type_owner: OwnerSummary,
    annotated: i64,
}

pub fn render_audit_review_pack_request(
    request: &AuditReviewPackRenderRequest,
) -> Result<(String, AuditReviewPackRenderResult)> {
    if request.schema_version != AUDIT_REVIEW_PACK_RENDER_REQUEST_SCHEMA_VERSION {
        bail!(
            "audit-review-pack-render: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    let markdown = render_audit_review_pack(request);
    let result = AuditReviewPackRenderResult {
        schema_version: AUDIT_REVIEW_PACK_RENDER_RESULT_SCHEMA_VERSION,
        path: request.output_path.clone(),
        bytes: markdown.len(),
    };
    Ok((markdown, result))
}

pub fn render_audit_review_pack(request: &AuditReviewPackRenderRequest) -> String {
    let lines = vec![
        "# Audit Review Pack".to_string(),
        String::new(),
        "Use this pack for full/deep repo review. It is a main-controller artifact brief, not a replacement for raw artifacts and not a subagent prompt.".to_string(),
        String::new(),
        format!("Scan range: {}.", scan_range(&request.manifest)),
        String::new(),
        "Controller rule: this file never calls external APIs or models. In Claude Code, the main assistant reads these lanes and decides whether the review needs built-in reviewer subagents. Use subagents for explicit full/deep/exhaustive review or when several independent code areas need a fresh pass; read locally for ordinary short chat answers.".to_string(),
        String::new(),
        "Recommended default for a full audit: read lanes 1-4 before finalizing the normal gentle summary. If using Claude Code subagents, translate each chosen lane into a codebase-reading assignment with concrete files, symbols, or hypotheses. Do not paste artifact/checklist lanes wholesale; the subagent should inspect code directly and report file:line evidence.".to_string(),
        String::new(),
        topology_lane(&request.topology, &request.call_graph, &request.barrels),
        type_lane(
            &request.discipline,
            &request.checklist_facts,
            &request.shape_index,
            &request.function_clones,
            &request.symbols,
            &request.manifest,
        ),
        dead_surface_lane(
            &request.fix_plan,
            &request.dead_classify,
            &request.manifest,
            &request.module_reachability,
        ),
        failure_lane(&request.checklist_facts, &request.manifest),
        "## Merge Instructions".to_string(),
        String::new(),
        "- Combine reviewer reports into at most three user-facing next actions.".to_string(),
        "- Preserve \"Keep As-Is\" decisions so low-ranked findings do not disappear.".to_string(),
        "- If reviewer lanes disagree, say what evidence differs instead of averaging their conclusions.".to_string(),
        "- Keep raw field paths in reserve unless the user asks for proof.".to_string(),
        String::new(),
    ];
    lines.join("\n")
}

fn value_to_string(value: Option<&Value>, fallback: &str) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        _ => fallback.to_string(),
    }
}

fn n(value: Option<&Value>) -> i64 {
    number_as_i64(value).unwrap_or(0)
}

fn n_or(value: Option<&Value>, fallback: i64) -> i64 {
    number_as_i64(value).unwrap_or(fallback)
}

fn number_as_i64(value: Option<&Value>) -> Option<i64> {
    match value {
        Some(Value::Number(number)) => Some(
            number
                .as_i64()
                .or_else(|| number.as_u64().and_then(|value| i64::try_from(value).ok()))
                .or_else(|| number.as_f64().map(|value| value as i64))
                .unwrap_or(0),
        ),
        _ => None,
    }
}

fn arr(value: Option<&Value>) -> &[Value] {
    value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn object(value: &Value) -> Option<&serde_json::Map<String, Value>> {
    value.as_object()
}

fn get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_object().and_then(|object| object.get(key))
}

fn plural(count: i64, singular: &str, plural_value: Option<&str>) -> String {
    if count == 1 {
        singular.to_string()
    } else {
        plural_value.unwrap_or(&format!("{singular}s")).to_string()
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn format_counter_object(counter: Option<&Value>) -> Option<String> {
    let object = counter?.as_object()?;
    let mut parts = object
        .iter()
        .filter_map(|(label, count)| Some((label.as_str(), number_as_i64(Some(count))?)))
        .collect::<Vec<_>>();
    parts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    if parts.is_empty() {
        None
    } else {
        Some(
            parts
                .into_iter()
                .map(|(label, count)| format!("{label} {count}"))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

fn format_rust_scope(summary: &Value) -> String {
    let Some(scope) = get(summary, "scanScope").filter(|value| value.is_object()) else {
        return String::new();
    };
    let tests = if get(scope, "includeTests").and_then(Value::as_bool) == Some(false) {
        "production files only"
    } else {
        "including tests"
    };
    let exclude_count = arr(get(scope, "exclude")).len();
    let excludes = if exclude_count > 0 {
        format!(
            ", {exclude_count} exclude {}",
            if exclude_count == 1 {
                "pattern"
            } else {
                "patterns"
            }
        )
    } else {
        String::new()
    };
    format!(" ({tests}{excludes})")
}

fn format_framework_resource_surface_counts(summary: Option<&Value>) -> Option<String> {
    let summary = summary?;
    let total = n(get(summary, "totalFilesWithSurfaces"));
    if total <= 0 {
        return None;
    }
    let lane_text = format_counter_object(get(summary, "byLane"));
    Some(format!(
        "Framework/resource surfaces: {total} files{}. Read manifest.json.frameworkResourceSurfaces and framework-resource-surfaces.json before treating import absence as deadness.",
        lane_text
            .map(|text| format!("; lanes {text}"))
            .unwrap_or_default()
    ))
}

fn format_dependency_hygiene_review_check(summary: Option<&Value>) -> Option<String> {
    let summary = summary?;
    if !summary.is_object() {
        return None;
    }
    let status = get(summary, "status")
        .and_then(Value::as_str)
        .unwrap_or("unavailable");
    if status != "complete" {
        return Some("Dependency hygiene review: evidence incomplete; do not infer dependency declaration absence. Read manifest.json.unusedDependencies and unused-deps.json.".to_string());
    }
    let review_unused = n(get(summary, "reviewUnusedCount"));
    let muted = n(get(summary, "mutedCount"));
    let confidence_limited = n(get(summary, "confidenceLimitedCount"));
    if review_unused <= 0 && confidence_limited <= 0 {
        return None;
    }
    Some(format!(
        "Dependency hygiene review: inspect unused-deps.json before changing package manifests. review-only={review_unused}; muted={muted}; confidence-limited={confidence_limited}."
    ))
}

fn format_sfc_evidence_review_check(summary: Option<&Value>) -> Option<String> {
    let summary = summary?;
    if !summary.is_object() {
        return None;
    }
    let empty = Value::Object(Default::default());
    let by_lane = get(summary, "byLane")
        .filter(|value| value.is_object())
        .unwrap_or(&empty);
    let total = n(get(summary, "totalEvidenceCount"));
    if total <= 0 {
        return None;
    }
    let lane_text = [
        (n(get(by_lane, "scriptImportConsumers")), "script-imports"),
        (n(get(by_lane, "scriptSrcReachability")), "script-src"),
        (n(get(by_lane, "styleAssetReferences")), "style-assets"),
        (n(get(by_lane, "templateComponentRefs")), "template-refs"),
        (
            n(get(by_lane, "globalComponentRegistrations")),
            "global-registrations",
        ),
        (
            n(get(by_lane, "generatedComponentManifests")),
            "generated-manifests",
        ),
        (
            n(get(by_lane, "frameworkConventionComponents")),
            "framework-conventions",
        ),
    ]
    .into_iter()
    .filter(|(count, _label)| *count > 0)
    .map(|(count, label)| format!("{label}={count}"))
    .collect::<Vec<_>>()
    .join("; ");
    Some(format!(
        "SFC evidence review: inspect manifest.json.sfcEvidence and SFC arrays in symbols.json before treating SFC absence as deadness. {}; review-only={}; sfc-scan-gap still applies.",
        if lane_text.is_empty() {
            "recorded-sfc-lanes".to_string()
        } else {
            lane_text
        },
        n(get(summary, "reviewOnlyEvidenceCount"))
    ))
}

fn format_unreachable_scc_review_check(module_reachability: &Value) -> Option<String> {
    let summary = get(module_reachability, "summary")?;
    let groups = n(get(summary, "unreachableStronglyConnectedComponents"));
    let files = n(get(summary, "unreachableStronglyConnectedFiles"));
    if groups <= 0 || files <= 0 {
        return None;
    }
    Some(format!(
        "Unreachable SCCs: {groups} {}, {files} {}. Read module-reachability.json.unreachableStronglyConnectedComponents[] before treating intra-cycle imports as liveness; use this as dead-file-group review evidence, not export SAFE_FIX proof.",
        plural(groups, "group", None),
        plural(files, "file", None)
    ))
}

fn scan_range(manifest: &Value) -> String {
    let empty = Value::Object(Default::default());
    let scan_range = get(manifest, "scanRange").unwrap_or(&empty);
    let langs = {
        let values = arr(get(scan_range, "languages"))
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>();
        if values.is_empty() {
            "unknown".to_string()
        } else {
            values.join(", ")
        }
    };
    let tests = if get(scan_range, "includeTests").and_then(Value::as_bool) == Some(false) {
        "production only"
    } else {
        "includes tests"
    };
    format!(
        "{} files; {langs}; {tests}",
        value_to_string(get(scan_range, "files"), "unknown")
    )
}

fn lane(title: &str, body: String) -> String {
    format!("## {title}\n\n{}\n", body.trim())
}

fn render_lane_prompt(
    title: &str,
    mission: String,
    artifacts: Vec<String>,
    checks: Vec<String>,
    report: &str,
) -> String {
    let mut lines = vec![
        "Controller-only lane. Read this in the main context as an artifact brief; do not paste the lane wholesale into a subagent.".to_string(),
        String::new(),
        format!("Role: {title}"),
        String::new(),
        format!("Mission: {mission}"),
        String::new(),
        format!("Artifacts for the controller to inspect first: {}", artifacts.join(", ")),
        String::new(),
        "Checks to convert into code questions:".to_string(),
    ];
    lines.extend(checks.into_iter().map(|check| format!("- {check}")));
    lines.extend([
        String::new(),
        format!("Report back with: {report}"),
        String::new(),
        "Subagent rule: if you dispatch a reviewer subagent, give it specific files, symbols, or hypotheses from this lane and ask it to read the codebase with file:line evidence. Do not ask the subagent to trust checklist or artifact summaries.".to_string(),
        String::new(),
        "Rules: cite artifact fields or file:line evidence; do not turn a gate value into a verdict; mark unknowns as \"not enough evidence yet\"; keep recommendations to the smallest useful slice.".to_string(),
    ]);
    lines.join("\n")
}

fn topology_lane(topology: &Value, call_graph: &Value, barrels: &Value) -> String {
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

fn type_lane(
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

fn dead_surface_lane(
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

fn failure_lane(checklist_facts: &Value, manifest: &Value) -> String {
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

fn summarize_any_contamination_owners(symbols: &Value) -> AnyContaminationSummary {
    if !symbols.is_object() {
        return AnyContaminationSummary::default();
    }
    let supported = symbols
        .pointer("/meta/supports/anyContamination")
        .and_then(Value::as_bool)
        == Some(true);
    let helper = summarize_owners(get(symbols, "helperOwnersByIdentity"));
    let type_owner = summarize_owners(get(symbols, "typeOwnersByIdentity"));
    AnyContaminationSummary {
        present: true,
        supported,
        annotated: helper.annotated + type_owner.annotated,
        helper,
        type_owner,
    }
}

fn summarize_owners(map: Option<&Value>) -> OwnerSummary {
    let mut rows = map
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .filter(|(_, owner)| owner.is_object())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    rows.sort_by(|left, right| left.0.cmp(right.0));
    let mut summary = OwnerSummary::default();
    for (identity, owner) in rows {
        let Some(annotation) = get(owner, "anyContamination") else {
            continue;
        };
        summary.annotated += 1;
        if has_label(annotation, "severely-any-contaminated") {
            summary.severe += 1;
            if summary.severe_examples.len() < 3 {
                summary.severe_examples.push(identity.clone());
            }
        }
        if has_label(annotation, "any-contaminated")
            || get(annotation, "label").and_then(Value::as_str) == Some("severely-any-contaminated")
        {
            summary.any_contaminated += 1;
        }
    }
    summary
}

fn has_label(annotation: &Value, label: &str) -> bool {
    arr(get(annotation, "labels"))
        .iter()
        .any(|value| value.as_str() == Some(label))
        || get(annotation, "label").and_then(Value::as_str) == Some(label)
}

fn example_text(summary: &AnyContaminationSummary) -> String {
    let examples = summary
        .type_owner
        .severe_examples
        .iter()
        .map(|id| format!("type {id}"))
        .chain(
            summary
                .helper
                .severe_examples
                .iter()
                .map(|id| format!("helper {id}")),
        )
        .take(3)
        .collect::<Vec<_>>();
    if examples.is_empty() {
        String::new()
    } else {
        format!(" Examples: {}.", examples.join("; "))
    }
}

fn format_any_contamination_review_check(symbols: &Value) -> String {
    let summary = summarize_any_contamination_owners(symbols);
    if !summary.present {
        return "Identity-level anyContamination: symbols.json not loaded in this lane. If symbols.json was produced, inspect helperOwnersByIdentity/typeOwnersByIdentity before semantic reuse claims.".to_string();
    }
    if !summary.supported {
        return "Identity-level anyContamination: producer capability is not available; do not claim contaminated identities are clean.".to_string();
    }
    if summary.annotated == 0 {
        return "Identity-level anyContamination: measured clean for exported owners. Keep this separate from occurrence-level discipline totals.".to_string();
    }
    format!(
        "Identity-level anyContamination: {} severe type {}, {} severe helper {}; {} any-contaminated type {}, {} helper {}. Inspect symbols.json owner maps before shape/reuse recommendations.{}",
        summary.type_owner.severe,
        plural(summary.type_owner.severe, "owner", None),
        summary.helper.severe,
        plural(summary.helper.severe, "owner", None),
        summary.type_owner.any_contaminated,
        plural(summary.type_owner.any_contaminated, "owner", None),
        summary.helper.any_contaminated,
        plural(summary.helper.any_contaminated, "owner", None),
        example_text(&summary)
    )
}

fn format_blocked_candidate_hints(hints: Option<&Value>) -> Option<String> {
    let parts = arr(hints)
        .iter()
        .take(3)
        .filter_map(|hint| {
            let target = get(hint, "candidatePath")
                .or_else(|| get(hint, "affectedPackageScope"))
                .and_then(Value::as_str)?;
            let specifier = get(hint, "specifier").and_then(Value::as_str)?;
            let reason = get(hint, "reason").and_then(Value::as_str)?;
            Some(format!("{target} via {specifier} ({reason})"))
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

fn format_distribution_list(
    items: Option<&Value>,
    label_key: &str,
    nested_key: &str,
) -> Option<String> {
    let parts = arr(items)
        .iter()
        .take(3)
        .filter_map(|item| {
            let label = get(item, label_key).and_then(Value::as_str)?;
            let count = n(get(item, "count"));
            if count == 0 {
                return None;
            }
            let nested = format_counter_object(get(item, nested_key));
            Some(format!(
                "{label} {count}{}",
                nested.map(|text| format!(" ({text})")).unwrap_or_default()
            ))
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

fn format_blocked_candidate_hint_distribution(resolver_diagnostics: &Value) -> Option<String> {
    let reason_text = format_distribution_list(
        get(resolver_diagnostics, "blockedCandidateHintReasonCounts"),
        "reason",
        "families",
    );
    let family_text = format_distribution_list(
        get(resolver_diagnostics, "blockedCandidateHintFamilyCounts"),
        "family",
        "reasons",
    );
    let parts = [
        reason_text.map(|text| format!("reasons {text}")),
        family_text.map(|text| format!("families {text}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}
