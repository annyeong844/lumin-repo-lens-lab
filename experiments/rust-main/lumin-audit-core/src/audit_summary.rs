use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod measured_cues;

use measured_cues::measured_cue_lines;

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
    if base_evidence_not_refreshed(manifest) {
        return "base audit not refreshed (lifecycle-only); use lifecycle evidence for this command"
            .to_string();
    }
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
    if base_evidence_not_refreshed(manifest) {
        return "base audit not evaluated; lifecycle evidence status is independent".to_string();
    }
    let confidence = get(manifest, "confidence").unwrap_or(&Value::Null);
    let blind_count = arr(get(manifest, "blindZones"))
        .iter()
        .filter(|zone| get(zone, "area").and_then(Value::as_str) != Some("base-audit"))
        .count();
    format!(
        "parse errors {}, unresolved internal {}, blind zones {blind_count}",
        value_to_string(get(confidence, "parseErrors"), "unknown"),
        pct(get(confidence, "unresolvedInternalRatio"))
    )
}

fn base_evidence_not_refreshed(manifest: &Value) -> bool {
    manifest
        .pointer("/baseEvidence/status")
        .and_then(Value::as_str)
        == Some("not-refreshed")
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

pub fn format_blind_zones_console_summary(zones: &[Value]) -> Option<String> {
    if zones.is_empty() {
        return None;
    }
    let base_scope_count = zones
        .iter()
        .filter(|zone| get(zone, "area").and_then(Value::as_str) == Some("base-audit"))
        .count();
    let analysis_zone_count = zones.len().saturating_sub(base_scope_count);
    let mut parts = Vec::new();
    for (severity, label) in [
        ("scan-gap", "scan-gap"),
        ("precision-gap", "precision-gap"),
        ("confidence-gap", "confidence-gap"),
    ] {
        let count = zones
            .iter()
            .filter(|zone| {
                get(zone, "area").and_then(Value::as_str) != Some("base-audit")
                    && get(zone, "severity").and_then(Value::as_str) == Some(severity)
            })
            .count();
        if count > 0 {
            parts.push(format!("{count} {label}"));
        }
    }
    let analysis_summary = if analysis_zone_count == 0 {
        "blindZones: none in current lifecycle evidence".to_string()
    } else if parts.is_empty() {
        format!("blindZones: {analysis_zone_count} unclassified")
    } else {
        format!("blindZones: {}", parts.join(", "))
    };
    let resolver_summary = zones
        .iter()
        .find(|zone| get(zone, "area").and_then(Value::as_str) == Some("resolver"))
        .and_then(|zone| {
            format_unresolved_reason_counts(zone.pointer("/details/topUnresolvedReasons"), 3)
        })
        .map(|reasons| format!("; resolver reasons: {reasons}"))
        .unwrap_or_default();
    let base_summary = if base_scope_count > 0 {
        "; baseEvidence: not refreshed (lifecycle-only)"
    } else {
        ""
    };
    Some(format!(
        "{analysis_summary}{resolver_summary}{base_summary}"
    ))
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
