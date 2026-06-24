use serde_json::Value;

pub fn summary_count(artifact: &Value, key: &str) -> u64 {
    artifact["summary"][key]
        .as_u64()
        .unwrap_or_else(|| panic!("summary.{key} count"))
}

pub fn summary_bucket_count(artifact: &Value, bucket: &str, key: &str) -> u64 {
    artifact["summary"][bucket][key]
        .as_u64()
        .unwrap_or_else(|| panic!("summary.{bucket}.{key} count"))
}

pub fn assert_syntax_artifact_metadata(artifact: &Value) {
    assert_eq!(artifact["schemaVersion"], 1);
    assert_eq!(artifact["meta"]["producer"], "rust-source-health");
    assert_eq!(artifact["meta"]["mode"], "syntax-only");
    assert_eq!(artifact["meta"]["parser"]["version"], "0.0.337");
    assert_eq!(
        artifact["meta"]["policy"]["version"],
        "m6-rust-source-health-syntax-v7"
    );
    assert_eq!(
        artifact["meta"]["policy"]["signalPolicy"]["id"],
        "rust-source-health-signal-policy"
    );
    assert_eq!(
        artifact["meta"]["policy"]["signalPolicy"]["version"],
        "rust-source-health-signal-policy.v2"
    );
}
