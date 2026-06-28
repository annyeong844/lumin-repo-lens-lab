use serde::Deserialize;

use super::input;

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationCorpusEntry {
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    name: Option<String>,
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    commit: Option<String>,
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    snapshot_id: Option<String>,
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    content_hash: Option<String>,
    #[serde(default, deserialize_with = "input::deserialize_optional_bool")]
    worktree_dirty: Option<bool>,
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    loc_bucket: Option<String>,
}

impl CalibrationCorpusEntry {
    pub(crate) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub(crate) fn display_name(&self) -> &str {
        self.name
            .as_deref()
            .filter(|name| !name.is_empty())
            .unwrap_or("(unnamed)")
    }

    pub(crate) fn has_immutable_identity(&self) -> bool {
        has_text(&self.commit) || has_text(&self.snapshot_id)
    }

    pub(crate) fn dirty_state_known(&self) -> bool {
        self.worktree_dirty.is_some()
    }

    pub(crate) fn dirty_state_captured(&self) -> bool {
        self.worktree_dirty != Some(true)
            || has_text(&self.snapshot_id)
            || has_text(&self.content_hash)
    }

    pub(crate) fn is_non_trivial(&self) -> bool {
        matches!(self.loc_bucket.as_deref(), Some("25k" | "50k" | "100k"))
    }
}

fn has_text(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(|value| !value.is_empty())
}
