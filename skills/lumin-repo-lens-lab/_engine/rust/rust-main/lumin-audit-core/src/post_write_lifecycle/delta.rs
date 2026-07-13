use super::protocol::{
    AnyInventory, CapabilityFailure, DeltaEntry, DeltaSummary, FileDelta, InventoryCompleteness,
    PlannedTypeEscape, PostWriteDeltaArtifact, PreWriteAdvisory, SidedParseError, StatusBlock,
    TypeEscapeOccurrence, CANONICAL_ESCAPE_KINDS, POST_WRITE_DELTA_SCHEMA_VERSION,
};
use crate::js_ts_extract::normalize_code_shape;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

pub fn type_escape_delta_not_applicable(advisory: &PreWriteAdvisory) -> bool {
    advisory.capabilities.post_write_type_escapes.as_deref() == Some("not-applicable")
        || advisory.capabilities.language.as_deref() == Some("rust")
        || advisory.intent.language.as_deref() == Some("rust")
        || advisory.rust_pre_write.is_some()
}

pub fn compute_delta(
    advisory: &PreWriteAdvisory,
    before: Option<&AnyInventory>,
    after: Option<&AnyInventory>,
    delta_invocation_id: &str,
    file_delta: FileDelta,
) -> PostWriteDeltaArtifact {
    let any_inventory_path = advisory.pre_write.any_inventory_path.clone();
    let not_applicable_reason = "Rust pre-write advisory has no TS any-equivalent post-write lane";

    if type_escape_delta_not_applicable(advisory) {
        return PostWriteDeltaArtifact {
            schema_version: POST_WRITE_DELTA_SCHEMA_VERSION.to_string(),
            pre_write_invocation_id: advisory.invocation_id.clone(),
            delta_invocation_id: delta_invocation_id.to_string(),
            intent_hash: advisory.intent_hash.clone(),
            baseline: StatusBlock {
                status: "not-applicable".to_string(),
                source: any_inventory_path,
                reason: Some(not_applicable_reason.to_string()),
                mismatch_detail: None,
            },
            capability_parity: StatusBlock {
                status: "not-applicable".to_string(),
                source: None,
                reason: None,
                mismatch_detail: Some(not_applicable_reason.to_string()),
            },
            scan_range_parity: StatusBlock::status("not-applicable"),
            inventory_completeness: InventoryCompleteness {
                after_complete: None,
                before_complete: None,
                files_with_parse_errors: Vec::new(),
            },
            type_escape_delta: StatusBlock {
                status: "not-applicable".to_string(),
                source: None,
                reason: Some(not_applicable_reason.to_string()),
                mismatch_detail: None,
            },
            entries: Vec::new(),
            summary: DeltaSummary::default(),
            capability_failures: Vec::new(),
            file_delta,
        };
    }

    let (capability_parity, capability_failure) = capability_parity(after);
    let capability_failures = capability_failure.into_iter().collect::<Vec<_>>();
    if capability_parity.status != "ok" {
        return PostWriteDeltaArtifact {
            schema_version: POST_WRITE_DELTA_SCHEMA_VERSION.to_string(),
            pre_write_invocation_id: advisory.invocation_id.clone(),
            delta_invocation_id: delta_invocation_id.to_string(),
            intent_hash: advisory.intent_hash.clone(),
            baseline: StatusBlock {
                status: "missing".to_string(),
                source: any_inventory_path,
                reason: Some("capability gate failed — see capabilityParity".to_string()),
                mismatch_detail: None,
            },
            capability_parity,
            scan_range_parity: StatusBlock::status("baseline-missing"),
            inventory_completeness: inventory_completeness(before, after, false),
            type_escape_delta: StatusBlock {
                status: "unavailable".to_string(),
                source: None,
                reason: Some("capability gate failed — see capabilityParity".to_string()),
                mismatch_detail: None,
            },
            entries: Vec::new(),
            summary: DeltaSummary::default(),
            capability_failures,
            file_delta,
        };
    }

    let usable_before = before.filter(|inventory| inventory_usable(inventory));
    let (baseline_status, baseline_reason) = match (before, usable_before) {
        (_, Some(_)) => ("available", None),
        (Some(_), None) => (
            "missing",
            Some(
                "beforeInventory present but unusable (meta.supports.typeEscapes !== true or escapeKinds drift)"
                    .to_string(),
            ),
        ),
        (None, None) => (
            "missing",
            Some(match any_inventory_path.as_deref() {
                Some(path) => format!("before-inventory not loaded from {path}"),
                None => "advisory has no preWrite.anyInventoryPath".to_string(),
            }),
        ),
    };
    let baseline = StatusBlock {
        status: baseline_status.to_string(),
        source: any_inventory_path,
        reason: baseline_reason,
        mismatch_detail: None,
    };
    let baseline_available = baseline.status == "available";
    let Some(after) = after else {
        return PostWriteDeltaArtifact {
            schema_version: POST_WRITE_DELTA_SCHEMA_VERSION.to_string(),
            pre_write_invocation_id: advisory.invocation_id.clone(),
            delta_invocation_id: delta_invocation_id.to_string(),
            intent_hash: advisory.intent_hash.clone(),
            baseline,
            capability_parity,
            scan_range_parity: StatusBlock::status("baseline-missing"),
            inventory_completeness: inventory_completeness(usable_before, None, false),
            type_escape_delta: StatusBlock {
                status: "unavailable".to_string(),
                source: None,
                reason: Some("afterInventory disappeared after capability validation".to_string()),
                mismatch_detail: None,
            },
            entries: Vec::new(),
            summary: DeltaSummary::default(),
            capability_failures,
            file_delta,
        };
    };
    let scan_range_parity = usable_before
        .map(|before| scan_range_parity(before, after))
        .unwrap_or_else(|| StatusBlock::status("baseline-missing"));
    let scan_ok = scan_range_parity.status == "ok";
    let completeness = inventory_completeness(usable_before, Some(after), baseline_available);

    let before_escapes = usable_before
        .map(|inventory| inventory.type_escapes.as_slice())
        .unwrap_or_default();
    let after_escapes = after.type_escapes.as_slice();
    let before_counts = count_by_occurrence_key(before_escapes);
    let absent_from_before = absent_from_before(after_escapes, &before_counts);
    let planned = match_planned(
        &advisory.intent.planned_type_escapes,
        after_escapes,
        &before_counts,
        &absent_from_before,
        baseline_available,
        scan_ok,
    );
    let mut entries = planned.entries;
    entries.extend(classify_remainders(
        after_escapes,
        usable_before,
        baseline_available,
        scan_ok,
        &planned.matched_after,
        &planned.carry_diagnostics,
        &before_counts,
        &absent_from_before,
    ));
    let summary = summarize(&entries);

    PostWriteDeltaArtifact {
        schema_version: POST_WRITE_DELTA_SCHEMA_VERSION.to_string(),
        pre_write_invocation_id: advisory.invocation_id.clone(),
        delta_invocation_id: delta_invocation_id.to_string(),
        intent_hash: advisory.intent_hash.clone(),
        baseline,
        capability_parity,
        scan_range_parity,
        inventory_completeness: completeness,
        type_escape_delta: StatusBlock {
            status: "computed".to_string(),
            source: Some("any-inventory.json".to_string()),
            reason: None,
            mismatch_detail: None,
        },
        entries,
        summary,
        capability_failures,
        file_delta,
    }
}

fn inventory_usable(inventory: &AnyInventory) -> bool {
    inventory.meta.supports.type_escapes
        && inventory
            .meta
            .supports
            .escape_kinds
            .iter()
            .map(String::as_str)
            .eq(CANONICAL_ESCAPE_KINDS.iter().copied())
}

fn capability_parity(after: Option<&AnyInventory>) -> (StatusBlock, Option<CapabilityFailure>) {
    let Some(after) = after else {
        return (
            StatusBlock {
                status: "missing".to_string(),
                source: None,
                reason: None,
                mismatch_detail: Some(
                    "afterInventory absent — see capabilityFailures[]".to_string(),
                ),
            },
            Some(CapabilityFailure {
                kind: "after-inventory-missing".to_string(),
                reason: "afterInventory argument was null or undefined".to_string(),
            }),
        );
    };
    if inventory_usable(after) {
        return (StatusBlock::status("ok"), None);
    }
    let mut reasons = Vec::new();
    if !after.meta.supports.type_escapes {
        reasons.push("meta.supports.typeEscapes !== true");
    }
    if !after
        .meta
        .supports
        .escape_kinds
        .iter()
        .map(String::as_str)
        .eq(CANONICAL_ESCAPE_KINDS.iter().copied())
    {
        reasons.push("meta.supports.escapeKinds drifts from canonical §3.9");
    }
    let reason = if reasons.is_empty() {
        "unusable".to_string()
    } else {
        reasons.join("; ")
    };
    (
        StatusBlock {
            status: "mismatch".to_string(),
            source: None,
            reason: None,
            mismatch_detail: Some(reason.clone()),
        },
        Some(CapabilityFailure {
            kind: "after-inventory-unusable".to_string(),
            reason,
        }),
    )
}

fn scan_range_parity(before: &AnyInventory, after: &AnyInventory) -> StatusBlock {
    let mut details = Vec::new();
    if before.meta.scope != after.meta.scope {
        details.push(format!(
            "scope: before={} after={}",
            json_text(before.meta.scope.as_ref()),
            json_text(after.meta.scope.as_ref())
        ));
    }
    if before.meta.include_tests != after.meta.include_tests {
        details.push(format!(
            "includeTests: before={} after={}",
            optional_bool_text(before.meta.include_tests),
            optional_bool_text(after.meta.include_tests)
        ));
    }
    let mut before_excludes = before.meta.exclude.clone();
    let mut after_excludes = after.meta.exclude.clone();
    before_excludes.sort();
    after_excludes.sort();
    if before_excludes != after_excludes {
        details.push(format!(
            "exclude: before={} after={}",
            serde_json::to_string(&before_excludes).unwrap_or_else(|_| "[]".to_string()),
            serde_json::to_string(&after_excludes).unwrap_or_else(|_| "[]".to_string())
        ));
    }
    if details.is_empty() {
        StatusBlock::status("ok")
    } else {
        StatusBlock {
            status: "mismatch".to_string(),
            source: None,
            reason: None,
            mismatch_detail: Some(details.join("; ")),
        }
    }
}

fn json_text(value: Option<&serde_json::Value>) -> String {
    value
        .and_then(|value| serde_json::to_string(value).ok())
        .unwrap_or_else(|| "undefined".to_string())
}

fn optional_bool_text(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "undefined",
    }
}

fn inventory_completeness(
    before: Option<&AnyInventory>,
    after: Option<&AnyInventory>,
    baseline_available: bool,
) -> InventoryCompleteness {
    let mut files_with_parse_errors = Vec::new();
    if let Some(after) = after {
        files_with_parse_errors.extend(after.meta.files_with_parse_errors.iter().map(|error| {
            SidedParseError {
                side: "after".to_string(),
                file: error.file.clone(),
                message: error.message.clone(),
                line: error.line.clone(),
            }
        }));
    }
    if baseline_available {
        if let Some(before) = before {
            files_with_parse_errors.extend(before.meta.files_with_parse_errors.iter().map(
                |error| SidedParseError {
                    side: "before".to_string(),
                    file: error.file.clone(),
                    message: error.message.clone(),
                    line: error.line.clone(),
                },
            ));
        }
    }
    InventoryCompleteness {
        after_complete: after.map(|inventory| inventory.meta.complete),
        before_complete: baseline_available
            .then(|| before.is_some_and(|inventory| inventory.meta.complete)),
        files_with_parse_errors,
    }
}

fn count_by_occurrence_key(
    occurrences: &[TypeEscapeOccurrence],
) -> BTreeMap<Option<String>, usize> {
    let mut counts = BTreeMap::new();
    for occurrence in occurrences {
        *counts.entry(occurrence.occurrence_key.clone()).or_insert(0) += 1;
    }
    counts
}

fn absent_from_before(
    after: &[TypeEscapeOccurrence],
    before_counts: &BTreeMap<Option<String>, usize>,
) -> BTreeSet<usize> {
    let mut indices = (0..after.len()).collect::<Vec<_>>();
    indices.sort_by(|left, right| compare_occurrences(&after[*left], &after[*right]));
    let mut seen = BTreeMap::<Option<String>, usize>::new();
    let mut absent = BTreeSet::new();
    for index in indices {
        let key = after[index].occurrence_key.clone();
        let count = seen.entry(key.clone()).or_insert(0);
        *count += 1;
        if *count > before_counts.get(&key).copied().unwrap_or(0) {
            absent.insert(index);
        }
    }
    absent
}

struct PlannedMatch {
    entries: Vec<DeltaEntry>,
    matched_after: Vec<bool>,
    carry_diagnostics: Vec<Vec<String>>,
}

fn match_planned(
    planned: &[PlannedTypeEscape],
    after: &[TypeEscapeOccurrence],
    before_counts: &BTreeMap<Option<String>, usize>,
    absent_from_before: &BTreeSet<usize>,
    baseline_available: bool,
    scan_ok: bool,
) -> PlannedMatch {
    let mut entries = Vec::new();
    let mut matched_after = vec![false; after.len()];
    let mut carry_diagnostics = vec![Vec::new(); after.len()];

    for planned_entry in planned {
        let mut candidates = after
            .iter()
            .enumerate()
            .filter(|(index, occurrence)| {
                !matched_after[*index]
                    && occurrence.escape_kind.as_deref() == Some(planned_entry.escape_kind.as_str())
                    && location_matches(occurrence, &planned_entry.location_hint)
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            entries.push(DeltaEntry {
                label: "planned-not-observed".to_string(),
                escape_kind: Some(planned_entry.escape_kind.clone()),
                file: None,
                line: None,
                code_shape: None,
                normalized_code_shape: None,
                inside_exported_identity: None,
                occurrence_key: None,
                planned_entry: Some(planned_entry.clone()),
                diagnostics: Vec::new(),
            });
            continue;
        }
        if baseline_available && scan_ok {
            let preferred = candidates
                .iter()
                .copied()
                .filter(|index| {
                    absent_from_before.contains(index)
                        || before_counts
                            .get(&after[*index].occurrence_key)
                            .copied()
                            .unwrap_or(0)
                            == 0
                })
                .collect::<Vec<_>>();
            if !preferred.is_empty() {
                candidates = preferred;
            }
        }
        if let Some(code_shape) = planned_entry.code_shape.as_deref() {
            if candidates.len() >= 2 {
                let target = normalize_code_shape(code_shape);
                let exact = candidates
                    .iter()
                    .copied()
                    .filter(|index| {
                        after[*index].normalized_code_shape.as_deref() == Some(target.as_str())
                    })
                    .collect::<Vec<_>>();
                if !exact.is_empty() {
                    candidates = exact;
                }
            }
        }
        candidates.sort_by(|left, right| compare_occurrences(&after[*left], &after[*right]));
        let choice = candidates[0];
        matched_after[choice] = true;
        entries.push(entry_from_occurrence(
            "planned",
            &after[choice],
            Some(planned_entry.clone()),
            Vec::new(),
        ));
        for index in candidates.into_iter().skip(1) {
            carry_diagnostics[index].push("ambiguous-planned-match".to_string());
        }
    }

    PlannedMatch {
        entries,
        matched_after,
        carry_diagnostics,
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_remainders(
    after: &[TypeEscapeOccurrence],
    before: Option<&AnyInventory>,
    baseline_available: bool,
    scan_ok: bool,
    matched_after: &[bool],
    carry_diagnostics: &[Vec<String>],
    before_counts: &BTreeMap<Option<String>, usize>,
    absent_from_before: &BTreeSet<usize>,
) -> Vec<DeltaEntry> {
    let mut entries = Vec::new();
    let trust_baseline = baseline_available && scan_ok;
    let after_counts = count_by_occurrence_key(after);
    let before_parse_error_files = before
        .into_iter()
        .flat_map(|inventory| inventory.meta.files_with_parse_errors.iter())
        .map(|error| error.file.as_str())
        .collect::<BTreeSet<_>>();

    for (index, occurrence) in after.iter().enumerate() {
        if matched_after[index] {
            continue;
        }
        let mut diagnostics = carry_diagnostics[index].clone();
        if !trust_baseline {
            entries.push(entry_from_occurrence(
                "observed-unbaselined",
                occurrence,
                None,
                diagnostics,
            ));
            continue;
        }
        let key = &occurrence.occurrence_key;
        let duplicate = after_counts.get(key).copied().unwrap_or(0) > 1
            || before_counts.get(key).copied().unwrap_or(0) > 1;
        if before_counts.get(key).copied().unwrap_or(0) > 0 && !absent_from_before.contains(&index)
        {
            entries.push(entry_from_occurrence(
                "pre-existing",
                occurrence,
                None,
                diagnostics,
            ));
        } else if occurrence
            .file
            .as_deref()
            .is_some_and(|file| before_parse_error_files.contains(file))
        {
            if duplicate {
                diagnostics.push("ambiguous-duplicate-occurrence-key".to_string());
            }
            diagnostics.push("before-file-parse-error".to_string());
            entries.push(entry_from_occurrence(
                "observed-unbaselined",
                occurrence,
                None,
                diagnostics,
            ));
        } else {
            if duplicate {
                diagnostics.push("ambiguous-duplicate-occurrence-key".to_string());
            }
            entries.push(entry_from_occurrence(
                "silent-new",
                occurrence,
                None,
                diagnostics,
            ));
        }
    }

    if trust_baseline {
        let mut seen_before = BTreeMap::<Option<String>, usize>::new();
        for occurrence in before
            .into_iter()
            .flat_map(|inventory| inventory.type_escapes.iter())
        {
            let key = occurrence.occurrence_key.clone();
            let count = seen_before.entry(key.clone()).or_insert(0);
            *count += 1;
            if *count > after_counts.get(&key).copied().unwrap_or(0) {
                entries.push(entry_from_occurrence(
                    "removed",
                    occurrence,
                    None,
                    Vec::new(),
                ));
            }
        }
    }
    entries
}

fn location_matches(observed: &TypeEscapeOccurrence, hint: &str) -> bool {
    hint == "unknown"
        || observed.inside_exported_identity.as_deref() == Some(hint)
        || observed.file.as_deref() == Some(hint)
        || (hint.ends_with('/')
            && observed
                .file
                .as_deref()
                .is_some_and(|file| file.starts_with(hint)))
}

fn compare_occurrences(left: &TypeEscapeOccurrence, right: &TypeEscapeOccurrence) -> Ordering {
    left.file
        .as_deref()
        .unwrap_or_default()
        .cmp(right.file.as_deref().unwrap_or_default())
        .then_with(|| {
            left.line
                .unwrap_or_default()
                .cmp(&right.line.unwrap_or_default())
        })
        .then_with(|| {
            left.occurrence_key
                .as_deref()
                .unwrap_or_default()
                .cmp(right.occurrence_key.as_deref().unwrap_or_default())
        })
}

fn entry_from_occurrence(
    label: &str,
    occurrence: &TypeEscapeOccurrence,
    planned_entry: Option<PlannedTypeEscape>,
    diagnostics: Vec<String>,
) -> DeltaEntry {
    DeltaEntry {
        label: label.to_string(),
        escape_kind: occurrence.escape_kind.clone(),
        file: occurrence.file.clone(),
        line: occurrence.line,
        code_shape: occurrence.code_shape.clone(),
        normalized_code_shape: occurrence.normalized_code_shape.clone(),
        inside_exported_identity: occurrence.inside_exported_identity.clone(),
        occurrence_key: occurrence.occurrence_key.clone(),
        planned_entry,
        diagnostics,
    }
}

fn summarize(entries: &[DeltaEntry]) -> DeltaSummary {
    let mut summary = DeltaSummary::default();
    for entry in entries {
        match entry.label.as_str() {
            "planned" => summary.planned += 1,
            "planned-not-observed" => summary.planned_not_observed += 1,
            "silent-new" => summary.silent_new += 1,
            "pre-existing" => summary.pre_existing += 1,
            "removed" => summary.removed += 1,
            "observed-unbaselined" => summary.observed_unbaselined += 1,
            _ => {}
        }
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::post_write_lifecycle::protocol::{
        AdvisoryCapabilities, AdvisoryFileInventory, AdvisoryIntent, AdvisoryPreWrite,
        AdvisoryScanRange, AnyInventoryMeta, AnyInventorySupports,
    };

    fn advisory(planned: Vec<PlannedTypeEscape>) -> PreWriteAdvisory {
        PreWriteAdvisory {
            invocation_id: "PRE".to_string(),
            intent_hash: "hash".to_string(),
            intent: AdvisoryIntent {
                language: None,
                files: Vec::new(),
                planned_type_escapes: planned,
            },
            pre_write: AdvisoryPreWrite {
                any_inventory_path: Some("any-inventory.pre.PRE.json".to_string()),
                file_inventory: AdvisoryFileInventory::default(),
            },
            scan_range: AdvisoryScanRange::default(),
            capabilities: AdvisoryCapabilities::default(),
            rust_pre_write: None,
        }
    }

    fn inventory(escapes: Vec<TypeEscapeOccurrence>) -> AnyInventory {
        AnyInventory {
            meta: AnyInventoryMeta {
                complete: true,
                scope: Some(serde_json::json!("TS/JS production files")),
                include_tests: Some(false),
                exclude: Vec::new(),
                files_with_parse_errors: Vec::new(),
                supports: AnyInventorySupports {
                    type_escapes: true,
                    escape_kinds: CANONICAL_ESCAPE_KINDS
                        .iter()
                        .map(|kind| (*kind).to_string())
                        .collect(),
                },
            },
            type_escapes: escapes,
        }
    }

    fn occurrence(key: &str) -> TypeEscapeOccurrence {
        TypeEscapeOccurrence {
            file: Some("src/a.ts".to_string()),
            line: Some(1),
            escape_kind: Some("as-any".to_string()),
            code_shape: Some("value as any".to_string()),
            normalized_code_shape: Some("value as any".to_string()),
            inside_exported_identity: None,
            occurrence_key: Some(key.to_string()),
        }
    }

    fn empty_file_delta() -> FileDelta {
        FileDelta {
            status: "computed".to_string(),
            reason: None,
            planned_files: Vec::new(),
            before_count: Some(0),
            after_count: Some(0),
            new_files: Some(Vec::new()),
            removed: Some(Vec::new()),
            planned_new: Some(Vec::new()),
            unexpected_new: Some(Vec::new()),
            planned_observed: Some(Vec::new()),
            planned_missing: Some(Vec::new()),
            summary: Some(Default::default()),
        }
    }

    #[test]
    fn classifies_new_and_existing_occurrences_without_weakening_baseline() {
        let before = inventory(vec![occurrence("old")]);
        let after = inventory(vec![occurrence("old"), occurrence("new")]);
        let delta = compute_delta(
            &advisory(Vec::new()),
            Some(&before),
            Some(&after),
            "DELTA",
            empty_file_delta(),
        );
        assert_eq!(delta.summary.pre_existing, 1);
        assert_eq!(delta.summary.silent_new, 1);
    }

    #[test]
    fn planned_match_is_one_to_one_and_ambiguous_remainder_keeps_baseline_label() {
        let planned = PlannedTypeEscape {
            escape_kind: "as-any".to_string(),
            location_hint: "unknown".to_string(),
            reason: "migration".to_string(),
            code_shape: None,
            alternative_considered: None,
        };
        let before = inventory(vec![occurrence("old-a"), occurrence("old-b")]);
        let after = inventory(vec![occurrence("old-a"), occurrence("old-b")]);
        let delta = compute_delta(
            &advisory(vec![planned]),
            Some(&before),
            Some(&after),
            "DELTA",
            empty_file_delta(),
        );
        assert_eq!(delta.summary.planned, 1);
        assert_eq!(delta.summary.pre_existing, 1);
        assert_eq!(delta.summary.silent_new, 0);
        assert!(delta.entries[1]
            .diagnostics
            .contains(&"ambiguous-planned-match".to_string()));
    }
}
