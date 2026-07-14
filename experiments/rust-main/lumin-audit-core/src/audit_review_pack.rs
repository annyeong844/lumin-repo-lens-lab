use anyhow::{bail, Result};

mod lanes;
mod protocol;
mod review_checks;
mod support;

use lanes::{dead_surface_lane, failure_lane, topology_lane, type_lane};
pub use protocol::{
    AuditReviewPackRenderRequest, AuditReviewPackRenderResult,
    AUDIT_REVIEW_PACK_RENDER_REQUEST_SCHEMA_VERSION,
    AUDIT_REVIEW_PACK_RENDER_RESULT_SCHEMA_VERSION,
};
use support::scan_range;

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
