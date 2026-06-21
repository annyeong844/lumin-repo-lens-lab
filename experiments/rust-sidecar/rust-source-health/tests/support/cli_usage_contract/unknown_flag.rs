use crate::assert_usage::assert_usage_error;

#[test]
fn unknown_flag_exits_2_without_json_artifact() {
    assert_usage_error(&["--unknown"]);
}

#[test]
fn invalid_artifact_profile_exits_2_without_json_artifact() {
    assert_usage_error(&[
        "--root",
        ".",
        "--output",
        "rust-health.json",
        "--source-commit",
        "test-source-commit",
        "--artifact-profile",
        "raw",
    ]);
}
