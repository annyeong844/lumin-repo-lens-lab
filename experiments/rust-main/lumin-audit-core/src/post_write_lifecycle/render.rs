use super::protocol::{DeltaEntry, PostWriteDeltaArtifact, CANONICAL_ESCAPE_KINDS};
use std::collections::BTreeMap;

const ANY_DELTA_COL_WIDTH: usize = 28;

pub fn render_markdown(delta: &PostWriteDeltaArtifact) -> String {
    let mut lines = vec![
        "## post-write delta (canonical/any-contamination §6 Stage 2)".to_string(),
        String::new(),
    ];
    let suppress_occurrences = matches!(
        delta.capability_parity.status.as_str(),
        "mismatch" | "missing"
    ) || delta.type_escape_delta.status == "not-applicable";
    if !suppress_occurrences {
        render_any_delta(&mut lines, delta);
        render_entries(&mut lines, delta, "planned", "Planned and observed:");
        render_entries(
            &mut lines,
            delta,
            "silent-new",
            "New escape sites (silent-new — REQUIRE acknowledgment):",
        );
        render_entries(
            &mut lines,
            delta,
            "observed-unbaselined",
            "Observed without baseline:",
        );
        render_entries(
            &mut lines,
            delta,
            "planned-not-observed",
            "Planned but not observed:",
        );
        render_entries(&mut lines, delta, "removed", "Removed:");
    }
    render_file_delta(&mut lines, delta);
    lines.push(format!(
        "Type-escape delta: {}{}",
        delta.type_escape_delta.status,
        optional_reason(delta.type_escape_delta.reason.as_deref())
    ));
    lines.push(String::new());
    render_inventory_completeness(&mut lines, delta);
    render_status_lines(&mut lines, delta);
    render_summary(&mut lines, delta);
    format!("{}\n", lines.join("\n"))
}

fn render_any_delta(lines: &mut Vec<String>, delta: &PostWriteDeltaArtifact) {
    let mut counts = CANONICAL_ESCAPE_KINDS
        .iter()
        .map(|kind| (*kind, 0_u64))
        .collect::<BTreeMap<_, _>>();
    for entry in delta
        .entries
        .iter()
        .filter(|entry| entry.label == "silent-new")
    {
        if let Some(kind) = entry.escape_kind.as_deref() {
            *counts.entry(kind).or_insert(0) += 1;
        }
    }
    lines.push("Any delta (silent-new counts):".to_string());
    for kind in CANONICAL_ESCAPE_KINDS {
        let label = kind_label(kind);
        let padding = " ".repeat(ANY_DELTA_COL_WIDTH.saturating_sub(label.len()).max(1));
        lines.push(format!(
            "- {label}:{padding}+{}",
            counts.get(kind).copied().unwrap_or(0)
        ));
    }
    lines.push(String::new());
}

fn render_entries(
    lines: &mut Vec<String>,
    delta: &PostWriteDeltaArtifact,
    label: &str,
    title: &str,
) {
    let entries = delta
        .entries
        .iter()
        .filter(|entry| entry.label == label)
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return;
    }
    lines.push(title.to_string());
    for (index, entry) in entries.into_iter().enumerate() {
        match label {
            "planned-not-observed" => render_planned_missing(lines, index, entry),
            _ => render_observed_entry(lines, index, entry),
        }
    }
    lines.push(String::new());
}

fn render_observed_entry(lines: &mut Vec<String>, index: usize, entry: &DeltaEntry) {
    lines.push(format!(
        "{}. {}:{}  `{}`{}",
        index + 1,
        entry.file.as_deref().unwrap_or("null"),
        entry
            .line
            .map(|line| line.to_string())
            .unwrap_or_else(|| "null".to_string()),
        entry.code_shape.as_deref().unwrap_or("null"),
        diagnostic_suffix(entry)
    ));
    if let Some(identity) = entry.inside_exported_identity.as_deref() {
        lines.push(format!("   insideExportedIdentity: {identity}"));
    }
    match entry.label.as_str() {
        "planned" => lines.push(format!(
            "   planned? yes — {}",
            entry
                .planned_entry
                .as_ref()
                .map(|planned| planned.reason.as_str())
                .unwrap_or_default()
        )),
        "silent-new" => {
            lines.push("   planned? no — reason missing".to_string());
            lines.push(format!(
                "   [grounded, any-inventory.json.typeEscapes[occurrenceKey={}] present in after, absent in before]",
                entry.occurrence_key.as_deref().unwrap_or("null")
            ));
        }
        "observed-unbaselined" => lines.push(
            "   [확인 불가, reason: before-inventory missing; cannot determine new-vs-existing]"
                .to_string(),
        ),
        "removed" => {
            let last = lines.len() - 1;
            lines[last].push_str("  (present in before, absent in after)");
        }
        _ => {}
    }
}

fn render_planned_missing(lines: &mut Vec<String>, index: usize, entry: &DeltaEntry) {
    if let Some(planned) = entry.planned_entry.as_ref() {
        lines.push(format!(
            "{}. planned `{}` at `{}` — not observed after write.{}",
            index + 1,
            planned.escape_kind,
            planned.location_hint,
            diagnostic_suffix(entry)
        ));
    }
}

fn diagnostic_suffix(entry: &DeltaEntry) -> String {
    if entry.diagnostics.is_empty() {
        return String::new();
    }
    let labels = entry
        .diagnostics
        .iter()
        .map(|diagnostic| {
            if diagnostic == "ambiguous-planned-match" {
                "ambiguous planned-match"
            } else {
                diagnostic.as_str()
            }
        })
        .collect::<Vec<_>>();
    format!(" ({})", labels.join(", "))
}

fn render_file_delta(lines: &mut Vec<String>, delta: &PostWriteDeltaArtifact) {
    let file_delta = &delta.file_delta;
    lines.push("File delta:".to_string());
    if file_delta.status != "computed" {
        lines.push(format!(
            "- status: {}{}",
            file_delta.status,
            optional_reason(file_delta.reason.as_deref())
        ));
        if file_delta.status == "baseline-missing" {
            lines.push(
                "- unexpected-new cannot be determined without the pre-write file inventory."
                    .to_string(),
            );
        }
        lines.push(String::new());
        render_file_list(
            lines,
            "Planned but not observed",
            file_delta.planned_missing.as_deref(),
        );
        return;
    }
    let summary = file_delta.summary.as_ref();
    lines.push(format!(
        "- new files: {} (planned {}, unexpected {})",
        summary.map_or(0, |summary| summary.new_files),
        summary.map_or(0, |summary| summary.planned_new),
        summary.map_or(0, |summary| summary.unexpected_new)
    ));
    lines.push(format!(
        "- removed files: {}",
        summary.map_or(0, |summary| summary.removed)
    ));
    lines.push(format!(
        "- planned files observed: {}/{}",
        summary.map_or(0, |summary| summary.planned_observed),
        file_delta.planned_files.len()
    ));
    lines.push(String::new());
    render_file_list(
        lines,
        "Unexpected new files",
        file_delta.unexpected_new.as_deref(),
    );
    render_file_list(
        lines,
        "Planned new files",
        file_delta.planned_new.as_deref(),
    );
    render_file_list(
        lines,
        "Planned but not observed",
        file_delta.planned_missing.as_deref(),
    );
}

fn render_file_list(lines: &mut Vec<String>, title: &str, files: Option<&[String]>) {
    let Some(files) = files.filter(|files| !files.is_empty()) else {
        return;
    };
    lines.push(format!("{title}:"));
    for (index, file) in files.iter().enumerate() {
        lines.push(format!("{}. {file}", index + 1));
    }
    lines.push(String::new());
}

fn render_inventory_completeness(lines: &mut Vec<String>, delta: &PostWriteDeltaArtifact) {
    let completeness = &delta.inventory_completeness;
    let after_errors = completeness
        .files_with_parse_errors
        .iter()
        .filter(|error| error.side == "after")
        .count();
    let before_errors = completeness
        .files_with_parse_errors
        .iter()
        .filter(|error| error.side == "before")
        .count();
    lines.push("Inventory completeness:".to_string());
    lines.push(match completeness.after_complete {
        Some(true) => "- after-inventory complete: yes".to_string(),
        Some(false) => {
            format!("- after-inventory complete: no — {after_errors} file(s) had parse errors")
        }
        None => "- after-inventory complete: n/a".to_string(),
    });
    lines.push(match completeness.before_complete {
        Some(true) => "- before-inventory complete: yes".to_string(),
        Some(false) => {
            format!("- before-inventory complete: no — {before_errors} file(s) had parse errors")
        }
        None if delta.type_escape_delta.status == "not-applicable" => {
            "- before-inventory complete: n/a".to_string()
        }
        None => "- before-inventory complete: n/a (baseline missing)".to_string(),
    });
    if completeness.files_with_parse_errors.is_empty() {
        lines.push("- files with parse errors: empty".to_string());
    } else {
        lines.push("- files with parse errors:".to_string());
        for error in &completeness.files_with_parse_errors {
            lines.push(format!(
                "  - {}:{} ({}) — {}",
                error.side, error.file, error.line, error.message
            ));
        }
    }
    lines.push(String::new());
}

fn render_status_lines(lines: &mut Vec<String>, delta: &PostWriteDeltaArtifact) {
    lines.push(format!(
        "Baseline status: {}{}",
        delta.baseline.status,
        optional_reason(delta.baseline.reason.as_deref())
    ));
    lines.push(format!(
        "Capability parity: {}{}",
        delta.capability_parity.status,
        optional_reason(delta.capability_parity.mismatch_detail.as_deref())
    ));
    lines.push(format!(
        "Scan-range parity: {}{}",
        delta.scan_range_parity.status,
        optional_reason(delta.scan_range_parity.mismatch_detail.as_deref())
    ));
    lines.push(String::new());
}

fn render_summary(lines: &mut Vec<String>, delta: &PostWriteDeltaArtifact) {
    if delta.type_escape_delta.status == "not-applicable" {
        lines.push("No TS type-escape acknowledgements required; this advisory has no TS any-equivalent post-write lane.".to_string());
        return;
    }
    if delta.summary.silent_new > 0 {
        lines.push(format!(
            "silent-new — REQUIRE acknowledgment: {} entries listed above.",
            delta.summary.silent_new
        ));
        return;
    }
    let mut reasons = Vec::new();
    match delta.capability_parity.status.as_str() {
        "missing" => reasons.push("after-inventory missing".to_string()),
        "mismatch" => reasons.push("after-inventory unusable".to_string()),
        _ => {}
    }
    if delta.baseline.status == "missing" {
        reasons.push("before-inventory missing".to_string());
    }
    if delta.scan_range_parity.status == "mismatch" {
        let detail = delta
            .scan_range_parity
            .mismatch_detail
            .as_deref()
            .map(|detail| format!(" ({detail})"))
            .unwrap_or_default();
        reasons.push(format!("scan-range mismatch{detail}"));
    }
    if delta.inventory_completeness.after_complete == Some(false) {
        let count = delta
            .inventory_completeness
            .files_with_parse_errors
            .iter()
            .filter(|error| error.side == "after")
            .count();
        reasons.push(format!(
            "after-inventory incomplete: {count} file(s) with parse errors"
        ));
    }
    if delta.inventory_completeness.before_complete == Some(false) {
        let count = delta
            .inventory_completeness
            .files_with_parse_errors
            .iter()
            .filter(|error| error.side == "before")
            .count();
        reasons.push(format!(
            "before-inventory incomplete: {count} file(s) with parse errors"
        ));
    }
    if reasons.is_empty() {
        lines.push("No silent new any in the scan range.".to_string());
    } else {
        lines.push(format!(
            "No silent-new acknowledgements required; delta confidence is limited by {}.",
            reasons.join(", ")
        ));
    }
}

fn optional_reason(reason: Option<&str>) -> String {
    reason
        .map(|reason| format!(" — {reason}"))
        .unwrap_or_default()
}

fn kind_label(kind: &str) -> &str {
    match kind {
        "explicit-any" => "explicit any",
        "as-any" => "as any",
        "angle-any" => "angle-any",
        "as-unknown-as-T" => "as unknown as T",
        "rest-any-args" => "rest-any-args",
        "index-sig-any" => "index-sig-any",
        "generic-default-any" => "generic-default-any",
        "ts-ignore" => "ts-ignore",
        "ts-expect-error" => "ts-expect-error",
        "no-explicit-any-disable" => "no-explicit-any-disable",
        "jsdoc-any" => "JSDoc any",
        _ => kind,
    }
}
