use std::path::PathBuf;

use anyhow::{Context, Result};
use lumin_rust_common::sha256_file;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::driver::{
    analyze_source_entries_compact_artifact, analyze_source_entries_with_options,
    CompactAnalysisResponse,
};
use crate::protocol::{
    HealthResponse, InputMeta, ParserRequest, PathPolicy, RuntimeRequest, SidecarMeta,
    DEFAULT_EXCLUDE, DEFAULT_INCLUDE, PARSER_EDITION, PARSER_EDITION_POLICY, PARSER_EDITION_SOURCE,
};
use crate::{AnalysisOptions, FinalMeta};

use super::files::{absolute_existing_dir, collect_rust_file_entries, RustFileScanScope};

#[derive(Debug)]
pub struct RustSourceHealthOptions {
    pub root: PathBuf,
    pub source_commit: String,
    pub include_tests: bool,
    pub exclude: Vec<String>,
    pub thread_count: Option<usize>,
    pub worker_stack_bytes: usize,
    pub retain_raw_name_refs: bool,
    pub retain_raw_signals: bool,
    pub retain_raw_ast_lanes: bool,
    pub cache_root: Option<PathBuf>,
    pub incremental_enabled: bool,
    pub clear_incremental_cache: bool,
}

pub fn analyze_root(options: RustSourceHealthOptions) -> Result<HealthResponse> {
    let root = absolute_existing_dir(&options.root)?;
    let scan_scope = RustFileScanScope::new(options.include_tests, &options.exclude);
    let (files, skipped_files) = collect_rust_file_entries(&root, &scan_scope)?;
    let input = input_meta(&scan_scope);
    let parser = ParserRequest {
        edition_policy: PARSER_EDITION_POLICY,
        edition: PARSER_EDITION,
        edition_source: PARSER_EDITION_SOURCE,
    };
    let runtime = RuntimeRequest {
        thread_count: options.thread_count,
        worker_stack_bytes: options.worker_stack_bytes,
    };
    let analysis_options = analysis_options(&options);
    let binary_sha256 =
        sha256_file(&std::env::current_exe().context("failed to read current executable path")?)
            .context("failed to hash current executable")?;
    analyze_source_entries_with_options(
        &files,
        parser,
        runtime,
        skipped_files,
        Some(FinalMeta {
            generated: OffsetDateTime::now_utc().format(&Rfc3339)?,
            sidecar: SidecarMeta {
                source_commit: options.source_commit,
                binary_sha256,
            },
            input,
        }),
        analysis_options,
    )
}

pub fn analyze_root_compact(options: RustSourceHealthOptions) -> Result<CompactAnalysisResponse> {
    let root = absolute_existing_dir(&options.root)?;
    let scan_scope = RustFileScanScope::new(options.include_tests, &options.exclude);
    let (files, skipped_files) = collect_rust_file_entries(&root, &scan_scope)?;
    let input = input_meta(&scan_scope);
    let parser = ParserRequest {
        edition_policy: PARSER_EDITION_POLICY,
        edition: PARSER_EDITION,
        edition_source: PARSER_EDITION_SOURCE,
    };
    let runtime = RuntimeRequest {
        thread_count: options.thread_count,
        worker_stack_bytes: options.worker_stack_bytes,
    };
    let binary_sha256 =
        sha256_file(&std::env::current_exe().context("failed to read current executable path")?)
            .context("failed to hash current executable")?;
    analyze_source_entries_compact_artifact(
        &files,
        parser,
        runtime,
        skipped_files,
        Some(FinalMeta {
            generated: OffsetDateTime::now_utc().format(&Rfc3339)?,
            sidecar: SidecarMeta {
                source_commit: options.source_commit,
                binary_sha256,
            },
            input,
        }),
        crate::driver::cache::CompactCacheOptions {
            root,
            cache_root: options.cache_root,
            incremental_enabled: options.incremental_enabled,
            clear_incremental_cache: options.clear_incremental_cache,
        },
    )
}

fn analysis_options(options: &RustSourceHealthOptions) -> AnalysisOptions {
    if options.retain_raw_name_refs {
        AnalysisOptions::full_artifact()
    } else {
        AnalysisOptions::compact_artifact()
    }
    .with_raw_signals(options.retain_raw_signals)
    .with_raw_ast_lanes(options.retain_raw_ast_lanes)
}

fn input_meta(scan_scope: &RustFileScanScope) -> InputMeta {
    let mut exclude = DEFAULT_EXCLUDE
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    exclude.extend(scan_scope.exclude_patterns().iter().cloned());
    InputMeta {
        path_policy: PathPolicy {
            include: DEFAULT_INCLUDE
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            exclude,
        },
        include_tests: scan_scope.include_tests(),
        exclude: scan_scope.exclude_patterns().to_vec(),
    }
}
