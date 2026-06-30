mod artifact;
mod cues;
mod index;
mod intent;
mod lookup;
mod operation;
mod tokens;

use anyhow::Result;
use lumin_rust_common::canonical_existing_dir_usage;
use lumin_rust_source_health::{analyze_root, RustSourceHealthOptions};

pub(crate) use artifact::PreWriteArtifact;

use crate::cli::PreWriteOptions;

pub(crate) fn run(options: &PreWriteOptions) -> Result<PreWriteArtifact> {
    let root = canonical_existing_dir_usage(&options.root, "--root")?;
    let loaded_intent = intent::load(&options.intent)?;
    let syntax = analyze_root(RustSourceHealthOptions {
        root: root.clone(),
        source_commit: options.source_commit.clone(),
        thread_count: options.thread_count,
        worker_stack_bytes: options.worker_stack_bytes,
        retain_raw_name_refs: false,
        retain_raw_signals: true,
        retain_raw_ast_lanes: true,
        cache_root: None,
        incremental_enabled: false,
        clear_incremental_cache: false,
    })?;
    artifact::build(loaded_intent, &syntax, &root)
}
