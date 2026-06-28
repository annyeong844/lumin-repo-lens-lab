use std::path::Path;

use anyhow::Result;

use crate::cargo_json::CargoJsonMessages;
use crate::classify::{diagnostic_ledger, StreamParseStatus};
use crate::command::CommandOutput;
use crate::environment::CompilationEnvironment;
use crate::metadata::{CargoMetadata, CargoPackage};
use crate::ownership::OwnershipResolver;
use crate::protocol::{
    ArtifactMeta, CacheReusePolicy, CargoCheckMode, CargoTargetDirMode, CargoTargetDirPolicy,
    InputMeta, MissingInfluenceKind, OraclePlan, SemanticArtifactMode, SemanticArtifactProducer,
    SemanticHealthArtifact,
};
use crate::scope::build_scope;
use crate::toolchain::{toolchain_meta, Toolchain};
use crate::util::generated_timestamp_string;
use crate::{
    DIAGNOSTIC_POLICY_VERSION, EVIDENCE_POLICY_VERSION, ORACLE_REGISTRY_VERSION,
    SEMANTIC_HEALTH_SCHEMA_VERSION,
};

mod command_line;
mod coverage;
mod diagnostics;
mod findings;
mod safe_action;
mod summaries;

use coverage::{build_coverage, CoverageInput};
use diagnostics::diagnostics_to_json;
use findings::findings_from_diagnostics;
use summaries::{cache_reuse_metadata, summary};

pub(crate) struct BuildArtifactInput<'a> {
    pub(crate) root: &'a Path,
    pub(crate) output: Option<&'a Path>,
    pub(crate) metadata: Option<&'a CargoMetadata>,
    pub(crate) messages: CargoJsonMessages<'a>,
    pub(crate) stream_parse_status: StreamParseStatus,
    pub(crate) invalid_json_line_count: usize,
    pub(crate) check_output: &'a CommandOutput,
    pub(crate) cargo_bin: &'a str,
    pub(crate) cargo_args: &'a [String],
    pub(crate) selected_packages: &'a [CargoPackage],
    pub(crate) metadata_unavailable_reason: Option<String>,
    pub(crate) registry_hash: &'a str,
    pub(crate) input_hash: &'a str,
    pub(crate) compilation_environment: &'a CompilationEnvironment,
    pub(crate) features: Option<&'a str>,
    pub(crate) package_name: Option<&'a str>,
    pub(crate) toolchain: &'a Toolchain,
    pub(crate) cargo_check_mode: CargoCheckMode,
    pub(crate) cargo_target_dir_mode: CargoTargetDirMode,
    pub(crate) cargo_target_dir: &'a Path,
    pub(crate) oracle_plan: OraclePlan,
}

pub(crate) fn build_artifact(input: BuildArtifactInput<'_>) -> Result<SemanticHealthArtifact> {
    let ownership = OwnershipResolver::new(input.root, input.metadata, input.selected_packages);
    let diagnostics = diagnostic_ledger(input.messages, &ownership);
    let scope = build_scope(
        input.root,
        input.metadata,
        input.messages,
        input.selected_packages,
        input.features,
        input.toolchain,
        input.compilation_environment,
    );

    let build_finished = input.messages.build_finished();

    let coverage = build_coverage(CoverageInput {
        build_finished,
        stream_parse_status: input.stream_parse_status,
        invalid_json_line_count: input.invalid_json_line_count,
        diagnostics: &diagnostics,
        scope: &scope,
        check_output: input.check_output,
        cargo_bin: input.cargo_bin,
        cargo_args: input.cargo_args,
        cargo_check_mode: input.cargo_check_mode,
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
    let cache_reuse = cache_reuse_metadata(input.metadata);
    let summary = summary(&findings, &diagnostics, &coverage, &cache_reuse);

    Ok(SemanticHealthArtifact {
        schema_version: SEMANTIC_HEALTH_SCHEMA_VERSION,
        policy_version: EVIDENCE_POLICY_VERSION,
        oracle_registry_version: ORACLE_REGISTRY_VERSION,
        meta: ArtifactMeta {
            producer: SemanticArtifactProducer::RustCargoOracle,
            mode: SemanticArtifactMode::SemanticOracle,
            generated: generated_timestamp_string()?,
            oracle_registry_version: ORACLE_REGISTRY_VERSION,
            evidence_policy_version: EVIDENCE_POLICY_VERSION,
            diagnostic_policy_version: DIAGNOSTIC_POLICY_VERSION,
            registry_content_hash: input.registry_hash.to_string(),
            analysis_input_set_hash: input.input_hash.to_string(),
            analysis_input_set_complete: false,
            missing_influence_kinds: vec![
                MissingInfluenceKind::BuildScriptRuntimeInputs,
                MissingInfluenceKind::ProcMacroRuntimeInputs,
                MissingInfluenceKind::IncludeStrNonRustFiles,
                MissingInfluenceKind::GeneratedFiles,
                MissingInfluenceKind::TargetSpecificCargoConfigExpanded,
            ],
            toolchain: toolchain_meta(input.toolchain),
            cache_reuse_policy: CacheReusePolicy::NoReuseUnlessCompleteInfluenceSetIsCaptured,
            cache_reuse,
            input: InputMeta {
                root: input.root.display().to_string(),
                package_name: input.package_name.map(str::to_string),
                features: input.features.map(str::to_string),
                cargo_check_mode: input.cargo_check_mode,
                cargo_target_dir_mode: input.cargo_target_dir_mode,
                cargo_target_dir_policy: CargoTargetDirPolicy::from_mode(
                    input.cargo_target_dir_mode,
                ),
                cargo_target_dir: input.cargo_target_dir.display().to_string(),
                cargo_bin: input.cargo_bin.to_string(),
                cargo_args: input.cargo_args.to_vec(),
            },
            output: input.output.map(|path| path.display().to_string()),
        },
        findings,
        diagnostics: diagnostics_to_json(diagnostics),
        coverage,
        oracle_plan: input.oracle_plan,
        summary,
    })
}
