use crate::policy::{
    evidence::{push_ast_file_support, SupportEvidence},
    FileParseStatus, ProductFileSemanticSummary, ProductSyntaxFileSummary,
};

pub(super) fn file_bridge_support_evidence(
    syntax_summary: ProductSyntaxFileSummary,
    semantic_summary: ProductFileSemanticSummary,
) -> Vec<SupportEvidence> {
    let mut supported_by = Vec::new();
    if syntax_summary.parse_status() != FileParseStatus::Ok {
        push_ast_file_support(
            &mut supported_by,
            syntax_summary.is_present(),
            syntax_summary.parse_status(),
            syntax_summary.parse_errors(),
        );
    }
    if semantic_summary.findings() > 0 || semantic_summary.diagnostics() > 0 {
        supported_by.push(SupportEvidence::cargo_rustc_diagnostics(
            semantic_summary.findings(),
            semantic_summary.diagnostics(),
        ));
    }
    supported_by
}
