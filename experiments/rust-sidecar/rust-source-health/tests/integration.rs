#[path = "support/artifact.rs"]
mod artifact;
#[path = "support/cli_usage_contract/assert_usage.rs"]
mod assert_usage;
#[path = "support/assertions.rs"]
mod assertions;
#[path = "support/ast_facts.rs"]
mod ast_facts;
#[path = "support/cli.rs"]
mod cli;
#[path = "support/cli_artifact_contract.rs"]
mod cli_artifact_contract;
#[path = "support/hard_stops/duplicate_path.rs"]
mod duplicate_path;
#[path = "support/hard_stops/input_integrity.rs"]
mod input_integrity;
#[path = "support/opaque_contract/macro_review.rs"]
mod macro_review_contract;
#[path = "support/opaque_contract/macro_summary.rs"]
mod macro_summary_contract;
#[path = "support/cli_usage_contract/missing_root.rs"]
mod missing_root;
#[path = "support/opaque/mod.rs"]
mod opaque;
#[path = "support/cli_usage_contract/output_without_root.rs"]
mod output_without_root;
#[path = "support/path_classification_contract.rs"]
mod path_classification_contract;
#[path = "support/hard_stops/relative_root.rs"]
mod relative_root;
#[path = "support/request.rs"]
mod request;
#[path = "support/hard_stops/request_contract.rs"]
mod request_contract;
#[path = "support/hard_stops/runtime_contract.rs"]
mod runtime_contract;
#[path = "support/signal_visibility_contract.rs"]
mod signal_visibility_contract;
#[path = "support/signals/mod.rs"]
mod signals;
#[path = "support/syntax_review_contract.rs"]
mod syntax_review_contract;
#[path = "support/cli_usage_contract/unknown_flag.rs"]
mod unknown_flag;
#[path = "support/hard_stops/unsafe_file_path.rs"]
mod unsafe_file_path;
#[path = "support/cli_usage_contract/zero_threads.rs"]
mod zero_threads;

#[path = "integration/cli_hard_stops.rs"]
mod cli_hard_stops;
#[path = "integration/file_and_parse.rs"]
mod file_and_parse;
#[path = "integration/function_body_fingerprints.rs"]
mod function_body_fingerprints;
#[path = "integration/opaque_surfaces.rs"]
mod opaque_surfaces;
#[path = "integration/path_visibility.rs"]
mod path_visibility;
#[path = "integration/syntax_signals.rs"]
mod syntax_signals;
#[path = "integration/unused_definitions.rs"]
mod unused_definitions;
