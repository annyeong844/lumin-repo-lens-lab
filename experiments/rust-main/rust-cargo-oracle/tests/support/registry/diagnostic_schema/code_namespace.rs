use serde_json::{json, Value};

pub fn assert_code_namespace(schema: &Value) {
    assert_eq!(
        schema["codeNamespace"],
        json!([
            "rustc-codeless",
            "rustc-error",
            "rustc-non-ecode",
            "unknown"
        ])
    );
}
