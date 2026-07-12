use anyhow::Result;
use lumin_rust_common::{atomic_write_json_pretty, CliAction};

use super::request::{analyze_root, analyze_root_compact, RustSourceHealthOptions};
use compact::CompactAnalysisHealthResponse;
use options::{ArtifactProfile, WrapperOptions};

mod compact;
mod options;

pub fn run_cli(args: Vec<String>) -> Result<()> {
    let options = match WrapperOptions::parse(args)? {
        CliAction::Run(options) => options,
        CliAction::Help => return Ok(()),
    };
    let output = options.output.clone();
    match options.artifact_profile {
        ArtifactProfile::Compact => {
            let response = analyze_root_compact(RustSourceHealthOptions {
                root: options.root,
                source_commit: options.source_commit,
                include_tests: true,
                exclude: Vec::new(),
                thread_count: options.thread_count,
                worker_stack_bytes: options.worker_stack_bytes,
                retain_raw_name_refs: false,
                retain_raw_signals: false,
                retain_raw_ast_lanes: false,
                cache_root: options.cache_root,
                incremental_enabled: options.incremental_enabled,
                clear_incremental_cache: options.clear_incremental_cache,
            })?;
            let artifact = CompactAnalysisHealthResponse::from_analysis(&response);
            atomic_write_json_pretty(&output, &artifact)?;
            println!("[rust-source-health] wrote {}", output.display());
            println!(
                "[rust-source-health] profile={} files={} skipped={} signals={}",
                options.artifact_profile.as_str(),
                response.summary.files,
                response.summary.skipped_files,
                response.summary.signals
            );
        }
        ArtifactProfile::Full => {
            let response = analyze_root(RustSourceHealthOptions {
                root: options.root,
                source_commit: options.source_commit,
                include_tests: true,
                exclude: Vec::new(),
                thread_count: options.thread_count,
                worker_stack_bytes: options.worker_stack_bytes,
                retain_raw_name_refs: true,
                retain_raw_signals: true,
                retain_raw_ast_lanes: true,
                cache_root: options.cache_root,
                incremental_enabled: false,
                clear_incremental_cache: false,
            })?;
            atomic_write_json_pretty(&output, &response)?;
            println!("[rust-source-health] wrote {}", output.display());
            println!(
                "[rust-source-health] profile={} files={} skipped={} signals={}",
                options.artifact_profile.as_str(),
                response.summary.files,
                response.summary.skipped_files,
                response.summary.signals
            );
        }
    }
    Ok(())
}
