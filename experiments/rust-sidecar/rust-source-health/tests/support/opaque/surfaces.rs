use serde_json::Value;

use super::super::artifact::opaque_surfaces;

pub fn assert_opaque_detail(
    artifact: &Value,
    path: &str,
    detail: &str,
    visibility: &str,
    mute_reason: Option<&str>,
) {
    assert!(opaque_surfaces(artifact, path).iter().any(|surface| {
        surface["detail"] == detail
            && surface["visibility"] == visibility
            && mute_reason
                .map(|reason| surface["muteReason"] == reason)
                .unwrap_or_else(|| surface["muteReason"].is_null())
    }));
}

pub fn assert_opaque_surfaces<const N: usize>(
    artifact: &Value,
    path: &str,
    expected: [(&str, &str, &str, Option<&str>); N],
) {
    let surfaces = opaque_surfaces(artifact, path);
    assert_eq!(surfaces.len(), expected.len());
    for (surface, (kind, reason, visibility, mute_reason)) in surfaces.iter().zip(expected) {
        assert_eq!(surface["kind"], kind);
        assert_eq!(surface["reason"], reason);
        assert_eq!(surface["visibility"], visibility);
        match mute_reason {
            Some(reason) => assert_eq!(surface["muteReason"], reason),
            None => assert!(surface["muteReason"].is_null()),
        }
    }
}

pub fn assert_opaque_visibility_count(
    artifact: &Value,
    path: &str,
    visibility: &str,
    count: usize,
) {
    assert_eq!(
        opaque_surfaces(artifact, path)
            .iter()
            .filter(|surface| surface["visibility"] == visibility)
            .count(),
        count
    );
}
