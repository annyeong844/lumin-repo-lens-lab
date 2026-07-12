use anyhow::Result;

use crate::analyzer::{analyze_files, analyze_source_file_entries, SourceFileEntry};
use crate::dead_exports::{
    classify_compact_unused_definitions_with_options, classify_unused_definitions_with_options,
    UnusedDefinitionAnalysisOptions,
};
use crate::function_clones::{
    function_clone_accumulator, group_function_body_fingerprints, group_function_clone_files,
};
use crate::parallel::{build_pool, RuntimeConfig};
use crate::protocol::{
    CompactFileHealth, HealthRequest, HealthResponse, IncrementalMeta, InputMeta, ParserMeta,
    PolicyMeta, ResponseMeta, RuntimeMeta, RustUnusedDefinitionAnalysis, SidecarMeta, SkippedFile,
    SourceHealthLimit, SourceHealthMode, SourceHealthProducer, Summary, PARSER_EDITION,
    PARSER_EDITION_POLICY, PARSER_EDITION_SOURCE, PARSER_KIND, PARSER_VERSION, POLICY_VERSION,
    SCHEMA_VERSION, SIGNAL_POLICY_ID, SIGNAL_POLICY_VERSION,
};
use crate::summary::{summarize, summarize_compact_summary_files};
use std::collections::BTreeMap;

use super::cache::{
    analyze_compact_with_cache, load_compact_dead_files_from_cache,
    load_compact_summary_files_from_cache, prepare_compact_cache,
    stream_compact_clone_files_from_cache, CompactCacheOptions,
};

pub struct FinalMeta {
    pub generated: String,
    pub sidecar: SidecarMeta,
    pub input: InputMeta,
}

pub struct CompactAnalysisResponse {
    pub schema_version: u32,
    pub meta: ResponseMeta,
    pub summary: Summary,
    pub function_clone_groups: crate::protocol::AstFunctionCloneGroups,
    pub unused_definition_analysis: RustUnusedDefinitionAnalysis,
    pub skipped_files: Vec<SkippedFile>,
    pub files: BTreeMap<String, CompactFileHealth>,
}

#[derive(Debug, Clone, Copy)]
pub struct AnalysisOptions {
    pub retain_raw_name_refs: bool,
    pub retain_raw_signals: bool,
    pub retain_raw_ast_lanes: bool,
    unused_definition_options: UnusedDefinitionAnalysisOptions,
}

impl AnalysisOptions {
    pub const fn full_artifact() -> Self {
        Self {
            retain_raw_name_refs: true,
            retain_raw_signals: true,
            retain_raw_ast_lanes: true,
            unused_definition_options: UnusedDefinitionAnalysisOptions::full_artifact(),
        }
    }

    pub const fn compact_artifact() -> Self {
        Self {
            retain_raw_name_refs: false,
            retain_raw_signals: false,
            retain_raw_ast_lanes: false,
            unused_definition_options: UnusedDefinitionAnalysisOptions::compact_artifact(),
        }
    }

    pub const fn with_raw_signals(mut self, retain_raw_signals: bool) -> Self {
        self.retain_raw_signals = retain_raw_signals;
        self
    }

    pub const fn with_raw_ast_lanes(mut self, retain_raw_ast_lanes: bool) -> Self {
        self.retain_raw_ast_lanes = retain_raw_ast_lanes;
        self
    }
}

pub fn analyze_request(
    request: HealthRequest,
    skipped_files: Vec<SkippedFile>,
    final_meta: Option<FinalMeta>,
) -> Result<HealthResponse> {
    analyze_request_with_options(
        request,
        skipped_files,
        final_meta,
        AnalysisOptions::full_artifact(),
    )
}

pub fn analyze_request_with_options(
    request: HealthRequest,
    skipped_files: Vec<SkippedFile>,
    final_meta: Option<FinalMeta>,
    options: AnalysisOptions,
) -> Result<HealthResponse> {
    let runtime_config = RuntimeConfig::try_from(request.runtime)?;
    let pool = build_pool(runtime_config)?;
    let files = pool.install(|| {
        analyze_files(
            &request.files,
            &request.parser,
            options.retain_raw_name_refs,
            options.retain_raw_signals,
            options.retain_raw_ast_lanes,
        )
    })?;
    response_from_files(
        files,
        skipped_files,
        final_meta,
        pool.current_num_threads(),
        runtime_config,
        options,
    )
}

pub(crate) fn analyze_source_entries_with_options(
    files: &[SourceFileEntry],
    parser: crate::protocol::ParserRequest,
    runtime: crate::protocol::RuntimeRequest,
    skipped_files: Vec<SkippedFile>,
    final_meta: Option<FinalMeta>,
    options: AnalysisOptions,
) -> Result<HealthResponse> {
    let runtime_config = RuntimeConfig::try_from(runtime)?;
    let pool = build_pool(runtime_config)?;
    let files = pool.install(|| {
        analyze_source_file_entries(
            files,
            &parser,
            options.retain_raw_name_refs,
            options.retain_raw_signals,
            options.retain_raw_ast_lanes,
        )
    })?;
    response_from_files(
        files,
        skipped_files,
        final_meta,
        pool.current_num_threads(),
        runtime_config,
        options,
    )
}

pub(crate) fn analyze_source_entries_compact_artifact(
    files: &[SourceFileEntry],
    parser: crate::protocol::ParserRequest,
    runtime: crate::protocol::RuntimeRequest,
    skipped_files: Vec<SkippedFile>,
    final_meta: Option<FinalMeta>,
    cache_options: CompactCacheOptions,
) -> Result<CompactAnalysisResponse> {
    let runtime_config = RuntimeConfig::try_from(runtime)?;
    let pool = build_pool(runtime_config)?;
    let thread_count = pool.current_num_threads();
    if cache_options.incremental_enabled {
        let prepared_cache =
            pool.install(|| prepare_compact_cache(files, &parser, cache_options.clone()))?;
        drop(pool);

        let mut clone_accumulator = function_clone_accumulator();
        stream_compact_clone_files_from_cache(files, &prepared_cache, |path, clone_file| {
            clone_accumulator.push_file(path, clone_file);
            Ok(())
        })?;
        let function_clone_groups = clone_accumulator.finish(&skipped_files);

        let compact_dead_files = load_compact_dead_files_from_cache(files, &prepared_cache)?;
        let unused_definition_analysis = classify_compact_unused_definitions_with_options(
            &compact_dead_files,
            UnusedDefinitionAnalysisOptions::compact_artifact(),
        );
        drop(compact_dead_files);

        let compact_summary_files = load_compact_summary_files_from_cache(files, &prepared_cache)?;
        let mut summary = summarize_compact_summary_files(&compact_summary_files);
        summary.skipped_files = skipped_files.len();
        summary.function_clone_exact_body_groups = function_clone_groups.exact_body_group_count;
        summary.function_clone_structure_groups = function_clone_groups.structure_group_count;
        summary.function_clone_signature_groups = function_clone_groups.signature_group_count;
        summary.function_clone_near_candidates =
            function_clone_groups.near_function_candidate_count;

        let (generated, sidecar, input) = final_meta
            .map(|meta| (Some(meta.generated), Some(meta.sidecar), Some(meta.input)))
            .unwrap_or((None, None, None));
        let compact_files = compact_summary_files
            .into_iter()
            .map(|(path, file)| (path, file.file))
            .collect();

        return Ok(CompactAnalysisResponse {
            schema_version: SCHEMA_VERSION,
            meta: response_meta(
                thread_count,
                runtime_config,
                generated,
                sidecar,
                input,
                Some(prepared_cache.incremental),
            ),
            summary,
            function_clone_groups,
            unused_definition_analysis,
            skipped_files,
            files: compact_files,
        });
    }

    let cache_run = pool.install(|| analyze_compact_with_cache(files, &parser, cache_options))?;
    drop(pool);

    let mut compact_dead_files = BTreeMap::new();
    let mut clone_files = BTreeMap::new();
    let mut compact_summary_files = BTreeMap::new();
    for (path, file) in cache_run.files {
        compact_summary_files.insert(path.clone(), file.summary_file);
        compact_dead_files.insert(path.clone(), file.dead_file);
        clone_files.insert(path, file.clone_file);
    }

    let unused_definition_analysis = classify_compact_unused_definitions_with_options(
        &compact_dead_files,
        UnusedDefinitionAnalysisOptions::compact_artifact(),
    );
    let mut summary = summarize_compact_summary_files(&compact_summary_files);
    let function_clone_groups = group_function_clone_files(&mut clone_files, &skipped_files);
    drop(clone_files);
    drop(compact_dead_files);

    summary.skipped_files = skipped_files.len();
    summary.function_clone_exact_body_groups = function_clone_groups.exact_body_group_count;
    summary.function_clone_structure_groups = function_clone_groups.structure_group_count;
    summary.function_clone_signature_groups = function_clone_groups.signature_group_count;
    summary.function_clone_near_candidates = function_clone_groups.near_function_candidate_count;

    let (generated, sidecar, input) = final_meta
        .map(|meta| (Some(meta.generated), Some(meta.sidecar), Some(meta.input)))
        .unwrap_or((None, None, None));
    let compact_files = compact_summary_files
        .into_iter()
        .map(|(path, file)| (path, file.file))
        .collect();

    Ok(CompactAnalysisResponse {
        schema_version: SCHEMA_VERSION,
        meta: response_meta(
            thread_count,
            runtime_config,
            generated,
            sidecar,
            input,
            Some(cache_run.incremental),
        ),
        summary,
        function_clone_groups,
        unused_definition_analysis,
        skipped_files,
        files: compact_files,
    })
}

fn response_from_files(
    mut files: std::collections::BTreeMap<String, crate::protocol::FileHealth>,
    skipped_files: Vec<SkippedFile>,
    final_meta: Option<FinalMeta>,
    thread_count: usize,
    runtime_config: RuntimeConfig,
    options: AnalysisOptions,
) -> Result<HealthResponse> {
    let unused_definition_analysis =
        classify_unused_definitions_with_options(&files, options.unused_definition_options);
    if !options.retain_raw_ast_lanes {
        for file in files.values_mut() {
            file.ast
                .prune_unused_definition_lanes_for_compact_source_health();
        }
    }
    let function_clone_groups =
        group_function_body_fingerprints(&mut files, &skipped_files, !options.retain_raw_ast_lanes);
    if !options.retain_raw_ast_lanes {
        for file in files.values_mut() {
            file.ast.prune_phase_lanes_for_compact_source_health();
        }
    }
    let mut summary = summarize(&files);
    summary.skipped_files = skipped_files.len();
    summary.function_clone_exact_body_groups = function_clone_groups.exact_body_group_count;
    summary.function_clone_structure_groups = function_clone_groups.structure_group_count;
    summary.function_clone_signature_groups = function_clone_groups.signature_group_count;
    summary.function_clone_near_candidates = function_clone_groups.near_function_candidate_count;
    let (generated, sidecar, input) = final_meta
        .map(|meta| (Some(meta.generated), Some(meta.sidecar), Some(meta.input)))
        .unwrap_or((None, None, None));
    Ok(HealthResponse {
        schema_version: SCHEMA_VERSION,
        meta: response_meta(
            thread_count,
            runtime_config,
            generated,
            sidecar,
            input,
            None,
        ),
        summary,
        function_clone_groups,
        unused_definition_analysis,
        skipped_files,
        files,
    })
}

fn response_meta(
    thread_count: usize,
    runtime_config: RuntimeConfig,
    generated: Option<String>,
    sidecar: Option<SidecarMeta>,
    input: Option<InputMeta>,
    incremental: Option<IncrementalMeta>,
) -> ResponseMeta {
    ResponseMeta {
        producer: SourceHealthProducer::RustSourceHealth,
        mode: SourceHealthMode::SyntaxOnly,
        parser: ParserMeta {
            kind: PARSER_KIND,
            version: PARSER_VERSION.to_string(),
            edition_policy: PARSER_EDITION_POLICY,
            edition: PARSER_EDITION,
            edition_source: PARSER_EDITION_SOURCE,
        },
        policy: PolicyMeta {
            version: POLICY_VERSION.to_string(),
            signal_policy: crate::protocol::SignalPolicyMeta {
                id: SIGNAL_POLICY_ID.to_string(),
                version: SIGNAL_POLICY_VERSION.to_string(),
            },
        },
        runtime: RuntimeMeta {
            thread_count,
            worker_stack_bytes: runtime_config.worker_stack_bytes,
        },
        limits: [
            SourceHealthLimit::SyntaxOnly,
            SourceHealthLimit::NoTypeInfo,
            SourceHealthLimit::NoTraitSolving,
            SourceHealthLimit::NoBorrowCheck,
        ],
        generated,
        sidecar,
        input,
        incremental,
    }
}
