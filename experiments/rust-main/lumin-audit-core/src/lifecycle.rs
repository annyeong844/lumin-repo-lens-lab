use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleSummary {
    pub summary_owner: &'static str,
    pub execution_owner: &'static str,
    pub source_status: LifecycleSourceStatus,
    pub requested_count: u64,
    pub ran_count: u64,
    pub not_run_count: u64,
    pub pre_write: LifecycleBlockSummary,
    pub post_write: LifecycleBlockSummary,
    pub canon_draft: LifecycleBlockSummary,
    pub check_canon: LifecycleBlockSummary,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestLifecycleUpdateInput {
    #[serde(default)]
    pub pre_write: Option<Value>,
    #[serde(default)]
    pub post_write: Option<Value>,
    #[serde(default)]
    pub canon_draft: Option<Value>,
    #[serde(default)]
    pub check_canon: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestLifecycleUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_write: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_write: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canon_draft: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_canon: Option<Value>,
    pub lifecycle: LifecycleSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleSourceStatus {
    Available,
    InvalidShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleBlockStatus {
    NotRequested,
    Complete,
    NotRun,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleBlockSummary {
    pub requested: bool,
    pub ran: bool,
    pub status: LifecycleBlockStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_source_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ran_source_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_source_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_invocations: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drift_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources_checked: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources_skipped: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources_failed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silent_new: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_acknowledgement_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_range_parity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_escape_delta_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_complete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_delta_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unexpected_new_file_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_missing_file_count: Option<u64>,
}

pub fn summarize_lifecycle(input: &Value) -> LifecycleSummary {
    let Some(root) = input.as_object() else {
        return LifecycleSummary {
            summary_owner: "lumin-audit-core",
            execution_owner: "audit-repo.mjs",
            source_status: LifecycleSourceStatus::InvalidShape,
            requested_count: 0,
            ran_count: 0,
            not_run_count: 0,
            pre_write: unavailable_block(),
            post_write: unavailable_block(),
            canon_draft: unavailable_block(),
            check_canon: unavailable_block(),
        };
    };

    let pre_write = summarize_block(root.get("preWrite"), BlockKind::PreWrite);
    let post_write = summarize_block(root.get("postWrite"), BlockKind::PostWrite);
    let canon_draft = summarize_block(root.get("canonDraft"), BlockKind::CanonDraft);
    let check_canon = summarize_block(root.get("checkCanon"), BlockKind::CheckCanon);

    let blocks = [&pre_write, &post_write, &canon_draft, &check_canon];
    LifecycleSummary {
        summary_owner: "lumin-audit-core",
        execution_owner: "audit-repo.mjs",
        source_status: LifecycleSourceStatus::Available,
        requested_count: blocks.iter().filter(|block| block.requested).count() as u64,
        ran_count: blocks.iter().filter(|block| block.ran).count() as u64,
        not_run_count: blocks
            .iter()
            .filter(|block| block.requested && !block.ran)
            .count() as u64,
        pre_write,
        post_write,
        canon_draft,
        check_canon,
    }
}

pub fn build_manifest_lifecycle_update(
    input: ManifestLifecycleUpdateInput,
) -> ManifestLifecycleUpdate {
    let pre_write = non_null_block(input.pre_write);
    let post_write = non_null_block(input.post_write);
    let canon_draft = non_null_block(input.canon_draft);
    let check_canon = non_null_block(input.check_canon);
    let summary_input = json!({
        "preWrite": pre_write.as_ref().unwrap_or(&Value::Null),
        "postWrite": post_write.as_ref().unwrap_or(&Value::Null),
        "canonDraft": canon_draft.as_ref().unwrap_or(&Value::Null),
        "checkCanon": check_canon.as_ref().unwrap_or(&Value::Null),
    });

    ManifestLifecycleUpdate {
        pre_write,
        post_write,
        canon_draft,
        check_canon,
        lifecycle: summarize_lifecycle(&summary_input),
    }
}

fn non_null_block(value: Option<Value>) -> Option<Value> {
    value.filter(|value| !value.is_null())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    PreWrite,
    PostWrite,
    CanonDraft,
    CheckCanon,
}

fn summarize_block(value: Option<&Value>, kind: BlockKind) -> LifecycleBlockSummary {
    let Some(value) = value else {
        return not_requested_block();
    };
    if value.is_null() {
        return not_requested_block();
    }
    let Some(object) = value.as_object() else {
        return unavailable_block();
    };

    let requested = bool_field(value, "requested").unwrap_or(false);
    let ran = bool_field(value, "ran").unwrap_or(false);
    let status = if !requested {
        LifecycleBlockStatus::NotRequested
    } else if ran {
        LifecycleBlockStatus::Complete
    } else {
        LifecycleBlockStatus::NotRun
    };

    let mut summary = LifecycleBlockSummary {
        requested,
        ran,
        status,
        engine: string_field(value, "engine"),
        language: string_field(value, "language"),
        producer: string_field(value, "producer"),
        reason: string_field(value, "reason"),
        requested_source_count: array_len(value, "requestedSources"),
        ran_source_count: count_per_source(value, |entry| bool_field(entry, "ran") == Some(true)),
        failed_source_count: count_per_source(value, |entry| {
            bool_field(entry, "ran") == Some(false)
        }),
        draft_count: array_len(value, "draftPaths"),
        strict: bool_field(value, "strict"),
        execution_mode: string_field(value, "executionMode"),
        child_invocations: number_field(value, "childInvocations"),
        drift_count: nested_number(value, &["summary", "driftCount"]),
        sources_checked: nested_number(value, &["summary", "sourcesChecked"]),
        sources_skipped: nested_number(value, &["summary", "sourcesSkipped"]),
        sources_failed: nested_number(value, &["summary", "sourcesFailed"]),
        silent_new: number_field(value, "silentNew"),
        required_acknowledgement_count: number_field(value, "requiredAcknowledgementCount"),
        baseline_status: string_field(value, "baselineStatus"),
        scan_range_parity: string_field(value, "scanRangeParity"),
        type_escape_delta_status: string_field(value, "typeEscapeDeltaStatus"),
        after_complete: bool_field(value, "afterComplete"),
        file_delta_status: string_field(value, "fileDeltaStatus"),
        unexpected_new_file_count: number_field(value, "unexpectedNewFileCount"),
        planned_missing_file_count: number_field(value, "plannedMissingFileCount"),
    };

    if kind != BlockKind::CanonDraft {
        summary.draft_count = None;
    }
    if kind != BlockKind::CheckCanon {
        summary.strict = None;
        summary.execution_mode = None;
        summary.child_invocations = None;
        summary.drift_count = None;
        summary.sources_checked = None;
        summary.sources_skipped = None;
        summary.sources_failed = None;
    }
    if kind != BlockKind::PostWrite {
        summary.silent_new = None;
        summary.required_acknowledgement_count = None;
        summary.baseline_status = None;
        summary.scan_range_parity = None;
        summary.type_escape_delta_status = None;
        summary.after_complete = None;
        summary.file_delta_status = None;
        summary.unexpected_new_file_count = None;
        summary.planned_missing_file_count = None;
    }
    if matches!(kind, BlockKind::PreWrite | BlockKind::PostWrite) {
        summary.requested_source_count = None;
        summary.ran_source_count = None;
        summary.failed_source_count = None;
    }
    if object.is_empty() {
        return unavailable_block();
    }
    summary
}

fn not_requested_block() -> LifecycleBlockSummary {
    LifecycleBlockSummary {
        requested: false,
        ran: false,
        status: LifecycleBlockStatus::NotRequested,
        engine: None,
        language: None,
        producer: None,
        reason: None,
        requested_source_count: None,
        ran_source_count: None,
        failed_source_count: None,
        draft_count: None,
        strict: None,
        execution_mode: None,
        child_invocations: None,
        drift_count: None,
        sources_checked: None,
        sources_skipped: None,
        sources_failed: None,
        silent_new: None,
        required_acknowledgement_count: None,
        baseline_status: None,
        scan_range_parity: None,
        type_escape_delta_status: None,
        after_complete: None,
        file_delta_status: None,
        unexpected_new_file_count: None,
        planned_missing_file_count: None,
    }
}

fn unavailable_block() -> LifecycleBlockSummary {
    LifecycleBlockSummary {
        status: LifecycleBlockStatus::Unavailable,
        ..not_requested_block()
    }
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn bool_field(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn number_field(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn nested_number(value: &Value, path: &[&str]) -> Option<u64> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    cursor.as_u64()
}

fn array_len(value: &Value, key: &str) -> Option<u64> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| items.len() as u64)
}

fn count_per_source(value: &Value, predicate: impl Fn(&Value) -> bool) -> Option<u64> {
    let per_source = value.get("perSource")?.as_object()?;
    Some(per_source.values().filter(|entry| predicate(entry)).count() as u64)
}
