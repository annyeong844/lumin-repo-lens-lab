pub fn assert_duplicate_paths_are_rejected() {
    let output = crate::cli::run_sidecar(crate::request::request(vec![
        crate::request::file("src/lib.rs", "fn one() {}"),
        crate::request::file("src/lib.rs", "fn two() {}"),
    ]));

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate file path"));
}
