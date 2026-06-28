use crate::artifact::analyze_file;
use crate::opaque::summary::{
    assert_macro_call_count, assert_muted_opaque_reason_count, assert_opaque_totals,
};

pub fn assert_muted_common_macro_opaque_surfaces_are_summarized() {
    let source = r#"
fn main() {
    assert!(true);
    let _ = vec![1, 2, 3];
    let _ = serde_json::json!({"ok": true});
    let _ = format!("value={}", 1);
    tracing::warn!("slow");
}
"#;
    let artifact = analyze_file("src/lib.rs", source);

    assert_macro_call_count(&artifact, 5);
    assert_opaque_totals(&artifact, 5, 0, 5);
    assert_muted_opaque_reason_count(&artifact, "assertion-macro", 1);
    assert_muted_opaque_reason_count(&artifact, "collection-macro", 1);
    assert_muted_opaque_reason_count(&artifact, "data-literal-macro", 1);
    assert_muted_opaque_reason_count(&artifact, "formatting-macro", 1);
    assert_muted_opaque_reason_count(&artifact, "logging-macro", 1);
}
