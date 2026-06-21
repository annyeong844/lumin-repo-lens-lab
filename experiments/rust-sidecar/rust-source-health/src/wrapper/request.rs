use std::path::PathBuf;

use anyhow::{Context, Result};
use lumin_rust_common::sha256_file;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::protocol::{
    HealthRequest, HealthResponse, InputMeta, ParserRequest, PathPolicy, RuntimeRequest,
    SidecarMeta, DEFAULT_EXCLUDE, DEFAULT_INCLUDE, PARSER_EDITION, PARSER_EDITION_POLICY,
    PARSER_EDITION_SOURCE, SCHEMA_VERSION,
};
use crate::{analyze_request, FinalMeta};

use super::files::{absolute_existing_dir, collect_rust_files};

#[derive(Debug)]
pub struct RustSourceHealthOptions {
    pub root: PathBuf,
    pub source_commit: String,
    pub thread_count: Option<usize>,
    pub worker_stack_bytes: usize,
}

pub fn analyze_root(options: RustSourceHealthOptions) -> Result<HealthResponse> {
    let root = absolute_existing_dir(&options.root)?;
    let (files, skipped_files) = collect_rust_files(&root)?;
    let path_policy = default_path_policy();
    let request = HealthRequest {
        schema_version: SCHEMA_VERSION,
        root: root.to_string_lossy().to_string(),
        files,
        path_policy: path_policy.clone(),
        parser: ParserRequest {
            edition_policy: PARSER_EDITION_POLICY,
            edition: PARSER_EDITION,
            edition_source: PARSER_EDITION_SOURCE,
        },
        runtime: RuntimeRequest {
            thread_count: options.thread_count,
            worker_stack_bytes: options.worker_stack_bytes,
        },
    };
    let binary_sha256 =
        sha256_file(&std::env::current_exe().context("failed to read current executable path")?)
            .context("failed to hash current executable")?;
    analyze_request(
        request,
        skipped_files,
        Some(FinalMeta {
            generated: OffsetDateTime::now_utc().format(&Rfc3339)?,
            sidecar: SidecarMeta {
                source_commit: options.source_commit,
                binary_sha256,
            },
            input: InputMeta { path_policy },
        }),
    )
}

fn default_path_policy() -> PathPolicy {
    PathPolicy {
        include: DEFAULT_INCLUDE
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
        exclude: DEFAULT_EXCLUDE
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
    }
}
