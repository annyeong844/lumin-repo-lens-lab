use serde_json::Value;
use std::path::Path;

use crate::classify::{diagnostic_ledger, Classification, Diagnostic};
use crate::command::CommandOutput;
use crate::metadata::{CargoMetadata, CargoPackage};
use crate::ownership::OwnershipResolver;
use crate::protocol::{
    ArtifactMeta, BlockingTarget, CacheReuse, ClaimKind, ClassificationEvidence, ConfidenceTier,
    CoverageEffect, CoverageEntry, CoverageKind, CoverageStatus, DiagnosticEvidence, Disposition,
    Finding, FindingConfidence, FindingSource, InputMeta, NormalizedDiagnostic,
    SemanticHealthArtifact, Summary,
};
use crate::scope::build_scope;
use crate::toolchain::{toolchain_meta, Toolchain};
use crate::util::unix_timestamp_string;
use crate::{
    ABSENCE_CLEAN_COVERAGE_ID, DIAGNOSTIC_POLICY_VERSION, EVENT_STREAM_COVERAGE_ID,
    EVIDENCE_POLICY_VERSION, ORACLE_REGISTRY_VERSION, SEMANTIC_HEALTH_SCHEMA_VERSION,
};

pub(crate) struct BuildArtifactInput<'a> {
    pub(crate) root: &'a Path,
    pub(crate) output: Option<&'a Path>,
    pub(crate) metadata: Option<&'a CargoMetadata>,
    pub(crate) messages: &'a [Value],
    pub(crate) stream_parse_status: &'a str,
    pub(crate) invalid_json_line_count: usize,
    pub(crate) check_output: &'a CommandOutput,
    pub(crate) cargo_bin: &'a str,
    pub(crate) cargo_args: &'a [String],
    pub(crate) selected_packages: &'a [CargoPackage],
    pub(crate) metadata_unavailable_reason: Option<String>,
    pub(crate) registry_hash: &'a str,
    pub(crate) input_hash: &'a str,
    pub(crate) features: Option<&'a str>,
    pub(crate) package_name: Option<&'a str>,
    pub(crate) toolchain: &'a Toolchain,
}

pub(crate) fn build_artifact(input: BuildArtifactInput<'_>) -> SemanticHealthArtifact {
    let ownership = OwnershipResolver::new(input.root, input.metadata, input.selected_packages);
    let diagnostics = diagnostic_ledger(input.messages, &ownership);
    let scope = build_scope(
        input.root,
        input.metadata,
        input.messages,
        input.selected_packages,
        input.features,
        input.toolchain,
    );

    let build_finished = input
        .messages
        .iter()
        .find(|message| message.get("reason").and_then(Value::as_str) == Some("build-finished"));

    let coverage = build_coverage(CoverageInput {
        build_finished,
        stream_parse_status: input.stream_parse_status,
        invalid_json_line_count: input.invalid_json_line_count,
        diagnostics: &diagnostics,
        scope: &scope,
        check_output: input.check_output,
        cargo_bin: input.cargo_bin,
        cargo_args: input.cargo_args,
        metadata_unavailable_reason: input.metadata_unavailable_reason.as_deref(),
        input_hash: input.input_hash,
        registry_hash: input.registry_hash,
    });

    let findings = findings_from_diagnostics(
        &diagnostics,
        input.input_hash,
        input.registry_hash,
        input.cargo_args,
        input.cargo_bin,
    );
    let summary = summary(&findings, &diagnostics, &coverage);

    SemanticHealthArtifact {
        schema_version: SEMANTIC_HEALTH_SCHEMA_VERSION,
        policy_version: EVIDENCE_POLICY_VERSION,
        oracle_registry_version: ORACLE_REGISTRY_VERSION,
        meta: ArtifactMeta {
            producer: "rust-cargo-oracle",
            mode: "semantic-oracle",
            generated: unix_timestamp_string(),
            oracle_registry_version: ORACLE_REGISTRY_VERSION,
            evidence_policy_version: EVIDENCE_POLICY_VERSION,
            diagnostic_policy_version: DIAGNOSTIC_POLICY_VERSION,
            registry_content_hash: input.registry_hash.to_string(),
            analysis_input_set_hash: input.input_hash.to_string(),
            analysis_input_set_complete: false,
            missing_influence_kinds: vec![
                "build-script-runtime-inputs",
                "proc-macro-runtime-inputs",
                "include-str-non-rust-files",
                "generated-files",
                "target-specific-cargo-config-expanded",
            ],
            toolchain: toolchain_meta(input.toolchain),
            cache_reuse_policy: "no-reuse-unless-complete-influence-set-is-captured",
            cache_reuse: cache_reuse_metadata(input.metadata),
            input: InputMeta {
                root: input.root.display().to_string(),
                package_name: input.package_name.map(str::to_string),
                features: input.features.map(str::to_string),
                cargo_bin: input.cargo_bin.to_string(),
                cargo_args: input.cargo_args.to_vec(),
            },
            output: input.output.map(|path| path.display().to_string()),
        },
        findings,
        diagnostics: diagnostics_to_json(&diagnostics),
        coverage,
        summary,
    }
}

struct CoverageInput<'a> {
    build_finished: Option<&'a Value>,
    stream_parse_status: &'a str,
    invalid_json_line_count: usize,
    diagnostics: &'a [Diagnostic],
    scope: &'a Value,
    check_output: &'a CommandOutput,
    cargo_bin: &'a str,
    cargo_args: &'a [String],
    metadata_unavailable_reason: Option<&'a str>,
    input_hash: &'a str,
    registry_hash: &'a str,
}

fn build_coverage(input: CoverageInput<'_>) -> Vec<CoverageEntry> {
    let stream_complete =
        input.stream_parse_status == "complete" && input.invalid_json_line_count == 0;
    let command_args_value = command_args(input.cargo_bin, input.cargo_args);
    let command = command_args_value.join(" ");
    let mut absence_unavailable = Vec::<String>::new();
    if input.stream_parse_status != "complete" {
        absence_unavailable.push(if input.stream_parse_status == "no-json-events" {
            "cargo JSON stream contained no events".to_string()
        } else {
            "cargo JSON stream did not parse completely".to_string()
        });
    }
    if input.invalid_json_line_count != 0 {
        absence_unavailable.push("cargo JSON stream contained invalid JSON lines".to_string());
    }
    if input.build_finished.is_none() {
        absence_unavailable.push("missing build-finished event".to_string());
    }
    if let Some(build_finished) = input.build_finished {
        match build_finished.get("success").and_then(Value::as_bool) {
            Some(true) => {}
            Some(false) => absence_unavailable.push("build-finished success was false".to_string()),
            None => absence_unavailable.push("build-finished success was not true".to_string()),
        }
    }
    if input.diagnostics.iter().any(|diagnostic| {
        diagnostic.classification.coverage_effect == Some(CoverageEffect::AbsenceCleanUnavailable)
    }) {
        absence_unavailable.push("non-user-code primary error diagnostic encountered".to_string());
    }
    if let Some(reason) = input.metadata_unavailable_reason {
        absence_unavailable.push(format!("cargo metadata unavailable: {reason}"));
    }

    let stream = CoverageEntry {
        id: EVENT_STREAM_COVERAGE_ID,
        oracle_id: "rust.cargo-check",
        coverage_kind: CoverageKind::CargoEventStream,
        status: if stream_complete {
            CoverageStatus::Ran
        } else {
            CoverageStatus::Unavailable
        },
        stream_parse_status: Some(input.stream_parse_status.to_string()),
        invalid_json_line_count: Some(input.invalid_json_line_count),
        scope: input.scope.clone(),
        command: command.clone(),
        command_args: command_args_value.clone(),
        exit_code: input.check_output.status,
        elapsed_ms: input.check_output.elapsed_ms,
        analysis_input_set_hash: input.input_hash.to_string(),
        registry_content_hash: input.registry_hash.to_string(),
        diagnostic_policy_version: DIAGNOSTIC_POLICY_VERSION,
        reason: (!stream_complete)
            .then(|| "cargo JSON stream unavailable or incomplete".to_string()),
        clean_kind: None,
        clean_scope: None,
        absence_of_claim_kinds: Vec::new(),
        allows_concurrent_claim_kinds: Vec::new(),
        clean: None,
    };

    let verified_errors = input
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(diagnostic.classification.disposition, Disposition::Finding)
                && matches!(
                    diagnostic.classification.confidence,
                    Some(ConfidenceTier::Verified)
                )
                && diagnostic
                    .classification
                    .claim_kind
                    .is_some_and(ClaimKind::is_verified_rustc_error)
        })
        .count();

    let absence_ran = absence_unavailable.is_empty();
    let absence = CoverageEntry {
        id: ABSENCE_CLEAN_COVERAGE_ID,
        oracle_id: "rust.cargo-check",
        coverage_kind: CoverageKind::AbsenceClean,
        status: if absence_ran {
            CoverageStatus::Ran
        } else {
            CoverageStatus::Unavailable
        },
        stream_parse_status: None,
        invalid_json_line_count: None,
        scope: input.scope.clone(),
        command,
        command_args: command_args(input.cargo_bin, input.cargo_args),
        exit_code: input.check_output.status,
        elapsed_ms: input.check_output.elapsed_ms,
        analysis_input_set_hash: input.input_hash.to_string(),
        registry_content_hash: input.registry_hash.to_string(),
        diagnostic_policy_version: DIAGNOSTIC_POLICY_VERSION,
        reason: (!absence_ran).then(|| absence_unavailable.join("; ")),
        clean_kind: absence_ran.then_some("verified-rustc-error-absence"),
        clean_scope: absence_ran
            .then_some("verified rustc error diagnostics for the declared cargo-check scope"),
        absence_of_claim_kinds: absence_ran
            .then(|| ClaimKind::ABSENCE_CLEAN_CLAIM_KINDS.to_vec())
            .unwrap_or_default(),
        allows_concurrent_claim_kinds: absence_ran
            .then(|| ClaimKind::ABSENCE_CLEAN_CONCURRENT_CLAIM_KINDS.to_vec())
            .unwrap_or_default(),
        clean: absence_ran.then_some(verified_errors == 0),
    };

    vec![stream, absence]
}

fn command_args(cargo_bin: &str, cargo_args: &[String]) -> Vec<String> {
    let mut out = vec![cargo_bin.to_string()];
    out.extend(cargo_args.iter().cloned());
    out
}

fn findings_from_diagnostics(
    diagnostics: &[Diagnostic],
    input_hash: &str,
    registry_hash: &str,
    cargo_args: &[String],
    cargo_bin: &str,
) -> Vec<Finding> {
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
            let command_args = command_args(cargo_bin, cargo_args);
            Finding {
                oracle_id: "rust.cargo-check",
                source: FindingSource {
                    oracle_id: "rust.cargo-check",
                    source_kind: "semantic-oracle",
                    version: "cargo-check-json.v1",
                    command: command_args.join(" "),
                    command_args,
                    registry_content_hash: registry_hash.to_string(),
                },
                confidence: FindingConfidence {
                    tier,
                    authority_ids: claim_kind.authority_ids(),
                    rule_ids: claim_kind.rule_ids(),
                    claim_kind,
                },
                confidence_tier: tier,
                claim_kind,
                message: diagnostic.message.clone(),
                span: diagnostic
                    .primary_spans
                    .first()
                    .cloned()
                    .unwrap_or(Value::Null),
                primary_spans: diagnostic.primary_spans.clone(),
                coverage_ref: EVENT_STREAM_COVERAGE_ID,
                analysis_input_set_hash: input_hash.to_string(),
                rule: diagnostic.classification.rule,
            }
        })
        .collect()
}

fn diagnostics_to_json(diagnostics: &[Diagnostic]) -> Vec<DiagnosticEvidence> {
    diagnostics
        .iter()
        .map(|diagnostic| DiagnosticEvidence {
            level: diagnostic.level.clone(),
            raw_code: diagnostic.raw.get("code").cloned().unwrap_or(Value::Null),
            normalized: NormalizedDiagnostic {
                code_presence: diagnostic.code_presence,
                code_value: diagnostic.code_value.clone(),
                code_namespace: diagnostic.code_namespace,
                code_kind: diagnostic.code_kind,
                primary_span: primary_span_class(&diagnostic.primary_spans),
            },
            classification: classification_json(&diagnostic.classification),
            message: diagnostic.message.clone(),
            primary_spans: diagnostic.primary_spans.clone(),
            rendered_first_line: diagnostic.rendered_first_line.clone(),
        })
        .collect()
}

fn primary_span_class(primary_spans: &[Value]) -> String {
    primary_spans
        .iter()
        .find_map(|span| {
            let class = span.get("primarySpanClass").and_then(Value::as_str)?;
            (class == "user-code").then_some(class)
        })
        .or_else(|| {
            primary_spans
                .first()
                .and_then(|span| span.get("primarySpanClass"))
                .and_then(Value::as_str)
        })
        .unwrap_or("unknown")
        .to_string()
}

fn classification_json(classification: &Classification) -> ClassificationEvidence {
    ClassificationEvidence {
        disposition: classification.disposition,
        confidence: classification.confidence,
        claim_kind: classification.claim_kind,
        coverage_effect: classification.coverage_effect,
        rule: classification.rule,
    }
}

fn summary(
    findings: &[Finding],
    diagnostics: &[Diagnostic],
    coverage: &[CoverageEntry],
) -> Summary {
    Summary {
        findings: findings.len(),
        diagnostics: diagnostics.len(),
        coverage: coverage.len(),
        verified_findings: findings
            .iter()
            .filter(|finding| finding.confidence.tier == ConfidenceTier::Verified)
            .count(),
        rule_backed_findings: findings
            .iter()
            .filter(|finding| finding.confidence.tier == ConfidenceTier::RuleBacked)
            .count(),
        candidate_findings: findings
            .iter()
            .filter(|finding| finding.confidence.tier == ConfidenceTier::Candidate)
            .count(),
        coverage_unavailable_diagnostics: diagnostics
            .iter()
            .filter(|diagnostic| {
                matches!(
                    diagnostic.classification.disposition,
                    Disposition::CoverageUnavailable
                )
            })
            .count(),
    }
}

fn cache_reuse_metadata(metadata: Option<&CargoMetadata>) -> CacheReuse {
    let blocking_targets: Vec<BlockingTarget> = metadata
        .map(|metadata| {
            metadata
                .packages
                .iter()
                .flat_map(|pkg| {
                    pkg.targets
                        .iter()
                        .filter_map(|target| {
                            if target
                                .kind
                                .iter()
                                .any(|kind| kind == "custom-build" || kind == "proc-macro")
                            {
                                Some(BlockingTarget {
                                    package_id: pkg.id.clone(),
                                    package_name: pkg.name.clone(),
                                    target_name: target.name.clone(),
                                    target_kinds: target.kind.clone(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        })
        .unwrap_or_default();
    CacheReuse {
        policy: "no-reuse-unless-complete-influence-set-is-captured",
        reason: "analysis-input-set-incomplete-for-cache-reuse",
        blocking_targets,
    }
}
