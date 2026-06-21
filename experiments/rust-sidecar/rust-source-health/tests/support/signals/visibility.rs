use serde_json::Value;

use super::super::artifact::file_signals;

pub fn assert_signal_visibility_count(
    artifact: &Value,
    path: &str,
    visibility: &str,
    mute_reason: Option<&str>,
    count: usize,
) {
    assert_eq!(
        file_signals(artifact, path)
            .iter()
            .filter(|signal| signal["visibility"] == visibility
                && mute_reason
                    .map(|reason| signal["muteReason"] == reason)
                    .unwrap_or_else(|| signal["muteReason"].is_null()))
            .count(),
        count
    );
}

pub fn assert_first_signal_visibility(
    artifact: &Value,
    path: &str,
    visibility: &str,
    mute_reason: Option<&str>,
) {
    let signal = &file_signals(artifact, path)[0];
    assert_eq!(signal["visibility"], visibility);
    match mute_reason {
        Some(reason) => assert_eq!(signal["muteReason"], reason),
        None => assert!(signal["muteReason"].is_null()),
    }
}
