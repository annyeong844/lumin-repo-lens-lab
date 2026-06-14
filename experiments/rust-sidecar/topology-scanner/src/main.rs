mod protocol;
mod scanner;

use std::io::{self, Read};
use std::time::Instant;

use anyhow::{bail, Context, Result};
use protocol::{ScanRequest, ScanResponse, Timing};
use scanner::{scan_file_text, POLICY_VERSION};

fn main() -> Result<()> {
    let started = Instant::now();
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let request: ScanRequest = serde_json::from_str(&input).context("invalid request JSON")?;
    if request.schema_version != 1 {
        bail!("unsupported schemaVersion {}", request.schema_version);
    }
    if request.policy_version != POLICY_VERSION {
        bail!("unsupported policyVersion {}", request.policy_version);
    }
    let _root = &request.root;

    let mut files = Vec::new();
    for file in &request.files {
        let source = std::fs::read_to_string(file).with_context(|| format!("read {}", file))?;
        files.push(scan_file_text(file, &source));
    }

    let response = ScanResponse {
        schema_version: 1,
        policy_version: POLICY_VERSION.to_string(),
        timing: Timing {
            files: files.len(),
            elapsed_ms: started.elapsed().as_millis(),
        },
        files,
    };
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}
