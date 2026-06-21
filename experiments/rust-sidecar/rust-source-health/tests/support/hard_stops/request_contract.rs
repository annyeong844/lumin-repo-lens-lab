use serde_json::json;

pub fn assert_unsupported_schema_is_rejected() {
    let mut value =
        crate::request::request(vec![crate::request::file("src/lib.rs", "fn main() {}")]);
    value["schemaVersion"] = json!(999);

    let output = crate::cli::run_sidecar(value);

    crate::assertions::assert_exit_code(&output, 2);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported schemaVersion"));
}

pub fn assert_unsupported_parser_policy_is_rejected() {
    let mut value =
        crate::request::request(vec![crate::request::file("src/lib.rs", "fn main() {}")]);
    value["parser"]["edition"] = json!("2024");

    let output = crate::cli::run_sidecar(value);

    crate::assertions::assert_exit_code(&output, 2);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported parser edition policy"));
}
