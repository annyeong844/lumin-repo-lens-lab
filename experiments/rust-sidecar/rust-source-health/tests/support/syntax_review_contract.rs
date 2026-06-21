use crate::artifact::{
    analyze_file, assert_file_fact_count, assert_file_parse_ok, assert_syntax_artifact_metadata,
};
use crate::signals::summary::{assert_signal_kind_count, assert_signal_totals};
use crate::signals::visibility::assert_first_signal_visibility;

pub fn assert_reports_syntax_facts_and_review_signals() {
    let source = r#"
fn main() {
    let value = Some(String::from("x"));
    let cloned = value.clone();
    let _ = cloned.expect("value");
    unsafe {
        do_thing();
    }
    panic!("boom");
}

unsafe fn do_thing() {}
"#;

    let artifact = analyze_file("src/lib.rs", source);

    assert_syntax_artifact_metadata(&artifact);
    assert_file_parse_ok(&artifact, "src/lib.rs", source);
    assert_first_signal_visibility(&artifact, "src/lib.rs", "review", None);
    assert_file_fact_count(&artifact, "src/lib.rs", "unsafeBlocks", 1);
    assert_file_fact_count(&artifact, "src/lib.rs", "unsafeFunctions", 1);
    assert_signal_kind_count(&artifact, "clone-call", 1);
    assert_signal_kind_count(&artifact, "expect-call", 1);
    assert_signal_kind_count(&artifact, "panic-macro", 1);
    assert_signal_kind_count(&artifact, "unsafe-block", 1);
    assert_signal_totals(&artifact, 4, 4, 0);
}
