use anyhow::{Context, Result};
use serde_json::Value;

pub(super) fn lookup<'a>(artifact: &'a Value, name: &str) -> Result<&'a Value> {
    artifact["lookups"]
        .as_array()
        .context("lookups")?
        .iter()
        .find(|lookup| lookup["intentName"] == name)
        .with_context(|| format!("lookup {name}"))
}

pub(super) fn card<'a>(artifact: &'a Value, identity: &str) -> Result<&'a Value> {
    artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .find(|card| card["candidate"]["identity"] == identity)
        .with_context(|| format!("cue card {identity}"))
}

pub(super) fn local_muted<'a>(
    artifact: &'a Value,
    identity: &str,
    reason: &str,
) -> Result<&'a Value> {
    artifact["suppressedCues"]
        .as_array()
        .context("suppressed cues")?
        .iter()
        .find(|cue| {
            cue["candidate"]["identity"] == identity
                && cue["evidenceLane"] == "local-operation-sibling"
                && cue["reason"] == reason
        })
        .with_context(|| format!("local operation muted cue {identity} {reason}"))
}
