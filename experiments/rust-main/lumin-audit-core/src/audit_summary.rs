use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const AUDIT_SUMMARY_RENDER_REQUEST_SCHEMA_VERSION: &str =
    "lumin-audit-summary-render-request.v1";
pub const AUDIT_SUMMARY_RENDER_RESULT_SCHEMA_VERSION: &str = "lumin-audit-summary-render-result.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSummaryRenderRequest {
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
    pub function_clones: Value,
    #[serde(default)]
    pub symbols: Value,
    #[serde(default)]
    pub module_reachability: Value,
    pub output_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSummaryRenderResult {
    pub schema_version: &'static str,
    pub path: String,
    pub bytes: usize,
    pub preview: Option<String>,
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

pub fn render_audit_summary_request(
    request: &AuditSummaryRenderRequest,
) -> Result<(String, AuditSummaryRenderResult)> {
    if request.schema_version != AUDIT_SUMMARY_RENDER_REQUEST_SCHEMA_VERSION {
        bail!(
            "audit-summary-render: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    let markdown = render_audit_summary(request);
    let result = AuditSummaryRenderResult {
        schema_version: AUDIT_SUMMARY_RENDER_RESULT_SCHEMA_VERSION,
        path: request.output_path.clone(),
        bytes: markdown.len(),
        preview: render_summary_console_preview(&markdown),
    };
    Ok((markdown, result))
}

pub fn render_audit_summary(request: &AuditSummaryRenderRequest) -> String {
    let command_result = summarize_lifecycle_command(&request.manifest);
    let mut lines = vec![
        "# Audit Artifact Brief".to_string(),
        String::new(),
        "This file is an orientation map, not a recommendation engine. Do not paste it as the final user answer. Read the raw artifacts and write the chat summary yourself.".to_string(),
        String::new(),
        format!(
            "Generated: {}",
            pointer_string(&request.manifest, "/meta/generated", "unknown")
        ),
        format!(
            "Profile: {}",
            get(&request.manifest, "profile")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        ),
        format!("Scan range: {}", summarize_scan_range(&request.manifest)),
        format!("Confidence: {}", summarize_confidence(&request.manifest)),
        String::new(),
    ];

    if !command_result.is_empty() {
        lines.extend(["## Command Result".to_string(), String::new()]);
        lines.extend(command_result);
        lines.push(String::new());
    }

    lines.extend([
        "## Read First".to_string(),
        String::new(),
        "- Start with `manifest.json` for scan range, confidence, blind zones, and lifecycle command status.".to_string(),
        "- Then read the raw artifact for the user question: symbols, topology, discipline, checklist, fix-plan, call-graph, barrels, shape-index, or function-clones.".to_string(),
        "- Curate the final chat answer from those artifacts. Do not inherit ordering from this brief.".to_string(),
        String::new(),
        "## Measured Cues (Unranked)".to_string(),
        String::new(),
    ]);
    lines.extend(measured_cue_lines(request));
    lines.extend([String::new(), "## Artifact Map".to_string(), String::new()]);
    lines.extend(artifact_map_lines(request));
    lines.push(String::new());
    lines.extend(living_audit_lines(&request.manifest));
    lines.extend(expansion_hint_lines(&request.manifest));
    lines.extend([
        "## Guardrails".to_string(),
        String::new(),
        "- Raw artifacts are authoritative; this brief is only a map of where to look.".to_string(),
        "- Gate values are triggers, not verdicts.".to_string(),
        "- Counts alone do not define priority. Re-rank by the user request, repo context, file role, and evidence quality.".to_string(),
        "- For vibe-coder chat, answer with what is stable, what to inspect next, what to leave alone, and how to verify.".to_string(),
        String::new(),
    ]);
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

fn pointer_string(value: &Value, pointer: &str, fallback: &str) -> String {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}

fn n(value: Option<&Value>) -> i64 {
    n_or(value, 0)
}

fn n_or(value: Option<&Value>, fallback: i64) -> i64 {
    match value {
        Some(Value::Number(number)) => number
            .as_i64()
            .or_else(|| number.as_u64().and_then(|value| i64::try_from(value).ok()))
            .or_else(|| number.as_f64().map(|value| value as i64))
            .unwrap_or(fallback),
        _ => fallback,
    }
}

fn f64_value(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn pct(value: Option<&Value>) -> String {
    let Some(value) = f64_value(value) else {
        return "unknown".to_string();
    };
    if value < 0.01 {
        format!("{:.2}%", value * 100.0)
    } else {
        format!("{:.1}%", value * 100.0)
    }
}

fn arr(value: Option<&Value>) -> &[Value] {
    value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn object(value: Option<&Value>) -> Option<&serde_json::Map<String, Value>> {
    value?.as_object()
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

fn format_counter_object(counter: Option<&Value>) -> Option<String> {
    let mut parts = object(counter)?
        .iter()
        .map(|(label, count)| (label.as_str(), n_or(Some(count), i64::MIN)))
        .filter(|(_, count)| *count != i64::MIN)
        .collect::<Vec<_>>();
    parts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    (!parts.is_empty()).then(|| {
        parts
            .into_iter()
            .map(|(label, count)| format!("{label} {count}"))
            .collect::<Vec<_>>()
            .join(", ")
    })
}

fn format_unresolved_reason_counts(reasons: Option<&Value>, limit: usize) -> Option<String> {
    let mut items = match reasons {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| {
                Some((
                    get(item, "reason")?.as_str()?.to_string(),
                    n_or(get(item, "count"), i64::MIN),
                ))
            })
            .filter(|(_, count)| *count != i64::MIN)
            .collect::<Vec<_>>(),
        Some(Value::Object(object)) => object
            .iter()
            .map(|(reason, count)| (reason.clone(), n_or(Some(count), i64::MIN)))
            .filter(|(_, count)| *count != i64::MIN)
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    if matches!(reasons, Some(Value::Object(_))) {
        items.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    }
    let parts = items
        .into_iter()
        .take(limit)
        .filter(|(reason, _)| !reason.is_empty())
        .map(|(reason, count)| format!("{reason} {count}"))
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join(", "))
}

fn format_top_unresolved_roots(roots: Option<&Value>, limit: usize) -> Option<String> {
    let parts = arr(roots)
        .iter()
        .take(limit)
        .filter_map(|root| {
            let name = get(root, "specifierRoot").and_then(Value::as_str)?;
            let count = n_or(get(root, "count"), i64::MIN);
            if count == i64::MIN {
                return None;
            }
            let reasons = format_unresolved_reason_counts(get(root, "reasons"), 3);
            Some(format!(
                "{name} {count}{}",
                reasons
                    .map(|reasons| format!(" ({reasons})"))
                    .unwrap_or_default()
            ))
        })
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("; "))
}

fn format_top_affected_package_scopes(scopes: Option<&Value>, limit: usize) -> Option<String> {
    let parts = arr(scopes)
        .iter()
        .take(limit)
        .filter_map(|scope| {
            let name = get(scope, "affectedPackageScope").and_then(Value::as_str)?;
            let count = n_or(get(scope, "count"), i64::MIN);
            (count != i64::MIN).then(|| format!("{name} {count}"))
        })
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("; "))
}

fn format_framework_resource_surface_counts(summary: Option<&Value>) -> Option<String> {
    let summary = summary?;
    let total = n(get(summary, "totalFilesWithSurfaces"));
    if total <= 0 {
        return None;
    }
    let lane_text = format_counter_object(get(summary, "byLane"));
    let confidence_text = format_counter_object(get(summary, "byConfidence"));
    let examples = arr(get(summary, "topExamples"))
        .iter()
        .take(2)
        .filter_map(|example| {
            let file = get(example, "file").and_then(Value::as_str)?;
            let reasons = arr(get(example, "reasons"))
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>();
            Some(format!(
                "{file}{}",
                if reasons.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", reasons.join(", "))
                }
            ))
        })
        .collect::<Vec<_>>()
        .join("; ");
    let parts = [
        Some(format!("{total} files")),
        lane_text.map(|text| format!("lanes {text}")),
        confidence_text.map(|text| format!("confidence {text}")),
        (!examples.is_empty()).then(|| format!("examples: {examples}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    Some(parts.join("; "))
}

fn format_dependency_hygiene_cue(summary: Option<&Value>) -> Option<String> {
    let summary = summary?;
    if !summary.is_object() {
        return None;
    }
    let status = get(summary, "status")
        .and_then(Value::as_str)
        .unwrap_or("unavailable");
    if status != "complete" {
        return Some("Dependency hygiene: evidence incomplete; do not infer dependency declaration absence. Read `manifest.json.unusedDependencies` and `unused-deps.json`.".to_string());
    }
    let review_unused = n(get(summary, "reviewUnusedCount"));
    let muted = n(get(summary, "mutedCount"));
    let confidence_limited = n(get(summary, "confidenceLimitedCount"));
    if review_unused <= 0 && confidence_limited <= 0 {
        return None;
    }
    let review_verb = if review_unused == 1 { "needs" } else { "need" };
    let confidence_text = if confidence_limited > 0 {
        format!(
            "; {confidence_limited} confidence-limited {}",
            plural(confidence_limited, "declaration", None)
        )
    } else {
        String::new()
    };
    Some(format!(
        "Dependency hygiene: {review_unused} review-only dependency {} {review_verb} inspection; {muted} muted {}{confidence_text}. Read `manifest.json.unusedDependencies` and `unused-deps.json` before changing package manifests.",
        plural(review_unused, "declaration", None),
        plural(muted, "explanation", None)
    ))
}

fn format_rust_analysis_cue(summary: Option<&Value>) -> Option<String> {
    let summary = summary?;
    if !summary.is_object() {
        return None;
    }
    if get(summary, "status").and_then(Value::as_str) != Some("complete")
        || get(summary, "available").and_then(Value::as_bool) != Some(true)
    {
        if get(summary, "requested").and_then(Value::as_bool) == Some(true) {
            return Some(format!(
                "Rust analyzer: {}{}. Do not use JS/TS artifacts for Rust absence claims.",
                get(summary, "status")
                    .and_then(Value::as_str)
                    .unwrap_or("not-run"),
                get(summary, "reason")
                    .and_then(Value::as_str)
                    .map(|reason| format!(" ({reason})"))
                    .unwrap_or_default()
            ));
        }
        return None;
    }
    let scope_text = format_rust_scope(summary);
    let clone_parts = [
        format!(
            "exact {}",
            n(get(summary, "syntaxFunctionCloneExactBodyGroups"))
        ),
        format!(
            "structure {}",
            n(get(summary, "syntaxFunctionCloneStructureGroups"))
        ),
        format!(
            "signature {}",
            n(get(summary, "syntaxFunctionCloneSignatureGroups"))
        ),
        format!(
            "near {}",
            n(get(summary, "syntaxFunctionCloneNearCandidates"))
        ),
    ]
    .join(", ");
    Some(format!(
        "Rust analyzer: {} files{scope_text}, review signals {}, opaque surfaces {}, clone cues {clone_parts}. Read `rust-analyzer-health.latest.json` before making Rust findings.",
        n(get(summary, "files")),
        n(get(summary, "syntaxReviewSignals")),
        n(get(summary, "syntaxReviewOpaqueSurfaces")),
    ))
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
    let exclude_count = arr(get(scope, "exclude")).len() as i64;
    let excludes = if exclude_count > 0 {
        format!(
            ", {exclude_count} exclude {}",
            plural(exclude_count, "pattern", None)
        )
    } else {
        String::new()
    };
    format!(" ({tests}{excludes})")
}

fn format_sfc_evidence_cue(summary: Option<&Value>) -> Option<String> {
    let summary = summary?;
    if !summary.is_object() {
        return None;
    }
    let empty = Value::Object(Default::default());
    let by_lane = get(summary, "byLane")
        .filter(|value| value.is_object())
        .unwrap_or(&empty);
    let total = n(get(summary, "totalEvidenceCount"));
    let review_only = n(get(summary, "reviewOnlyEvidenceCount"));
    let script_consumers = n_or(
        get(summary, "scriptImportConsumerCount"),
        n(get(by_lane, "scriptImportConsumers")),
    );
    let reachability_only = n_or(
        get(summary, "reachabilityOnlyCount"),
        n(get(by_lane, "scriptSrcReachability")),
    );
    if total <= 0 {
        return None;
    }
    let lane_text = [
        (script_consumers, "script imports"),
        (reachability_only, "script-src reachability"),
        (n(get(by_lane, "styleAssetReferences")), "style assets"),
        (n(get(by_lane, "templateComponentRefs")), "template refs"),
        (
            n(get(by_lane, "globalComponentRegistrations")),
            "global registrations",
        ),
        (
            n(get(by_lane, "generatedComponentManifests")),
            "generated manifests",
        ),
        (
            n(get(by_lane, "frameworkConventionComponents")),
            "framework conventions",
        ),
    ]
    .into_iter()
    .filter(|(count, _)| *count > 0)
    .map(|(count, label)| format!("{label} {count}"))
    .collect::<Vec<_>>()
    .join(", ");
    Some(format!(
        "SFC evidence: {total} {} across {}; {review_only} review-only {}. Read `manifest.json.sfcEvidence` and SFC arrays in `symbols.json`; review-only SFC lanes are not fan-in or action-tier proof, and sfc-scan-gap still applies.",
        plural(total, "record", None),
        if lane_text.is_empty() {
            "recorded SFC lanes"
        } else {
            &lane_text
        },
        plural(review_only, "record", None),
    ))
}

fn format_unreachable_scc_cue(module_reachability: &Value) -> Option<String> {
    let groups = n(module_reachability.pointer("/summary/unreachableStronglyConnectedComponents"));
    let files = n(module_reachability.pointer("/summary/unreachableStronglyConnectedFiles"));
    if groups <= 0 || files <= 0 {
        return None;
    }
    Some(format!(
        "Unreachable SCCs: {groups} {}, {files} {}",
        plural(groups, "group", None),
        plural(files, "file", None)
    ))
}

fn format_top_specifiers(specifiers: Option<&Value>, limit: usize) -> Option<String> {
    let parts = arr(specifiers)
        .iter()
        .take(limit)
        .filter_map(|item| {
            let specifier = get(item, "specifier").and_then(Value::as_str)?;
            let count = n_or(get(item, "count"), i64::MIN);
            (count != i64::MIN).then(|| format!("{specifier} {count}"))
        })
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join(", "))
}

fn format_generated_consumer_blind_zone_scopes(
    groups: Option<&Value>,
    limit: usize,
) -> Option<String> {
    let parts = arr(groups)
        .iter()
        .take(limit)
        .filter_map(|group| {
            let scope = get(group, "scopePackageRoot").and_then(Value::as_str)?;
            let count = n_or(get(group, "count"), i64::MIN);
            if count == i64::MIN {
                return None;
            }
            let status_text = format_counter_object(get(group, "statuses"));
            let specifier_text = format_top_specifiers(get(group, "topSpecifiers"), 2);
            let detail = [status_text, specifier_text]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("; ");
            Some(format!(
                "{scope} {count}{}",
                if detail.is_empty() {
                    String::new()
                } else {
                    format!(" ({detail})")
                }
            ))
        })
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("; "))
}

fn artifact_name(file_path: Option<&Value>) -> Option<String> {
    let path = file_path?.as_str()?;
    let mut parts = path
        .replace('\\', "/")
        .split('/')
        .map(String::from)
        .collect::<Vec<_>>();
    let start = parts.len().saturating_sub(2);
    Some(parts.drain(start..).collect::<Vec<_>>().join("/"))
}

fn summarize_lifecycle_command(manifest: &Value) -> Vec<String> {
    let mut out = Vec::new();

    if get(manifest, "preWrite")
        .and_then(|pre| get(pre, "requested"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        let pre = get(manifest, "preWrite").unwrap_or(&Value::Null);
        if get(pre, "ran").and_then(Value::as_bool) == Some(true) {
            let specific = artifact_name(get(pre, "advisoryPath"))
                .unwrap_or_else(|| "the invocation-specific advisory".to_string());
            let latest = artifact_name(get(pre, "latestAdvisoryPath"))
                .unwrap_or_else(|| "pre-write-advisory.latest.json".to_string());
            out.push(format!("- Pre-write ran and wrote an advisory. Use `{specific}` for the matching post-write check; `{latest}` is only the latest pointer."));
        } else {
            out.push(format!(
                "- Pre-write did not run: {}.",
                get(pre, "reason")
                    .and_then(Value::as_str)
                    .unwrap_or("reason unavailable")
            ));
        }
    }

    if get(manifest, "postWrite")
        .and_then(|post| get(post, "requested"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        let post = get(manifest, "postWrite").unwrap_or(&Value::Null);
        if get(post, "ran").and_then(Value::as_bool) == Some(true) {
            let baseline_status = get(post, "baselineStatus")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let scan_range_parity = get(post, "scanRangeParity")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let after_complete = get(post, "afterComplete").and_then(Value::as_bool) == Some(true);
            if baseline_status != "available" || scan_range_parity != "ok" || !after_complete {
                out.push(format!("- Post-write ran, but delta confidence is limited: baseline={baseline_status}, scanRange={scan_range_parity}, afterComplete={after_complete}. Read `post-write-delta.latest.json` before closing."));
            } else {
                let silent_new = n(get(post, "silentNew"));
                out.push(format!(
                    "- Post-write type-escape delta found {silent_new} {}. This is not a full behavior verdict.",
                    plural(silent_new, "new unplanned any-like escape", None)
                ));
            }
            let unexpected_new_files = n(get(post, "unexpectedNewFileCount"));
            let planned_missing_files = n(get(post, "plannedMissingFileCount"));
            if unexpected_new_files > 0 || planned_missing_files > 0 {
                out.push(format!("- Post-write file delta needs review: {unexpected_new_files} unexpected new {}, {planned_missing_files} planned missing {}. Read `post-write-delta.latest.json` before closing.",
                    plural(unexpected_new_files, "file", None),
                    plural(planned_missing_files, "file", None)
                ));
            }
        } else {
            out.push(format!(
                "- Post-write did not run: {}.",
                get(post, "reason")
                    .and_then(Value::as_str)
                    .unwrap_or("reason unavailable")
            ));
        }
    }

    if get(manifest, "canonDraft")
        .and_then(|draft| get(draft, "requested"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        let draft = get(manifest, "canonDraft").unwrap_or(&Value::Null);
        let draft_paths = arr(get(draft, "draftPaths"));
        let draft_count = draft_paths.len() as i64;
        if get(draft, "ran").and_then(Value::as_bool) == Some(true) && draft_count > 0 {
            let shown = draft_paths
                .iter()
                .take(3)
                .filter_map(|path| artifact_name(Some(path)))
                .collect::<Vec<_>>()
                .join(", ");
            let more = if draft_count > 3 {
                format!(", plus {} more", draft_count - 3)
            } else {
                String::new()
            };
            out.push(format!(
                "- Canon draft wrote {draft_count} proposal {} under canonical-draft/. Review manually before promotion.{}",
                plural(draft_count, "file", None),
                if shown.is_empty() {
                    String::new()
                } else {
                    format!(" Drafts: {shown}{more}.")
                }
            ));
        } else if get(draft, "ran").and_then(Value::as_bool) == Some(true) {
            out.push("- Canon draft ran, but no proposal path was recorded. Check per-source status before promotion.".to_string());
        } else {
            out.push(format!(
                "- Canon draft did not write proposals: {}.",
                get(draft, "reason")
                    .and_then(Value::as_str)
                    .unwrap_or("all requested sources failed")
            ));
        }
    }

    if get(manifest, "checkCanon")
        .and_then(|check| get(check, "requested"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        let check = get(manifest, "checkCanon").unwrap_or(&Value::Null);
        let summary = get(check, "summary").unwrap_or(&Value::Null);
        let drift_count = n(get(summary, "driftCount"));
        let checked = n(get(summary, "sourcesChecked"));
        let skipped = n(get(summary, "sourcesSkipped"));
        let failed = n(get(summary, "sourcesFailed"));
        if get(check, "ran").and_then(Value::as_bool) != Some(true) {
            out.push(format!(
                "- Check-canon did not run: {}.",
                get(check, "reason")
                    .and_then(Value::as_str)
                    .unwrap_or("reason unavailable")
            ));
        } else if checked == 0 {
            out.push(format!(
                "- Check-canon could not compare promoted canon yet: {skipped} {}, {failed} failed.",
                plural(skipped, "area", None)
            ));
        } else if drift_count > 0 {
            let drift_sources = object(get(check, "driftCounts"))
                .map(|object| object.values().filter(|count| n(Some(count)) > 0).count() as i64)
                .unwrap_or(0);
            out.push(format!(
                "- Check-canon found {drift_count} drift {} across {drift_sources}/{checked} checked {}.",
                plural(drift_count, "item", None),
                plural(checked, "area", None)
            ));
        } else {
            let caveat_count = skipped + failed;
            out.push(
                format!(
                    "- Check-canon is clean across {checked} checked {}.{}",
                    plural(checked, "area", None),
                    if caveat_count > 0 {
                        format!(
                            " {caveat_count} {} could not be checked.",
                            plural(caveat_count, "area", None)
                        )
                    } else {
                        String::new()
                    }
                )
                .trim()
                .to_string(),
            );
        }
    }

    out
}

fn summarize_scan_range(manifest: &Value) -> String {
    let empty = Value::Object(Default::default());
    let scan_range = get(manifest, "scanRange").unwrap_or(&empty);
    let langs = arr(get(scan_range, "languages"))
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let langs = if langs.is_empty() {
        "unknown".to_string()
    } else {
        langs.join(", ")
    };
    let tests = if get(scan_range, "includeTests").and_then(Value::as_bool) == Some(false) {
        "production files only"
    } else {
        "including tests"
    };
    let excludes = arr(get(scan_range, "excludes"));
    let exclude_text = if excludes.is_empty() {
        String::new()
    } else {
        format!(
            "; excludes: {}",
            excludes
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    format!(
        "{} files, {langs}, {tests}{exclude_text}",
        value_to_string(get(scan_range, "files"), "unknown")
    )
}

fn summarize_confidence(manifest: &Value) -> String {
    let confidence = get(manifest, "confidence").unwrap_or(&Value::Null);
    let blind_count = arr(get(manifest, "blindZones")).len();
    format!(
        "parse errors {}, unresolved internal {}, blind zones {blind_count}",
        value_to_string(get(confidence, "parseErrors"), "unknown"),
        pct(get(confidence, "unresolvedInternalRatio"))
    )
}

fn type_escape_total(discipline: &Value) -> i64 {
    let totals = get(discipline, "totals").unwrap_or(&Value::Null);
    n(get(totals, ":any"))
        + n(get(totals, "as any"))
        + n(get(totals, "as unknown as"))
        + n(get(totals, "@ts-ignore"))
        + n(get(totals, "@ts-expect-error"))
        + n(get(totals, "@ts-nocheck"))
        + n(get(totals, "jsdoc-any"))
}

fn measured_cue_lines(request: &AuditSummaryRenderRequest) -> Vec<String> {
    let mut lines = Vec::new();

    if get(&request.topology, "summary").is_some()
        || get(&request.checklist_facts, "A6_circular_deps").is_some()
    {
        let scc_count = n_or(
            request.topology.pointer("/summary/sccCount"),
            n(request
                .checklist_facts
                .pointer("/A6_circular_deps/sccCount")),
        );
        lines.push(format!("- Runtime cycles: {scc_count}. Read `topology.json.summary.sccCount` and `topology.json.sccs[]` before deciding whether a cycle matters."));
    }

    if let Some(a2) = get(&request.checklist_facts, "A2_function_size") {
        let oversized = if get(a2, "oversized").and_then(Value::as_array).is_some() {
            arr(get(a2, "oversized")).len() as i64
        } else {
            n(get(a2, "big"))
        };
        let watch = if get(a2, "watch").and_then(Value::as_array).is_some() {
            arr(get(a2, "watch")).len() as i64
        } else {
            n(get(a2, "medium"))
        };
        lines.push(format!(
            "- Function size: gate {}, oversized {oversized}, watch {watch}. Read `checklist-facts.json.A2_function_size` and screen test/script roles before proposing a split.",
            get(a2, "gate").and_then(Value::as_str).unwrap_or("unknown")
        ));
    }

    if let Some(e2) = get(&request.checklist_facts, "E2_silent_catch") {
        lines.push(format!(
            "- Catch handling: empty silent {}, non-empty anonymous {}, unused params {}. Read `checklist-facts.json.E2_silent_catch` before saying this lane is clean.",
            n(get(e2, "count")),
            n(get(e2, "nonEmptyAnonymousCount")),
            n(get(e2, "unusedParamCount"))
        ));
    }

    if get(&request.discipline, "totals").is_some() {
        lines.push(format!("- Type-check escapes: {} total any/ignore-style hits. Read `discipline.json.totals` and offender lists; do not rank this by count alone.", type_escape_total(&request.discipline)));
    }

    if let Some(cue) = format_any_contamination_cue(&request.symbols) {
        lines.push(cue);
    }

    if let Some(shape_drift) = get(&request.checklist_facts, "B1B2_shape_drift") {
        lines.push(format!(
            "- Shape drift: exact groups {}, near-shape cues {}. Read `checklist-facts.json.B1B2_shape_drift` and the declarations before merging concepts.",
            n(get(shape_drift, "exactDuplicateGroups")),
            n(get(shape_drift, "nearShapeCandidateCount"))
        ));
    }

    if get(&request.checklist_facts, "B1_duplicate_implementation").is_some()
        || get(&request.function_clones, "meta").is_some()
    {
        let b1 =
            get(&request.checklist_facts, "B1_duplicate_implementation").unwrap_or(&Value::Null);
        let meta = get(&request.function_clones, "meta").unwrap_or(&Value::Null);
        lines.push(format!("- JS/TS function clone cues: exact body groups {}, same-structure groups {}, same-signature groups {}, near-function cues {}. Read `function-clones.json` and source file:line evidence before calling JS/TS helpers duplicated; use Rust analyzer evidence for Rust files.",
            n_or(get(b1, "exactBodyGroups"), n(get(meta, "exactBodyGroupCount"))),
            n_or(get(b1, "structureGroupCandidates"), n(get(meta, "structureGroupCount"))),
            n_or(get(b1, "signatureGroupCandidates"), n(get(meta, "signatureGroupCount"))),
            n_or(get(b1, "nearFunctionCandidates"), n(get(meta, "nearFunctionCandidateCount"))),
        ));
    }

    if let Some(summary) = get(&request.fix_plan, "summary") {
        lines.push(format!(
            "- Dead-export tiers: SAFE_FIX {}, REVIEW_FIX {}, DEGRADED {}, MUTED {}. Read `fix-plan.json` plus FP context before recommending removal.",
            n(get(summary, "SAFE_FIX")),
            n(get(summary, "REVIEW_FIX")),
            n(get(summary, "DEGRADED")),
            n(get(summary, "MUTED"))
        ));
    }

    if let Some(cue) = format_unreachable_scc_cue(&request.module_reachability) {
        lines.push(format!("- {cue}. Read `module-reachability.json.unreachableStronglyConnectedComponents[]` before treating intra-cycle imports as liveness. This is dead-file-group review evidence, not export SAFE_FIX proof."));
    }

    let generated_consumer_zone_count = n(request
        .manifest
        .pointer("/generatedArtifacts/generatedConsumerBlindZoneCount"));
    if generated_consumer_zone_count > 0 {
        let top_scopes = format_generated_consumer_blind_zone_scopes(
            request
                .manifest
                .pointer("/generatedArtifacts/topGeneratedConsumerBlindZones"),
            3,
        );
        lines.push(format!(
            "- Generated consumer blind zones: {generated_consumer_zone_count}{}. Read `manifest.json.generatedArtifacts.topGeneratedConsumerBlindZones` and `symbols.json.generatedConsumerBlindZones` before treating generated code as absent.",
            top_scopes
                .map(|text| format!("; top scopes: {text}"))
                .unwrap_or_default()
        ));
    }

    if let Some(framework_resource_surfaces) = format_framework_resource_surface_counts(get(
        &request.manifest,
        "frameworkResourceSurfaces",
    )) {
        lines.push(format!("- Framework/resource surfaces: {framework_resource_surfaces}. Read `manifest.json.frameworkResourceSurfaces` and `framework-resource-surfaces.json` before treating import absence as deadness."));
    }

    if let Some(cue) = format_dependency_hygiene_cue(get(&request.manifest, "unusedDependencies")) {
        lines.push(format!("- {cue}"));
    }

    if let Some(cue) = format_sfc_evidence_cue(get(&request.manifest, "sfcEvidence")) {
        lines.push(format!("- {cue}"));
    }

    if let Some(cue) = format_rust_analysis_cue(get(&request.manifest, "rustAnalysis")) {
        lines.push(format!("- {cue}"));
    }

    if let Some(summary) = get(&request.call_graph, "summary") {
        let semi_dead = n_or(
            get(summary, "semiDead"),
            arr(get(&request.call_graph, "semiDeadList")).len() as i64,
        );
        lines.push(format!("- Call graph: semi-dead imports {semi_dead}. Read `call-graph.json.semiDeadList` and framework/test conventions before cleanup."));
    }

    let blind_zones = arr(get(&request.manifest, "blindZones"));
    if !blind_zones.is_empty() {
        lines.push(format!(
            "- Blind zones: {}. Read `manifest.json.blindZones` before any absence or removal claim.",
            blind_zones.len()
        ));
        let resolver_zone = blind_zones
            .iter()
            .find(|zone| get(zone, "area").and_then(Value::as_str) == Some("resolver"));
        if let Some(reasons) = format_unresolved_reason_counts(
            resolver_zone.and_then(|zone| zone.pointer("/details/topUnresolvedReasons")),
            3,
        ) {
            lines.push(format!("- Resolver blind-zone reasons: {reasons}. Read `symbols.json.unresolvedInternalSummaryByReason` and `manifest.json.blindZones[].details.topUnresolvedReasons` before treating unresolved imports as generic noise."));
        }
        if let Some(unresolved_roots) = format_top_unresolved_roots(
            request
                .manifest
                .pointer("/resolverDiagnostics/topSpecifierRoots"),
            3,
        ) {
            lines.push(format!("- Top unresolved roots: {unresolved_roots}. Read `manifest.json.resolverDiagnostics.topSpecifierRoots` to see which package or alias roots concentrate resolver blind zones."));
        }
        if let Some(affected_scopes) = format_top_affected_package_scopes(
            request
                .manifest
                .pointer("/resolverDiagnostics/topAffectedPackageScopes"),
            3,
        ) {
            lines.push(format!("- Resolver affected scopes: {affected_scopes}. Read `manifest.json.resolverDiagnostics.topAffectedPackageScopes` before treating resolver blind zones as repo-global blockers."));
        }
        let resolver = get(&request.manifest, "resolverDiagnostics").unwrap_or(&Value::Null);
        let blocked_count = n(get(resolver, "blockedCandidateHintCount"));
        let blocked_sample_limit = n(get(resolver, "blockedCandidateHintSampleLimit"));
        let blocked_hints =
            format_blocked_candidate_hints(get(resolver, "blockedCandidateHints"), 3);
        if blocked_count > 0 {
            let sample_limit = if blocked_sample_limit > 0 {
                format!("; manifest sample limit {blocked_sample_limit}")
            } else {
                String::new()
            };
            if let Some(distribution) = format_blocked_candidate_hint_distribution(resolver) {
                lines.push(format!("- Resolver blocked absence distribution: {distribution}. Read `manifest.json.resolverDiagnostics.blockedCandidateHintReasonCounts` and `manifest.json.resolverDiagnostics.blockedCandidateHintFamilyCounts` before opening the full hint list."));
            }
            lines.push(format!(
                "- Resolver blocked absence hints: {blocked_count}{sample_limit}{}. Read `manifest.json.resolverDiagnostics.blockedCandidateHints` and `resolver-diagnostics.json.blockedCandidateHints` before treating affected exports as absent.",
                blocked_hints
                    .map(|text| format!("; examples: {text}"))
                    .unwrap_or_default()
            ));
        }
    }

    if lines.is_empty() {
        vec!["- No measured cue lines were available from the provided artifacts. Read `manifest.json` and rerun the relevant profile before making structural claims.".to_string()]
    } else {
        lines
    }
}

fn artifact_map_lines(request: &AuditSummaryRenderRequest) -> Vec<String> {
    let produced = arr(get(&request.manifest, "artifactsProduced"))
        .iter()
        .filter_map(Value::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let mut lines = vec![
        "- `manifest.json`: scan range, confidence, blind zones, and command status.".to_string(),
    ];
    if request.symbols.is_object() || produced.contains("symbols.json") {
        lines.push("- `symbols.json`: export identities, total/type/value fan-in, dependency import consumers, public owner facts, unresolved internal reason summaries, generated consumer blind zones, and identity-level anyContamination owner maps.".to_string());
    }
    if request.checklist_facts.is_object() || produced.contains("checklist-facts.json") {
        lines.push("- `checklist-facts.json`: checklist gates and measured review cues; gates are triggers, not verdicts.".to_string());
    }
    if request.fix_plan.is_object() || produced.contains("fix-plan.json") {
        lines.push("- `fix-plan.json`: dead-export tiering; screen public surface and FP families before action.".to_string());
    }
    if request.topology.is_object() || produced.contains("topology.json") {
        lines.push(
            "- `topology.json`: cycles, cross-submodule edges, largest files, and topology details."
                .to_string(),
        );
    }
    if request.module_reachability.is_object() || produced.contains("module-reachability.json") {
        lines.push("- `module-reachability.json`: entry-rooted file reachability, unreachable files, and entry-unreachable SCC review cues.".to_string());
    }
    if produced.contains("topology.mermaid.md") {
        lines.push("- `topology.mermaid.md`: capped Mermaid diagrams plus hub-file notes for topology review; visual aid only, not citation authority.".to_string());
    }
    if request.discipline.is_object() || produced.contains("discipline.json") {
        lines.push(
            "- `discipline.json`: regex/AST-supported type-escape and suppression counts."
                .to_string(),
        );
    }
    if request.call_graph.is_object() || produced.contains("call-graph.json") {
        lines.push(
            "- `call-graph.json`: call graph and semi-dead import evidence from full profile."
                .to_string(),
        );
    }
    if produced.contains("shape-index.json") {
        lines.push("- `shape-index.json`: JS/TS exact shape-hash facts for full-profile B1/B2 review; use Rust analyzer shape/signature evidence for Rust files.".to_string());
    }
    if request.function_clones.is_object() || produced.contains("function-clones.json") {
        lines.push("- `function-clones.json`: JS/TS top-level exported and file-local function-body clone cues; candidates require source review before merge advice and are not Rust evidence.".to_string());
    }
    if request
        .manifest
        .pointer("/rustAnalysis/status")
        .and_then(Value::as_str)
        == Some("complete")
        && request
            .manifest
            .pointer("/rustAnalysis/available")
            .and_then(Value::as_bool)
            == Some(true)
    {
        lines.push("- `rust-analyzer-health.latest.json`: Rust-owned syntax, clone, unused-definition, and Cargo metadata evidence; use this for Rust files instead of JS/TS graph absence.".to_string());
    }
    if produced.contains("barrels.json") {
        lines.push(
            "- `barrels.json`: barrel discipline evidence for full-profile C7 review.".to_string(),
        );
    }
    if get(&request.manifest, "unusedDependencies").is_some()
        || produced.contains("unused-deps.json")
    {
        lines.push("- `unused-deps.json`: review-only dependency declaration evidence; inspect before changing package manifests.".to_string());
    }
    if get(&request.manifest, "sfcEvidence").is_some() {
        lines.push("- `symbols.json` SFC arrays: SFC import, reachability, asset, template, registration, generated-manifest, and framework-convention evidence; review-only SFC lanes do not prove fan-in or action readiness.".to_string());
    }
    lines
}

fn living_audit_lines(manifest: &Value) -> Vec<String> {
    let docs = arr(manifest.pointer("/livingAudit/existingDocs"));
    if docs.is_empty() {
        return Vec::new();
    }
    let shown = docs
        .iter()
        .filter_map(|doc| {
            let path = get(doc, "path")
                .and_then(Value::as_str)
                .or_else(|| doc.as_str())?;
            Some(format!("`{path}`"))
        })
        .collect::<Vec<_>>()
        .join(", ");
    vec![
        "## Living Audit Tracking".to_string(),
        String::new(),
        format!(
            "- Existing living audit document{} found: {shown}.",
            if docs.len() == 1 { "" } else { "s" }
        ),
        "- Read and update the document before the final answer. Mark items `RESOLVED` only with comparable scan range and produced evidence; otherwise use `NOT_RECHECKED`. Do not ask a subagent to own this document.".to_string(),
        String::new(),
    ]
}

fn expansion_hint_lines(manifest: &Value) -> Vec<String> {
    let profile = get(manifest, "profile").and_then(Value::as_str);
    if profile != Some("full") && profile != Some("ci") {
        return Vec::new();
    }
    vec![
        "## Expansion Hint".to_string(),
        String::new(),
        "Full-profile evidence is available. If the final chat answer stays short, add one low-pressure line saying the same evidence can be expanded into a full checklist walk, formal report, or due-diligence handoff.".to_string(),
        "Copyable phrases: `full checklist로 펼쳐줘`, `formal report로 써줘`, `due-diligence handoff로 정리해줘`.".to_string(),
        String::new(),
    ]
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
    let mut rows = object(map)
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

fn format_any_contamination_cue(symbols: &Value) -> Option<String> {
    let summary = summarize_any_contamination_owners(symbols);
    if !summary.present {
        return None;
    }
    if !summary.supported {
        return Some("- Exported any-contamination: not measured by this symbols.json. Treat semantic reuse/shape safety claims as not enough evidence yet.".to_string());
    }
    if summary.annotated == 0 {
        return Some("- Exported any-contamination: measured; no contaminated exported owner identities observed. Read `symbols.json.helperOwnersByIdentity` and `symbols.json.typeOwnersByIdentity` before semantic reuse or shape-merge claims.".to_string());
    }
    Some(format!(
        "- Exported any-contamination: {} severe type {}, {} severe helper {} ({} any-contaminated type {}, {} helper {}). Read `symbols.json.typeOwnersByIdentity` and `symbols.json.helperOwnersByIdentity` before semantic reuse or shape-merge claims.{}",
        summary.type_owner.severe,
        plural(summary.type_owner.severe, "owner", None),
        summary.helper.severe,
        plural(summary.helper.severe, "owner", None),
        summary.type_owner.any_contaminated,
        plural(summary.type_owner.any_contaminated, "owner", None),
        summary.helper.any_contaminated,
        plural(summary.helper.any_contaminated, "owner", None),
        example_text(&summary)
    ))
}

fn format_blocked_candidate_hints(hints: Option<&Value>, limit: usize) -> Option<String> {
    let parts = arr(hints)
        .iter()
        .take(limit)
        .filter_map(|hint| {
            let target = get(hint, "candidatePath")
                .or_else(|| get(hint, "affectedPackageScope"))
                .and_then(Value::as_str)?;
            let specifier = get(hint, "specifier").and_then(Value::as_str)?;
            let reason = get(hint, "reason").and_then(Value::as_str)?;
            Some(format!("{target} via {specifier} ({reason})"))
        })
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("; "))
}

fn format_distribution_list(
    items: Option<&Value>,
    label_key: &str,
    nested_key: &str,
    limit: usize,
) -> Option<String> {
    let parts = arr(items)
        .iter()
        .take(limit)
        .filter_map(|item| {
            let label = get(item, label_key).and_then(Value::as_str)?;
            let count = n_or(get(item, "count"), i64::MIN);
            if count == i64::MIN {
                return None;
            }
            let nested = format_counter_object(get(item, nested_key));
            Some(format!(
                "{label} {count}{}",
                nested.map(|text| format!(" ({text})")).unwrap_or_default()
            ))
        })
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join(", "))
}

fn format_blocked_candidate_hint_distribution(resolver_diagnostics: &Value) -> Option<String> {
    let reason_text = format_distribution_list(
        get(resolver_diagnostics, "blockedCandidateHintReasonCounts"),
        "reason",
        "families",
        3,
    );
    let family_text = format_distribution_list(
        get(resolver_diagnostics, "blockedCandidateHintFamilyCounts"),
        "family",
        "reasons",
        3,
    );
    let parts = [
        reason_text.map(|text| format!("reasons {text}")),
        family_text.map(|text| format!("families {text}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("; "))
}

fn shorten_console_line(line: &str, max: usize) -> String {
    let normalized = line.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized = normalized.trim();
    if normalized.chars().count() > max {
        format!(
            "{}…",
            normalized
                .chars()
                .take(max.saturating_sub(1))
                .collect::<String>()
        )
    } else {
        normalized.to_string()
    }
}

fn collect_summary_section_lines(markdown: &str, heading: &str, limit: usize) -> Vec<String> {
    let lines = markdown.lines().collect::<Vec<_>>();
    let Some(start) = lines.iter().position(|line| line.trim() == heading) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in lines.iter().skip(start + 1) {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }
        if is_list_item(trimmed) {
            out.push(shorten_console_line(trimmed, 150));
            if out.len() >= limit {
                break;
            }
        }
    }
    out
}

fn is_list_item(line: &str) -> bool {
    if line.starts_with("- ") {
        return true;
    }
    let mut chars = line.chars().peekable();
    let mut saw_digit = false;
    while matches!(chars.peek(), Some(ch) if ch.is_ascii_digit()) {
        saw_digit = true;
        chars.next();
    }
    saw_digit && chars.next() == Some('.') && chars.next() == Some(' ')
}

pub fn render_summary_console_preview(markdown: &str) -> Option<String> {
    let sections = [
        (
            "Command Result",
            collect_summary_section_lines(markdown, "## Command Result", 3),
        ),
        (
            "Read First",
            collect_summary_section_lines(markdown, "## Read First", 2),
        ),
        (
            "Measured Cues",
            collect_summary_section_lines(markdown, "## Measured Cues (Unranked)", 3),
        ),
        (
            "Living Audit Tracking",
            collect_summary_section_lines(markdown, "## Living Audit Tracking", 2),
        ),
        (
            "Guardrails",
            collect_summary_section_lines(markdown, "## Guardrails", 2),
        ),
    ];
    let sections = sections
        .into_iter()
        .filter(|(_, lines)| !lines.is_empty())
        .collect::<Vec<_>>();
    if sections.is_empty() {
        return None;
    }
    let mut out = vec!["[audit-repo] artifact brief preview:".to_string()];
    for (label, lines) in sections {
        out.push(format!("[audit-repo]   {label}:"));
        out.extend(
            lines
                .into_iter()
                .map(|line| format!("[audit-repo]     {line}")),
        );
    }
    Some(out.join("\n"))
}
