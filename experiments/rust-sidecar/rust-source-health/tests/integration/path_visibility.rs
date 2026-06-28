use crate::{path_classification_contract, signal_visibility_contract};

#[test]
fn keeps_source_paths_that_only_contain_policy_words() {
    path_classification_contract::assert_source_paths_with_policy_words_stay_source();
}

#[test]
fn classifies_generated_paths_without_substring_matching() {
    path_classification_contract::assert_generated_paths_do_not_use_substring_matching();
}

#[test]
fn classifies_test_like_paths() {
    path_classification_contract::assert_test_like_paths_are_classified_as_test();
}

#[test]
fn mutes_cfg_test_module_signals_without_dropping_raw_evidence() {
    signal_visibility_contract::assert_cfg_test_module_signals_are_muted();
}

#[test]
fn generated_path_unwrap_signal_is_muted_without_dropping_raw_evidence() {
    signal_visibility_contract::assert_generated_path_unwrap_is_muted();
}

#[test]
fn source_path_unwrap_signal_stays_reviewable() {
    signal_visibility_contract::assert_source_path_unwrap_stays_reviewable();
}

#[test]
fn mutes_test_attribute_function_signals_without_dropping_raw_evidence() {
    signal_visibility_contract::assert_test_attribute_function_signals_are_muted();
}

#[test]
fn test_path_unwrap_signal_is_muted_without_dropping_raw_evidence() {
    signal_visibility_contract::assert_test_path_unwrap_is_muted();
}
