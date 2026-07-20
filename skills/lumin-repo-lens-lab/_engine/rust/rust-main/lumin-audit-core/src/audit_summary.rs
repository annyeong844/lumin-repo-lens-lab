use anyhow::{bail, Result};
use serde_json::Value;

mod console;
mod lifecycle;
mod measured_cues;
mod protocol;
mod sections;
mod support;

pub use console::{format_blind_zones_console_summary, render_summary_console_preview};
use lifecycle::summarize_lifecycle_command;
use measured_cues::measured_cue_lines;
pub use protocol::{
    AuditSummaryRenderRequest, AuditSummaryRenderResult,
    AUDIT_SUMMARY_RENDER_REQUEST_SCHEMA_VERSION, AUDIT_SUMMARY_RENDER_RESULT_SCHEMA_VERSION,
};
use sections::{
    artifact_map_lines, expansion_hint_lines, living_audit_lines, required_analysis_failure_lines,
};
use support::{get, pointer_string, summarize_confidence, summarize_scan_range};

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

    lines.extend(required_analysis_failure_lines(&request.manifest));

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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    fn request(manifest: Value) -> AuditSummaryRenderRequest {
        AuditSummaryRenderRequest {
            schema_version: AUDIT_SUMMARY_RENDER_REQUEST_SCHEMA_VERSION.to_string(),
            manifest,
            checklist_facts: Value::Null,
            fix_plan: Value::Null,
            topology: Value::Null,
            discipline: Value::Null,
            call_graph: Value::Null,
            function_clones: Value::Null,
            symbols: Value::Null,
            module_reachability: Value::Null,
            output_path: "audit-summary.latest.md".to_string(),
        }
    }

    #[test]
    fn required_symbol_graph_failure_is_prominent_and_not_reported_as_clean() -> Result<()> {
        let markdown = render_audit_summary(&request(serde_json::json!({
            "commandsRun": [{
                "step": "build-symbol-graph.mjs",
                "status": "failed-required"
            }]
        })));

        let failure = markdown
            .find("## Required Analysis Failures")
            .context("required failure section")?;
        let read_first = markdown
            .find("## Read First")
            .context("read first section")?;
        assert!(failure < read_first);
        assert!(markdown.contains("Dead-export and reachability analysis is unavailable"));
        assert!(markdown.contains("do not read missing `symbols.json`"));
        let preview = render_summary_console_preview(&markdown).context("console preview")?;
        assert!(preview.contains("Required Analysis Failures"));
        assert!(preview.contains("Symbol graph failed"));
        Ok(())
    }

    #[test]
    fn successful_runs_do_not_render_required_failure_section() {
        let markdown = render_audit_summary(&request(serde_json::json!({
            "commandsRun": [{
                "step": "build-symbol-graph.mjs",
                "status": "ok"
            }]
        })));

        assert!(!markdown.contains("## Required Analysis Failures"));
    }
}
