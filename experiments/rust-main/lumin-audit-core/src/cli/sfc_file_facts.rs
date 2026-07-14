use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use super::io_support::{
    read_json_input, take_path, take_string, write_json_file, write_stdout_json,
};
use super::usage::USAGE;
use lumin_audit_core::sfc_file_facts::{build_sfc_file_facts_response, SfcFileFactsRequest};

pub(super) fn run_sfc_file_facts_artifact(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output: Option<PathBuf> = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("sfc-file-facts-artifact: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("sfc-file-facts-artifact: missing --input <path|->")?;
    let json = read_json_input(&input, "sfc-file-facts-artifact")?;
    let request = serde_json::from_value::<SfcFileFactsRequest>(json)
        .context("sfc-file-facts-artifact: invalid request shape")?;
    let artifact = build_sfc_file_facts_response(request)?;
    if let Some(path) = result_output {
        write_json_file(&path, &artifact)
    } else {
        write_stdout_json(&artifact)
    }
}
