use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use super::io_support::{
    read_json_input, take_path, take_string, write_json_file, write_stdout_json,
};
use super::usage::USAGE;
use lumin_audit_core::function_clones::{build_function_clones_artifact, FunctionClonesRequest};

pub(super) fn run_function_clones_artifact(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output: Option<PathBuf> = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("function-clones-artifact: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("function-clones-artifact: missing --input <path|->")?;
    let json = read_json_input(&input, "function-clones-artifact")?;
    let request = serde_json::from_value::<FunctionClonesRequest>(json)
        .context("function-clones-artifact: invalid request shape")?;
    let artifact = build_function_clones_artifact(request)?;
    if let Some(path) = result_output {
        write_json_file(&path, &artifact)
    } else {
        write_stdout_json(&artifact)
    }
}
