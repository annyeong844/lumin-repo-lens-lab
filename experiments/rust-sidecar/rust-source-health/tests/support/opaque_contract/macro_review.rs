use crate::artifact::analyze_file;
use crate::opaque::summary::{assert_macro_call_count, assert_opaque_totals};
use crate::opaque::surfaces::assert_opaque_detail;

pub fn assert_risky_macro_opaque_surfaces_stay_reviewable() {
    let source = r#"
fn main() {
    panic!("boom");
    custom_macro!();
}
"#;
    let artifact = analyze_file("src/lib.rs", source);

    assert_macro_call_count(&artifact, 2);
    assert_opaque_totals(&artifact, 2, 2, 0);
    assert_opaque_detail(&artifact, "src/lib.rs", "panic", "review", None);
    assert_opaque_detail(&artifact, "src/lib.rs", "custom_macro", "review", None);
}
