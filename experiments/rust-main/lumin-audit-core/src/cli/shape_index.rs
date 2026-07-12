use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use super::io_support::{
    read_json_input, take_path, take_string, write_json_file, write_stdout_json,
};
use super::usage::USAGE;
use lumin_audit_core::shape_index::{build_shape_index_artifact, ShapeIndexRequest};

pub(super) fn run_shape_index_artifact(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output: Option<PathBuf> = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("shape-index-artifact: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("shape-index-artifact: missing --input <path|->")?;
    let json = read_json_input(&input, "shape-index-artifact")?;
    let request = serde_json::from_value::<ShapeIndexRequest>(json)
        .context("shape-index-artifact: invalid request shape")?;
    let artifact = build_shape_index_artifact(request)?;
    if let Some(path) = result_output {
        write_json_file(&path, &artifact)
    } else {
        write_stdout_json(&artifact)
    }
}
