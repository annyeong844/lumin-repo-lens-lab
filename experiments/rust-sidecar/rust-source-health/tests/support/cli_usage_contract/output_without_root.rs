use crate::assert_usage::assert_usage_error;

#[test]
fn output_without_root_exits_2_without_json_artifact() {
    assert_usage_error(&[
        "--output",
        "out.json",
        "--source-commit",
        "test-source-commit",
    ]);
}
