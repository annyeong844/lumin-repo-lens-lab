use serde_json::Value;

use super::support::{arr, format_counter_object, get, n, plural};

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

pub(super) fn format_framework_resource_surface_counts(summary: Option<&Value>) -> Option<String> {
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

pub(super) fn format_dependency_hygiene_review_check(summary: Option<&Value>) -> Option<String> {
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

pub(super) fn format_sfc_evidence_review_check(summary: Option<&Value>) -> Option<String> {
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

pub(super) fn format_unreachable_scc_review_check(module_reachability: &Value) -> Option<String> {
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

pub(super) fn format_any_contamination_review_check(symbols: &Value) -> String {
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

pub(super) fn format_blocked_candidate_hints(hints: Option<&Value>) -> Option<String> {
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

pub(super) fn format_blocked_candidate_hint_distribution(
    resolver_diagnostics: &Value,
) -> Option<String> {
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
