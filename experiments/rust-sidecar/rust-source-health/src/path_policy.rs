pub fn is_test_like_rust_path(path: &str) -> bool {
    let base = path.rsplit_once('/').map_or(path, |(_, name)| name);
    if matches!(base, "test.rs" | "tests.rs")
        || base.ends_with(".test.rs")
        || base.ends_with(".spec.rs")
        || base.ends_with("_test.rs")
        || base.ends_with("_tests.rs")
    {
        return true;
    }

    path.split('/').any(|segment| {
        matches!(
            segment,
            "test"
                | "tests"
                | "e2e"
                | "integration"
                | "fixtures"
                | "fixture"
                | "mocks"
                | "mock"
                | "test-support"
                | "test-utils"
                | "runtime-tests"
                | "playground"
                | "playgrounds"
                | "example"
                | "examples"
                | "bench"
                | "benches"
        ) || (segment.len() >= 4 && segment.starts_with("__") && segment.ends_with("__"))
            || segment.ends_with("-fixture")
            || segment.ends_with("-fixtures")
    })
}
