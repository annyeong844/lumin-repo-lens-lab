pub fn assert_mismatched_sha_is_rejected() {
    let output = crate::cli::run_sidecar(crate::request::request(vec![
        crate::request::file_with_sha(
            "src/lib.rs",
            "fn main() {}",
            &format!("sha256:{}", "0".repeat(64)),
        ),
    ]));

    crate::assertions::assert_exit_code(&output, 2);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("sha256/text mismatch"));
}
