use serde_json::Value;

use super::super::artifact::{summary_bucket_count, summary_count};

pub fn assert_signal_totals(artifact: &Value, signals: u64, review: u64, muted: u64) {
    assert_eq!(summary_count(artifact, "signals"), signals);
    assert_eq!(summary_count(artifact, "reviewSignals"), review);
    assert_eq!(summary_count(artifact, "mutedSignals"), muted);
}

pub fn assert_signal_kind_count(artifact: &Value, kind: &str, count: u64) {
    assert_eq!(summary_bucket_count(artifact, "signalsByKind", kind), count);
}

pub fn assert_review_signal_kind_count(artifact: &Value, kind: &str, count: u64) {
    assert_eq!(
        summary_bucket_count(artifact, "reviewSignalsByKind", kind),
        count
    );
}

pub fn assert_muted_signal_reason_count(artifact: &Value, reason: &str, count: u64) {
    assert_eq!(
        summary_bucket_count(artifact, "mutedSignalsByReason", reason),
        count
    );
}
