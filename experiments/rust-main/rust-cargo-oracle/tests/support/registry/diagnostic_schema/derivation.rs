use serde_json::{json, Value};

pub fn assert_code_namespace_derivation(schema: &Value) {
    assert_eq!(
        schema["codeNamespaceDerivation"],
        json!([
            {
                "when": "message.code === null",
                "codeNamespace": "rustc-codeless",
                "codeKind": "null-error-code"
            },
            {
                "when": "message.code.code matches ^E[0-9]+$",
                "codeNamespace": "rustc-error",
                "codeKind": "rustc-error-code"
            },
            {
                "when": "message.code.code is any other non-empty string",
                "codeNamespace": "rustc-non-ecode",
                "codeKind": "non-ecode-name"
            },
            {
                "when": "message.code is missing or malformed",
                "codeNamespace": "unknown",
                "codeKind": "unknown"
            }
        ])
    );
}
