use std::io::{self, Read};
use std::process;
use std::time::Instant;

use anyhow::{Context, Result};
use lumin_rust_common::{is_usage_error, usage_error};
use lumin_topology_scanner::protocol::{PolicyVersion, ScanRequest, ScanResponse, Timing};
use lumin_topology_scanner::scan_file_text;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        process::exit(if is_usage_error(&error) { 2 } else { 1 });
    }
}

fn run() -> Result<()> {
    let started = Instant::now();
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let request: ScanRequest = serde_json::from_str(&input)
        .map_err(|error| usage_error(format!("invalid request JSON: {error}")))?;
    if request.schema_version != 1 {
        return Err(usage_error(format!(
            "unsupported schemaVersion {}",
            request.schema_version
        )));
    }
    if !request.policy_version.is_supported() {
        return Err(usage_error(format!(
            "unsupported policyVersion {}",
            request.policy_version.as_str()
        )));
    }
    let _root = &request.root;

    let mut files = Vec::new();
    for file in &request.files {
        let source = std::fs::read_to_string(file).with_context(|| format!("read {file}"))?;
        files.push(scan_file_text(file, &source));
    }

    let response = ScanResponse {
        schema_version: 1,
        policy_version: PolicyVersion::current(),
        timing: Timing {
            files: files.len(),
            elapsed_ms: started.elapsed().as_millis(),
        },
        files,
    };
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}
