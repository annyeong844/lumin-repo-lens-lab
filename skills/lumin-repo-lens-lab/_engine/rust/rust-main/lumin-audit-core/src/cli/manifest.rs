use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;

mod base_evidence;
mod evidence;
mod finalize;
mod protocol;
mod root;
mod updates;

pub(super) use evidence::{
    run_manifest_evidence_refresh, run_manifest_evidence_refresh_with_reads,
    run_manifest_evidence_summary, run_manifest_evidence_summary_with_reads,
};
pub(super) use finalize::{
    run_finalize_audit_run, run_finalize_audit_run_with_companions, run_manifest_closeout_write,
    run_manifest_write,
};
pub(super) use root::{
    run_manifest_lifecycle_evidence_refresh, run_manifest_meta, run_manifest_root,
    run_manifest_root_with_evidence,
};
pub(super) use updates::{
    run_manifest_artifacts_produced_update, run_manifest_closeout_update,
    run_manifest_companion_update, run_manifest_core_summary, run_manifest_evidence_update,
    run_manifest_final_summary_update,
};

use super::io_support::{write_json_file, write_stdout_json};
use base_evidence::{mark_base_evidence_not_refreshed, required_base_pipeline_skip_reason};

fn write_json_result<T: Serialize>(result_output: Option<PathBuf>, value: &T) -> Result<()> {
    if let Some(result_output) = result_output {
        write_json_file(&result_output, value)
    } else {
        write_stdout_json(value)
    }
}
