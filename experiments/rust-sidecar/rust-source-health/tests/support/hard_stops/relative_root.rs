use serde_json::json;

pub fn assert_relative_root_is_rejected() {
    let mut value =
        crate::request::request(vec![crate::request::file("src/lib.rs", "fn main() {}")]);
    value["root"] = json!("relative/repo");

    let output = crate::cli::run_sidecar(value);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("root must be absolute"));
}
