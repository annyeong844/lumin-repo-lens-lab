use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use super::io_support::{
    read_json_input, take_path, take_string, write_json_file, write_stdout_json,
};
use super::usage::USAGE;
use lumin_audit_core::resolver_diagnostics_artifacts::{
    build_resolver_diagnostics_artifacts, ResolverDiagnosticsArtifactsRequest,
};

pub(super) fn run_resolver_diagnostics_artifacts(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output: Option<PathBuf> = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("resolver-diagnostics-artifacts: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("resolver-diagnostics-artifacts: missing --input <path|->")?;
    let json = read_json_input(&input, "resolver-diagnostics-artifacts")?;
    let request = serde_json::from_value::<ResolverDiagnosticsArtifactsRequest>(json)
        .context("resolver-diagnostics-artifacts: invalid request shape")?;
    let artifacts = build_resolver_diagnostics_artifacts(request)?;
    if let Some(path) = result_output {
        write_json_file(&path, &artifacts)
    } else {
        write_stdout_json(&artifacts)
    }
}
