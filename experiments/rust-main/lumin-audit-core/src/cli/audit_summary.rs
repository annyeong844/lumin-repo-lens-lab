use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use super::io_support::{
    read_json_input, take_path, take_string, write_json_file, write_stdout_json, write_text_file,
};
use super::usage::USAGE;
use lumin_audit_core::audit_summary::{render_audit_summary_request, AuditSummaryRenderRequest};

pub(super) fn run_audit_summary_render(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output: Option<PathBuf> = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("audit-summary-render: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("audit-summary-render: missing --input <path|->")?;
    let json = read_json_input(&input, "audit-summary-render")?;
    let request = serde_json::from_value::<AuditSummaryRenderRequest>(json)
        .context("audit-summary-render: invalid request shape")?;
    let output_path = PathBuf::from(&request.output_path);
    let (markdown, result) = render_audit_summary_request(&request)?;
    write_text_file(&output_path, &markdown)?;
    if let Some(path) = result_output {
        write_json_file(&path, &result)
    } else {
        write_stdout_json(&result)
    }
}
