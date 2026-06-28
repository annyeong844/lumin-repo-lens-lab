use crate::assert_usage::assert_usage_error;

#[test]
fn zero_threads_exits_2_without_json_artifact() {
    assert_usage_error(&[
        "--root",
        ".",
        "--output",
        "out.json",
        "--source-commit",
        "test-source-commit",
        "--threads",
        "0",
    ]);
}
