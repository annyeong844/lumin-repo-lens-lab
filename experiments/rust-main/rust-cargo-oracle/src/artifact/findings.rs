use crate::classify::Diagnostic;
use crate::protocol::{
    ClaimKind, Disposition, Finding, FindingConfidence, FindingSource, FindingSourceKind,
    FindingSourceVersion, OracleId, PrimarySpan, EVENT_STREAM_COVERAGE_ID,
};

use super::command_line::command_args;
use super::safe_action::safe_action_decision;

pub(super) fn findings_from_diagnostics(
    diagnostics: &[Diagnostic],
    input_hash: &str,
    registry_hash: &str,
    cargo_args: &[String],
    cargo_bin: &str,
) -> Vec<Finding> {
    let command_args = command_args(cargo_bin, cargo_args);
    let command = command_args.join(" ");
    let source = FindingSource {
        oracle_id: OracleId::RustCargoCheck,
        source_kind: FindingSourceKind::SemanticOracle,
        version: FindingSourceVersion::CargoCheckJsonV1,
        command,
        command_args,
        registry_content_hash: registry_hash.to_string(),
    };
    diagnostics
        .iter()
        .filter(|diagnostic| matches!(diagnostic.classification.disposition, Disposition::Finding))
        .map(|diagnostic| {
            let claim_kind = diagnostic
                .classification
                .claim_kind
                .unwrap_or(ClaimKind::UnclassifiedCargoDiagnostic);
            let tier = diagnostic
                .classification
                .confidence
                .unwrap_or_else(|| claim_kind.tier());
            let action_decision = safe_action_decision(diagnostic);
            Finding {
                oracle_id: OracleId::RustCargoCheck,
                source: source.clone(),
                confidence: FindingConfidence {
                    tier,
                    authority_ids: claim_kind.authority_ids(),
                    rule_ids: claim_kind.rule_ids(),
                    claim_kind,
                },
                confidence_tier: tier,
                claim_kind,
                message: diagnostic.message.clone(),
                span: PrimarySpan::representative(&diagnostic.primary_spans).cloned(),
                primary_spans: diagnostic.primary_spans.clone(),
                coverage_ref: EVENT_STREAM_COVERAGE_ID,
                analysis_input_set_hash: input_hash.to_string(),
                rule: diagnostic.classification.rule,
                action_blockers: action_decision.action_blockers,
                safe_action: action_decision.safe_action,
            }
        })
        .collect()
}
