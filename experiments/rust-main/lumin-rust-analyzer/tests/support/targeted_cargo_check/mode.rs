use serde_json::Value;

pub(super) fn assert_targeted_mode(artifact: &Value, plan: &Value) {
    assert_eq!(
        artifact["meta"]["input"]["semanticMode"],
        "targeted-cargo-check"
    );
    assert_eq!(
        artifact["phases"]["semantic"]["mode"],
        "targeted-cargo-check"
    );
    assert_eq!(plan["mode"], "targeted-cargo-check");
    assert_eq!(plan["status"], "ran");
    assert_eq!(plan["reason"], "review-syntax-evidence-package-scope");
}
