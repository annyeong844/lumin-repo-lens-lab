# M7 Rust Cargo Oracle Structure Verification - 2026-06-17

This note records the local verification for the M7 Rust cargo oracle structure
refactor.

## Commands

All commands were run from:

```text
C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab
```

### Cargo Oracle Tests

```text
cargo fmt --manifest-path experiments\rust-main\rust-cargo-oracle\Cargo.toml
cargo test --manifest-path experiments\rust-main\rust-cargo-oracle\Cargo.toml
```

Result: PASS.

Observed test groups:

```text
classify::tests::derives_code_namespaces ... ok
classify::tests::metadata_unavailable_root_src_error_remains_user_finding ... ok
classify::tests::timeout_preserves_already_emitted_json_messages ... ok
command::tests::cargo_metadata_args_follow_check_feature_selection ... ok
command::tests::cargo_metadata_args_omit_features_when_unselected ... ok
config::tests::cargo_config_paths_prefer_deeper_project_config_over_parent ... ok
config::tests::cargo_config_paths_prefer_extensionless_config_in_same_directory ... ok
config::tests::malformed_quoted_build_value_is_ignored_without_panic ... ok
ownership::tests::metadata_unavailable_fallback_does_not_mark_dependency_shaped_path_user_code ... ok
ownership::tests::metadata_unavailable_fallback_keeps_root_src_diagnostic_user_code ... ok
scope::tests::target_triple_prefers_deeper_project_config_over_parent ... ok
scope::tests::target_triple_prefers_extensionless_config_over_config_toml ... ok

oracle_runner.rs:
- analysis_input_hash_changes_when_cargo_config_changes ... ok
- analysis_input_hash_changes_when_rustflags_changes ... ok
- artifact_marks_analysis_input_set_as_incomplete_for_reuse ... ok
- build_finished_without_success_does_not_prove_clean ... ok
- dependency_events_do_not_replace_selected_scope_target ... ok
- dependency_primary_error_is_coverage_unavailable_not_user_finding ... ok
- dependency_package_id_blocks_user_finding_even_when_metadata_omits_dependency_package ... ok
- empty_cargo_stdout_is_unavailable_not_ran_stream ... ok
- e_code_user_error_is_verified_rustc_error_diagnostic ... ok
- large_cargo_stdout_is_drained_while_process_runs ... ok
- multi_target_fallback_scope_does_not_pick_an_arbitrary_target ... ok
- provenance_records_the_actual_cargo_binary ... ok
- warning_lint_is_rule_backed_without_blocking_verified_error_clean ... ok

registry_contract.rs:
- classifier_claim_kinds_are_declared_in_registry ... ok
- classifier_rules_are_declared_in_registry ... ok
- coverage_kind_and_absence_claim_sets_match_registry ... ok
- documents_normalized_cargo_code_derivation ... ok
- does_not_promote_footer_or_non_user_diagnostics_to_user_findings ... ok
- keeps_clean_scoped_to_verified_rustc_error_absence ... ok
- registry_diagnostic_rules_are_known_to_rust_code ... ok
- requires_input_identity_to_include_local_rust_source_bytes ... ok
- uses_cargo_diagnostic_claim_vocabulary ... ok
```

No compiler warnings were emitted after the final import cleanup.

### Rust Source Health Tests

```text
cargo fmt --manifest-path experiments\rust-sidecar\rust-source-health\Cargo.toml
cargo test --manifest-path experiments\rust-sidecar\rust-source-health\Cargo.toml
```

Result: PASS.

Observed test groups:

```text
Unit:
- locations::tests::reports_one_based_byte_columns ... ok
- locations::tests::reports_ranges_across_lines ... ok

Integration:
- classifies_root_level_test_and_generated_paths ... ok
- does_not_emit_method_signals_for_plain_identifiers ... ok
- emits_files_in_deterministic_path_order ... ok
- records_parse_errors_as_file_data ... ok
- rejects_duplicate_paths_without_json_artifact ... ok
- rejects_invalid_runtime_stack_without_json_artifact ... ok
- rejects_mismatched_sha256_without_json_artifact ... ok
- rejects_relative_root_without_json_artifact ... ok
- rejects_unsupported_parser_policy_without_json_artifact ... ok
- rejects_unsupported_schema_without_json_artifact ... ok
- reports_macro_signals_from_ast_paths ... ok
- reports_syntax_facts_and_review_signals ... ok
- rejects_unsafe_file_paths_without_json_artifact ... ok
- cli_usage_errors_exit_2_without_json_artifact ... ok
- cli_collects_sources_and_writes_final_artifact_without_node_wrapper ... ok
```

### Diff Hygiene

```text
git diff --check
```

Result: PASS for whitespace errors.

Git printed LF-to-CRLF conversion warnings for existing working-tree files. No
whitespace error was reported.

## Workspace Notes

`experiments/rust-main/` and the M7 fixture directory are part of the staged
change set. They must remain included before handing a fresh checkout to a
reviewer or CI.

The main/reference repository was not used for this verification.
