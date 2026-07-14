use serde_json::Value;

use super::super::support::{arr, format_counter_object, get, n, n_or, plural};

pub(super) fn format_framework_resource_surface_counts(summary: Option<&Value>) -> Option<String> {
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

pub(super) fn format_dependency_hygiene_cue(summary: Option<&Value>) -> Option<String> {
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

pub(super) fn format_rust_analysis_cue(summary: Option<&Value>) -> Option<String> {
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

pub(super) fn format_sfc_evidence_cue(summary: Option<&Value>) -> Option<String> {
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

pub(super) fn format_unreachable_scc_cue(module_reachability: &Value) -> Option<String> {
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

pub(super) fn format_generated_consumer_blind_zone_scopes(
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

pub(super) fn type_escape_total(discipline: &Value) -> i64 {
    let totals = get(discipline, "totals").unwrap_or(&Value::Null);
    n(get(totals, ":any"))
        + n(get(totals, "as any"))
        + n(get(totals, "as unknown as"))
        + n(get(totals, "@ts-ignore"))
        + n(get(totals, "@ts-expect-error"))
        + n(get(totals, "@ts-nocheck"))
        + n(get(totals, "jsdoc-any"))
}
