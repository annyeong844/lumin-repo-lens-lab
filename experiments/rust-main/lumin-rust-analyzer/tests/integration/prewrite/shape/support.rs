use anyhow::{Context, Result};
use lumin_rust_source_health::{
    analyze_root,
    protocol::{HealthResponse, DEFAULT_WORKER_STACK_BYTES},
    RustSourceHealthOptions,
};

use crate::support::prewrite::PreWriteRepo;

pub(super) fn source_health(repo: &PreWriteRepo) -> Result<HealthResponse> {
    analyze_root(RustSourceHealthOptions {
        root: repo.root_path().to_path_buf(),
        source_commit: "test-source-commit".to_string(),
        include_tests: true,
        exclude: Vec::new(),
        thread_count: None,
        worker_stack_bytes: DEFAULT_WORKER_STACK_BYTES,
        retain_raw_name_refs: false,
        retain_raw_signals: true,
        retain_raw_ast_lanes: true,
        cache_root: None,
        incremental_enabled: false,
        clear_incremental_cache: false,
    })
}

pub(super) fn shape_hash(health: &HealthResponse, file: &str, name: &str) -> Result<String> {
    Ok(health
        .files
        .get(file)
        .with_context(|| format!("{file} health"))?
        .ast
        .shape_hashes
        .iter()
        .find(|fact| fact.name == name)
        .with_context(|| format!("{name} shape hash"))?
        .hash
        .clone())
}

pub(super) fn signature_hash(health: &HealthResponse, file: &str, name: &str) -> Result<String> {
    Ok(health
        .files
        .get(file)
        .with_context(|| format!("{file} health"))?
        .ast
        .function_signatures
        .iter()
        .find(|fact| fact.name == name)
        .with_context(|| format!("{name} function signature"))?
        .hash
        .clone())
}
