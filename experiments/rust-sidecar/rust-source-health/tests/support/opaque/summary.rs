use serde_json::Value;

use super::super::artifact::{summary_bucket_count, summary_count};

pub fn assert_opaque_totals(artifact: &Value, opaque: u64, review: u64, muted: u64) {
    assert_eq!(summary_count(artifact, "opaqueSurfaces"), opaque);
    assert_eq!(summary_count(artifact, "reviewOpaqueSurfaces"), review);
    assert_eq!(summary_count(artifact, "mutedOpaqueSurfaces"), muted);
}

pub fn assert_macro_call_count(artifact: &Value, count: u64) {
    assert_eq!(summary_count(artifact, "macroCalls"), count);
}

pub fn assert_cfg_gate_count(artifact: &Value, count: u64) {
    assert_eq!(summary_count(artifact, "cfgGates"), count);
}

pub fn assert_muted_opaque_reason_count(artifact: &Value, reason: &str, count: u64) {
    assert_eq!(
        summary_bucket_count(artifact, "mutedOpaqueSurfacesByReason", reason),
        count
    );
}
