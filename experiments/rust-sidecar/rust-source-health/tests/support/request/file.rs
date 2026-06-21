use lumin_rust_common::sha256_text;
use serde_json::{json, Value};

pub fn file(path: &str, text: &str) -> Value {
    json!({
        "path": path,
        "sha256": sha256_text(text),
        "text": text
    })
}

pub fn file_with_sha(path: &str, text: &str, sha256: &str) -> Value {
    json!({
        "path": path,
        "sha256": sha256,
        "text": text
    })
}
