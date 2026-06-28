pub fn assert_unsafe_file_paths_are_rejected() {
    for bad_path in [
        "src\\lib.rs",
        "C:/repo/src/lib.rs",
        "src//lib.rs",
        "./src/lib.rs",
        "src/../lib.rs",
    ] {
        let output = crate::cli::run_sidecar(crate::request::request(vec![crate::request::file(
            bad_path,
            "fn main() {}",
        )]));

        assert!(!output.status.success(), "path should fail: {bad_path}");
        assert!(
            output.stdout.is_empty(),
            "path should not emit JSON: {bad_path}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("file path"),
            "stderr should mention file path for {bad_path}: {stderr}",
            stderr = String::from_utf8_lossy(&output.stderr)
        );
    }
}
