use anyhow::Result;
use lumin_rust_common::{atomic_write_json_pretty, CliAction};

use super::request::{analyze_root, RustSourceHealthOptions};
use compact::CompactHealthResponse;
use options::{ArtifactProfile, WrapperOptions};

mod compact;
mod options;

pub fn run_cli(args: Vec<String>) -> Result<()> {
    let options = match WrapperOptions::parse(args)? {
        CliAction::Run(options) => options,
        CliAction::Help => return Ok(()),
    };
    let output = options.output.clone();
    let response = analyze_root(RustSourceHealthOptions {
        root: options.root,
        source_commit: options.source_commit,
        thread_count: options.thread_count,
        worker_stack_bytes: options.worker_stack_bytes,
    })?;
    match options.artifact_profile {
        ArtifactProfile::Compact => {
            let artifact = CompactHealthResponse::from_response(&response);
            atomic_write_json_pretty(&output, &artifact)?;
        }
        ArtifactProfile::Full => {
            atomic_write_json_pretty(&output, &response)?;
        }
    }
    println!("[rust-source-health] wrote {}", output.display());
    println!(
        "[rust-source-health] profile={} files={} skipped={} signals={}",
        options.artifact_profile.as_str(),
        response.summary.files,
        response.summary.skipped_files,
        response.summary.signals
    );
    Ok(())
}
