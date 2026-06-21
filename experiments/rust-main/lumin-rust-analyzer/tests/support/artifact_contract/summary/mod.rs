use serde_json::Value;

mod semantic;
mod syntax;
mod tier;

pub(super) fn assert_summary_projection(artifact: &Value) {
    assert_eq!(artifact["summary"]["files"], 3);
    syntax::assert_syntax_summary(artifact);
    semantic::assert_semantic_summary(artifact);
    tier::assert_tier_summary(artifact);
    assert_bridge_summary(artifact);
}

fn assert_bridge_summary(artifact: &Value) {
    assert_eq!(
        artifact["summary"]["semanticClean"]["status"],
        "unavailable"
    );
    assert_eq!(artifact["summary"]["oracleBridgeStatus"], "oracle-partial");
}
