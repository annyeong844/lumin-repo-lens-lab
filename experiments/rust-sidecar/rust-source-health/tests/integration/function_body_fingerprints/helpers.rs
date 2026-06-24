use anyhow::{Context, Result};
use serde_json::Value;

pub(super) fn fact_named<'a>(facts: &'a [Value], name: &str) -> Result<&'a Value> {
    facts
        .iter()
        .find(|fact| fact["name"] == name)
        .with_context(|| format!("missing function body fingerprint for {name}"))
}

pub(super) fn identity_list_contains(group: &Value, identity: &str) -> bool {
    group["identities"]
        .as_array()
        .is_some_and(|identities| identities.iter().any(|entry| entry == identity))
}

pub(super) fn group_with_identity<'a>(groups: &'a [Value], identity: &str) -> Option<&'a Value> {
    groups
        .iter()
        .find(|group| identity_list_contains(group, identity))
}
