use lumin_rust_cargo_oracle::protocol::Finding;

use crate::policy::{
    evidence::{push_ast_file_support, CoverageEvidence, SupportEvidence},
    FileParseStatus,
};

pub(super) fn finding_support_evidence(
    finding: &Finding,
    syntax_present: bool,
    parse_status: FileParseStatus,
    parse_errors: usize,
    coverage: &CoverageEvidence<'_>,
    has_safe_action: bool,
) -> Vec<SupportEvidence> {
    let mut supported_by = Vec::new();
    supported_by.push(SupportEvidence::cargo_rustc_diagnostic(
        finding.oracle_id,
        finding.claim_kind,
        finding.confidence_tier,
    ));
    push_ast_file_support(
        &mut supported_by,
        syntax_present,
        parse_status,
        parse_errors,
    );
    coverage.push_supported_by(&mut supported_by);
    if has_safe_action {
        supported_by.push(SupportEvidence::rustc_machine_applicable_safe_action());
    }
    supported_by
}
