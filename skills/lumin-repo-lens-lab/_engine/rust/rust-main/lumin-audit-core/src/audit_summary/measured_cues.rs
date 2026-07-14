use serde_json::Value;

mod any_contamination;
mod artifact_cues;
mod resolver_cues;

use super::protocol::AuditSummaryRenderRequest;
use super::support::{
    arr, base_evidence_not_refreshed, format_unresolved_reason_counts, get, n, n_or,
};
use any_contamination::format_any_contamination_cue;
use artifact_cues::{
    format_dependency_hygiene_cue, format_framework_resource_surface_counts,
    format_generated_consumer_blind_zone_scopes, format_rust_analysis_cue, format_sfc_evidence_cue,
    format_unreachable_scc_cue, type_escape_total,
};
use resolver_cues::{
    format_blocked_candidate_hint_distribution, format_blocked_candidate_hints,
    format_top_affected_package_scopes, format_top_unresolved_roots,
};

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
