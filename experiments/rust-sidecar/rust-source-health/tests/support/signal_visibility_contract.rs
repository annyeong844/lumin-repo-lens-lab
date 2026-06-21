use crate::artifact::analyze_file;
use crate::signals::summary::{
    assert_muted_signal_reason_count, assert_review_signal_kind_count, assert_signal_kind_count,
    assert_signal_totals,
};
use crate::signals::visibility::{assert_first_signal_visibility, assert_signal_visibility_count};

pub fn assert_cfg_test_module_signals_are_muted() {
    let source = r#"
#[cfg(test)]
mod tests {
    fn helper() {
        let value = Some(1);
        let _ = value.unwrap();
        panic!("boom");
    }
}

fn live() {
    let value = Some(1);
    let _ = value.unwrap();
}
"#;

    let artifact = analyze_file("src/lib.rs", source);

    assert_signal_totals(&artifact, 3, 1, 2);
    assert_signal_kind_count(&artifact, "unwrap-call", 2);
    assert_signal_kind_count(&artifact, "panic-macro", 1);
    assert_review_signal_kind_count(&artifact, "unwrap-call", 1);
    assert_muted_signal_reason_count(&artifact, "cfg-test", 2);
    assert_signal_visibility_count(&artifact, "src/lib.rs", "muted", Some("cfg-test"), 2);
    assert_signal_visibility_count(&artifact, "src/lib.rs", "review", None, 1);
}

pub fn assert_generated_path_unwrap_is_muted() {
    let source = "fn main() { let value = Some(1); let _ = value.unwrap(); }";
    let artifact = analyze_file("generated/bindings.rs", source);

    assert_signal_totals(&artifact, 1, 0, 1);
    assert_muted_signal_reason_count(&artifact, "generated-path", 1);
    assert_first_signal_visibility(
        &artifact,
        "generated/bindings.rs",
        "muted",
        Some("generated-path"),
    );
}

pub fn assert_source_path_unwrap_stays_reviewable() {
    let source = "fn main() { let value = Some(1); let _ = value.unwrap(); }";
    let artifact = analyze_file("src/lib.rs", source);

    assert_signal_totals(&artifact, 1, 1, 0);
    assert_first_signal_visibility(&artifact, "src/lib.rs", "review", None);
}

pub fn assert_test_attribute_function_signals_are_muted() {
    let source = r#"
#[test]
fn parses() {
    let value = Some(1);
    let _ = value.expect("value");
}

fn live() {
    let value = Some(1);
    let _ = value.expect("value");
}
"#;

    let artifact = analyze_file("src/lib.rs", source);

    assert_signal_totals(&artifact, 2, 1, 1);
    assert_signal_kind_count(&artifact, "expect-call", 2);
    assert_review_signal_kind_count(&artifact, "expect-call", 1);
    assert_muted_signal_reason_count(&artifact, "test-attribute", 1);
    assert_signal_visibility_count(&artifact, "src/lib.rs", "muted", Some("test-attribute"), 1);
    assert_signal_visibility_count(&artifact, "src/lib.rs", "review", None, 1);
}

pub fn assert_test_path_unwrap_is_muted() {
    let source = "fn main() { let value = Some(1); let _ = value.unwrap(); }";
    let artifact = analyze_file("tests/integration.rs", source);

    assert_signal_totals(&artifact, 1, 0, 1);
    assert_muted_signal_reason_count(&artifact, "test-path", 1);
    assert_first_signal_visibility(
        &artifact,
        "tests/integration.rs",
        "muted",
        Some("test-path"),
    );
}
