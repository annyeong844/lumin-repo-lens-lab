use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use super::io_support::{
    read_json_input, take_path, take_string, write_json_file, write_stdout_json,
};
use super::usage::USAGE;
use lumin_audit_core::js_ts_pre_write::{
    start_js_ts_pre_write_evidence, JsTsPreWriteEvidenceRequest,
};

pub(super) fn run_js_ts_pre_write_evidence(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output: Option<PathBuf> = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("js-ts-pre-write-evidence: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("js-ts-pre-write-evidence: missing --input <path|->")?;
    let json = read_json_input(&input, "js-ts-pre-write-evidence")?;
    let request = serde_json::from_value::<JsTsPreWriteEvidenceRequest>(json)
        .context("js-ts-pre-write-evidence: invalid request shape")?;
    let evidence = start_js_ts_pre_write_evidence(request)?.into_evidence();
    if let Some(path) = result_output {
        write_json_file(&path, &evidence)
    } else {
        write_stdout_json(&evidence)
    }
}
