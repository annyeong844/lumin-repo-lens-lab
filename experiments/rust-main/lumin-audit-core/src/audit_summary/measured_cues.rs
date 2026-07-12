use super::{
    arr, base_evidence_not_refreshed, format_unresolved_reason_counts, get, n, n_or, object,
    plural, AuditSummaryRenderRequest,
};
use serde_json::Value;

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

pub(super) fn measured_cue_lines(request: &AuditSummaryRenderRequest) -> Vec<String> {
    if base_evidence_not_refreshed(&request.manifest) {
        return vec!["- Lifecycle scope: base audit not refreshed; this limits base-audit absence and freshness claims but does not degrade current lifecycle evidence.".to_string()];
    }

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
    let analysis_blind_zone_count = blind_zones
        .iter()
        .filter(|zone| get(zone, "area").and_then(Value::as_str) != Some("base-audit"))
        .count();
    if analysis_blind_zone_count > 0 {
        lines.push(format!(
            "- Blind zones: {analysis_blind_zone_count}. Read `manifest.json.blindZones` before any absence or removal claim."
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
