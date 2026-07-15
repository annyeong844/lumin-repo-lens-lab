use std::collections::BTreeSet;

use serde_json::Value;

use super::protocol::AuditSummaryRenderRequest;
use super::support::{arr, get};

pub(super) fn required_analysis_failure_lines(manifest: &Value) -> Vec<String> {
    let failures = arr(get(manifest, "commandsRun"))
        .iter()
        .filter(|command| get(command, "status").and_then(Value::as_str) == Some("failed-required"))
        .filter_map(|command| get(command, "step").and_then(Value::as_str))
        .collect::<Vec<_>>();
    if failures.is_empty() {
        return Vec::new();
    }

    let mut lines = vec!["## Required Analysis Failures".to_string(), String::new()];
    for step in failures.into_iter().take(5) {
        if step == "build-symbol-graph.mjs" {
            lines.push("- **Symbol graph failed.** Dead-export and reachability analysis is unavailable: do not read missing `symbols.json`, `fix-plan.json`, `dead-classify.json`, `module-reachability.json`, or `entry-surface.json` as zero findings.".to_string());
        } else {
            lines.push(format!(
                "- **Required producer `{step}` failed.** Its missing downstream evidence is unavailable, not clean."
            ));
        }
    }
    lines.push(String::new());
    lines
}

pub(super) fn artifact_map_lines(request: &AuditSummaryRenderRequest) -> Vec<String> {
    let produced = arr(get(&request.manifest, "artifactsProduced"))
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
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

pub(super) fn living_audit_lines(manifest: &Value) -> Vec<String> {
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

pub(super) fn expansion_hint_lines(manifest: &Value) -> Vec<String> {
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
