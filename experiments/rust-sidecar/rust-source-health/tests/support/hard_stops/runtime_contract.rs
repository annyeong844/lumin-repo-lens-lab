use serde_json::json;

pub fn assert_invalid_worker_stack_is_rejected() {
    let mut value =
        crate::request::request(vec![crate::request::file("src/lib.rs", "fn main() {}")]);
    value["runtime"]["workerStackBytes"] = json!(1);

    let output = crate::cli::run_sidecar(value);

    crate::assertions::assert_exit_code(&output, 2);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("runtime.workerStackBytes"));
}
