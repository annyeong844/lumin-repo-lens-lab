use anyhow::Result;

use crate::{
    cli_artifact_contract, duplicate_path, input_integrity, relative_root, request_contract,
    runtime_contract, unsafe_file_path,
};

#[test]
fn cli_collects_sources_and_writes_final_artifact_without_node_wrapper() -> Result<()> {
    cli_artifact_contract::assert_cli_collects_sources_and_writes_final_artifact_without_node_wrapper(
    )
}

#[test]
fn cli_full_profile_preserves_raw_ast_fact_artifact() -> Result<()> {
    cli_artifact_contract::assert_cli_full_profile_preserves_raw_ast_fact_artifact()
}

#[test]
fn rejects_duplicate_paths_without_json_artifact() {
    duplicate_path::assert_duplicate_paths_are_rejected();
}

#[test]
fn rejects_mismatched_sha256_without_json_artifact() {
    input_integrity::assert_mismatched_sha_is_rejected();
}

#[test]
fn rejects_relative_root_without_json_artifact() {
    relative_root::assert_relative_root_is_rejected();
}

#[test]
fn rejects_unsupported_schema_without_json_artifact() {
    request_contract::assert_unsupported_schema_is_rejected();
}

#[test]
fn rejects_unsupported_parser_policy_without_json_artifact() {
    request_contract::assert_unsupported_parser_policy_is_rejected();
}

#[test]
fn rejects_invalid_runtime_stack_without_json_artifact() {
    runtime_contract::assert_invalid_worker_stack_is_rejected();
}

#[test]
fn rejects_unsafe_file_paths_without_json_artifact() {
    unsafe_file_path::assert_unsafe_file_paths_are_rejected();
}
