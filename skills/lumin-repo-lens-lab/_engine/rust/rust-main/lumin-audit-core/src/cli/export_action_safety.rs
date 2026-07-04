use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use super::io_support::{
    read_json_input, take_path, take_string, write_json_file, write_stdout_json,
};
use super::usage::USAGE;
use lumin_audit_core::export_action_safety::{
    build_export_action_safety_artifact, ExportActionSafetyRequest,
};

pub(super) fn run_export_action_safety_artifact(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output: Option<PathBuf> = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("export-action-safety-artifact: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("export-action-safety-artifact: missing --input <path|->")?;
    let json = read_json_input(&input, "export-action-safety-artifact")?;
    let request = serde_json::from_value::<ExportActionSafetyRequest>(json)
        .context("export-action-safety-artifact: invalid request shape")?;
    let artifact = build_export_action_safety_artifact(request)?;
    if let Some(path) = result_output {
        write_json_file(&path, &artifact)
    } else {
        write_stdout_json(&artifact)
    }
}
