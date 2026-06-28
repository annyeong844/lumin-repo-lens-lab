use lumin_rust_common::sha256_text;
use serde_json::Value;

pub fn file_health<'a>(artifact: &'a Value, path: &str) -> &'a Value {
    &artifact["files"][path]
}

pub fn assert_file_parse_ok(artifact: &Value, path: &str, source: &str) {
    let file = file_health(artifact, path);
    assert_eq!(file["sha256"], sha256_text(source));
    assert_eq!(file["parse"]["ok"], true);
}

pub fn assert_file_fact_count(artifact: &Value, path: &str, key: &str, count: u64) {
    assert_eq!(file_health(artifact, path)["facts"][key], count);
}

pub fn file_signals<'a>(artifact: &'a Value, path: &str) -> &'a Vec<Value> {
    artifact["files"][path]["signals"]
        .as_array()
        .unwrap_or_else(|| panic!("{path} signals"))
}

pub fn opaque_surfaces<'a>(artifact: &'a Value, path: &str) -> &'a Vec<Value> {
    artifact["files"][path]["ast"]["opaqueSurfaces"]
        .as_array()
        .unwrap_or_else(|| panic!("{path} opaque surfaces"))
}
