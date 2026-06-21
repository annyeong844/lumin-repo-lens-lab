use serde_json::Value;

use super::summary::{
    assert_cfg_gate_count, assert_macro_call_count, assert_muted_opaque_reason_count,
    assert_opaque_totals,
};
use super::surfaces::assert_opaque_surfaces;

pub fn assert_review_and_muted_surfaces(artifact: &Value, path: &str) {
    assert_macro_call_count(artifact, 1);
    assert_cfg_gate_count(artifact, 2);
    assert_opaque_totals(artifact, 3, 2, 1);
    assert_muted_opaque_reason_count(artifact, "cfg-test", 1);
    assert_opaque_surfaces(
        artifact,
        path,
        [
            ("cfg-gate", "cfg-condition-not-evaluated", "review", None),
            (
                "macro-expansion",
                "macro-expansion-not-evaluated",
                "review",
                None,
            ),
            (
                "cfg-gate",
                "cfg-condition-not-evaluated",
                "muted",
                Some("cfg-test"),
            ),
        ],
    );
}
