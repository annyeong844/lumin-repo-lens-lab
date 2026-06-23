use lumin_rust_common::posix_path_has_segment;

use crate::protocol::PathClassification;

pub(super) fn classify_path(path: &str) -> Vec<PathClassification> {
    if posix_path_has_segment(path, "generated") || file_name(path) == "generated.rs" {
        vec![PathClassification::Generated]
    } else if is_test_like_path(path) {
        vec![PathClassification::Test]
    } else {
        vec![PathClassification::Source]
    }
}

fn is_test_like_path(path: &str) -> bool {
    let base = file_name(path);
    if base == "tests.rs"
        || base == "test.rs"
        || base.ends_with("_test.rs")
        || base.ends_with("_tests.rs")
        || base.ends_with(".test.rs")
        || base.ends_with(".spec.rs")
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
                | "examples"
                | "example"
                | "benches"
                | "bench"
        ) || (segment.len() >= 4 && segment.starts_with("__") && segment.ends_with("__"))
            || segment.ends_with("-fixture")
            || segment.ends_with("-fixtures")
    })
}

fn file_name(path: &str) -> &str {
    path.rsplit_once('/').map_or(path, |(_, name)| name)
}
