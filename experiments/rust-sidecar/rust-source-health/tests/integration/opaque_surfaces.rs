use crate::artifact::analyze_file;
use crate::opaque::{
    self,
    summary::assert_muted_opaque_reason_count,
    surfaces::{assert_opaque_detail, assert_opaque_visibility_count},
};
use crate::{macro_review_contract, macro_summary_contract};

#[test]
fn emits_review_and_muted_opaque_surfaces_for_oracle_escalation() {
    let source = r#"
#[cfg(feature = "fast")]
pub fn gated() {
    custom_macro!();
}

#[cfg_attr(test, allow(dead_code))]
fn test_only_attr() {}
"#;
    let artifact = analyze_file("src/lib.rs", source);

    opaque::assert_review_and_muted_surfaces(&artifact, "src/lib.rs");
}

#[test]
fn keeps_muted_macro_opaque_surface_evidence() {
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

    assert_opaque_visibility_count(&artifact, "src/lib.rs", "muted", 5);
}

#[test]
fn keeps_risky_macro_opaque_surfaces_for_review() {
    macro_review_contract::assert_risky_macro_opaque_surfaces_stay_reviewable();
}

#[test]
fn mutes_builtin_derive_macro_opaque_surface_evidence() {
    let source = r#"
#[derive(Debug, Clone, Default)]
pub struct Demo {
    value: i32,
}
"#;
    let artifact = analyze_file("src/lib.rs", source);

    assert_opaque_visibility_count(&artifact, "src/lib.rs", "muted", 1);
    assert_muted_opaque_reason_count(&artifact, "builtin-derive-macro", 1);
    assert_opaque_detail(
        &artifact,
        "src/lib.rs",
        "derive(Debug,Clone,Default)",
        "muted",
        Some("builtin-derive-macro"),
    );
}

#[test]
fn keeps_custom_derive_and_attribute_macros_for_review() {
    let source = r#"
#[derive(CustomDerive)]
pub struct Demo;

#[tokio::main]
async fn main() {}
"#;
    let artifact = analyze_file("src/lib.rs", source);

    assert_opaque_visibility_count(&artifact, "src/lib.rs", "review", 2);
    assert_opaque_detail(
        &artifact,
        "src/lib.rs",
        "derive(CustomDerive)",
        "review",
        None,
    );
    assert_opaque_detail(&artifact, "src/lib.rs", "tokio::main", "review", None);
}

#[test]
fn ignores_inert_lint_and_tool_attributes_without_muting_custom_attribute_macros() {
    let source = r#"
#![warn(missing_docs)]
#![expect(dead_code)]

#[rustfmt::skip]
#[clippy::msrv = "1.70"]
pub fn plain() {}

#[custom_attr]
pub fn expanded() {}
"#;
    let artifact = analyze_file("src/lib.rs", source);

    assert_opaque_visibility_count(&artifact, "src/lib.rs", "review", 1);
    assert_opaque_detail(&artifact, "src/lib.rs", "custom_attr", "review", None);
}

#[test]
fn ignores_derive_helper_attributes_without_hiding_parent_derive_surface() {
    let source = r#"
#[derive(Serialize, Deserialize, JsonSchema, TS, ExperimentalApi)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
#[ts(export)]
pub struct WireShape {
    #[serde(default)]
    #[ts(optional)]
    value: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum DemoError {
    #[error("bad input: {0}")]
    BadInput(#[from] anyhow::Error),
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WireMessage {
    #[prost(string, tag = "1")]
    value: String,
}
"#;
    let artifact = analyze_file("src/lib.rs", source);

    assert_opaque_visibility_count(&artifact, "src/lib.rs", "review", 1);
    assert_opaque_visibility_count(&artifact, "src/lib.rs", "muted", 2);
    assert_muted_opaque_reason_count(&artifact, "known-data-derive-macro", 2);
    assert_opaque_detail(
        &artifact,
        "src/lib.rs",
        "derive(Serialize,Deserialize,JsonSchema,TS,ExperimentalApi)",
        "muted",
        Some("known-data-derive-macro"),
    );
    assert_opaque_detail(
        &artifact,
        "src/lib.rs",
        "derive(Clone,PartialEq,::prost::Message)",
        "muted",
        Some("known-data-derive-macro"),
    );
    assert_opaque_detail(
        &artifact,
        "src/lib.rs",
        "derive(Debug,thiserror::Error)",
        "review",
        None,
    );
}

#[test]
fn mutes_async_test_attribute_macro_and_body_opaque_surfaces() {
    let source = r#"
#[tokio::test]
async fn exercises_async_path() {
    custom_macro!();
    panic!("boom");
}
"#;
    let artifact = analyze_file("src/lib.rs", source);

    assert_opaque_visibility_count(&artifact, "src/lib.rs", "review", 0);
    assert_opaque_visibility_count(&artifact, "src/lib.rs", "muted", 3);
    assert_muted_opaque_reason_count(&artifact, "test-attribute", 3);
}

#[test]
fn mutes_plural_test_module_file_opaque_surfaces() {
    let source = r#"
#[cfg(debug_assertions)]
fn debug_only() {}

fn helper() {
    custom_macro!();
}
"#;
    let artifact = analyze_file("src/client_tests.rs", source);

    assert_opaque_visibility_count(&artifact, "src/client_tests.rs", "review", 0);
    assert_opaque_visibility_count(&artifact, "src/client_tests.rs", "muted", 2);
    assert_muted_opaque_reason_count(&artifact, "test-path", 2);
}

#[test]
fn summarizes_muted_common_macro_opaque_surfaces() {
    macro_summary_contract::assert_muted_common_macro_opaque_surfaces_are_summarized();
}
