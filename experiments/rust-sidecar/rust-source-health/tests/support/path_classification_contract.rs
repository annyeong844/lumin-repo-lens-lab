use crate::cli::{run_sidecar, stdout_json};
use crate::request::{file, request};
use serde_json::json;

pub fn assert_source_paths_with_policy_words_stay_source() {
    let artifact = stdout_json(run_sidecar(request(vec![
        file("src/notgenerated.rs", "fn source() {}"),
        file("src/contest.rs", "fn contest() {}"),
    ])));

    assert_eq!(
        artifact["files"]["src/notgenerated.rs"]["path"]["classifications"],
        json!(["source"])
    );
    assert_eq!(
        artifact["files"]["src/contest.rs"]["path"]["classifications"],
        json!(["source"])
    );
}

pub fn assert_generated_paths_do_not_use_substring_matching() {
    let artifact = stdout_json(run_sidecar(request(vec![file(
        "generated/bindings.rs",
        "fn generated() {}",
    )])));

    assert_eq!(
        artifact["files"]["generated/bindings.rs"]["path"]["classifications"],
        json!(["generated"])
    );
}

pub fn assert_test_like_paths_are_classified_as_test() {
    let artifact = stdout_json(run_sidecar(request(vec![
        file("tests/integration.rs", "fn integration() {}"),
        file("src/migrate/tests.rs", "fn module_tests() {}"),
        file("fixtures/sample.rs", "fn fixture() {}"),
        file("src/__mocks__/client.rs", "fn mock_client() {}"),
        file("examples/demo.rs", "fn example() {}"),
        file("benches/walk.rs", "fn bench() {}"),
    ])));

    for path in [
        "tests/integration.rs",
        "src/migrate/tests.rs",
        "fixtures/sample.rs",
        "src/__mocks__/client.rs",
        "examples/demo.rs",
        "benches/walk.rs",
    ] {
        assert_eq!(
            artifact["files"][path]["path"]["classifications"],
            json!(["test"])
        );
    }
}
