use serde_json::{json, Value};
use std::collections::BTreeSet;

use super::value::{number_u64, optional_array};
use super::{zone, BlindZoneSeverity, BlindZoneSummary};

pub(crate) fn detect_parser_zone(symbols: Option<&Value>) -> Option<BlindZoneSummary> {
    let warnings = optional_array(
        symbols.and_then(|symbols| symbols.get("meta").and_then(|meta| meta.get("warnings"))),
    )?;
    let parse_warning = warnings.iter().find(|warning| {
        warning.get("kind").and_then(Value::as_str) == Some("parse-errors")
            || warning.get("type").and_then(Value::as_str) == Some("parse-errors")
            || warning
                .get("message")
                .and_then(Value::as_str)
                .is_some_and(|message| message.to_ascii_lowercase().contains("parse"))
    })?;
    Some(zone(
        "parser",
        BlindZoneSeverity::PrecisionGap,
        "Graph is partial — some files failed to parse; their defs and uses are missing from the analysis.",
        Some(json!({
            "count": parse_warning.get("count").cloned().unwrap_or(Value::Null),
            "message": parse_warning.get("message").cloned().unwrap_or(Value::Null),
        })),
    ))
}

pub(crate) fn detect_cjs_export_surface_zone(symbols: Option<&Value>) -> Option<BlindZoneSummary> {
    let by_file = symbols
        .and_then(|symbols| symbols.get("cjsExportSurfaceByFile"))
        .and_then(Value::as_object)?;
    let mut opaque_forms = Vec::new();
    for (file, surface) in by_file {
        let opaque = optional_array(surface.get("opaque"))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if opaque.is_empty() {
            continue;
        }
        let mut kinds = BTreeSet::new();
        for entry in opaque {
            if let Some(kind) = entry.get("kind").and_then(Value::as_str) {
                kinds.insert(kind.to_string());
            }
        }
        opaque_forms.push(json!({ "file": file, "kinds": kinds.into_iter().collect::<Vec<_>>() }));
    }
    if opaque_forms.is_empty() {
        return None;
    }
    opaque_forms.sort_by(|left, right| {
        left.get("file")
            .and_then(Value::as_str)
            .unwrap_or("")
            .cmp(right.get("file").and_then(Value::as_str).unwrap_or(""))
    });
    Some(zone(
        "commonjs-export-surface",
        BlindZoneSeverity::PrecisionGap,
        "Some CommonJS files use opaque export forms; named CJS export claims are limited to exact surface facts.",
        Some(json!({
            "files": opaque_forms.len(),
            "opaqueForms": opaque_forms.into_iter().take(10).collect::<Vec<_>>(),
        })),
    ))
}

pub(crate) fn detect_cjs_require_opacity_zone(symbols: Option<&Value>) -> Option<BlindZoneSummary> {
    let calls = optional_array(symbols.and_then(|symbols| symbols.get("cjsRequireOpacity")))?;
    if calls.is_empty() {
        return None;
    }
    let files = calls
        .iter()
        .filter_map(|entry| entry.get("consumerFile").and_then(Value::as_str))
        .filter(|file| !file.is_empty())
        .collect::<BTreeSet<_>>();
    let mut examples = calls.to_vec();
    examples.sort_by_key(cjs_call_sort_key);
    Some(zone(
        "commonjs-dynamic-require",
        BlindZoneSeverity::PrecisionGap,
        "CommonJS dynamic require calls can hide internal consumers; dead-export absence claims near these files are degraded.",
        Some(json!({
            "files": files.len(),
            "calls": calls.len(),
            "examples": examples
                .iter()
                .take(5)
                .map(|entry| json!({
                    "consumerFile": entry.get("consumerFile").cloned().unwrap_or(Value::Null),
                    "line": entry.get("line").cloned().unwrap_or(Value::Null),
                    "kind": entry.get("kind").cloned().unwrap_or(Value::Null),
                }))
                .collect::<Vec<_>>(),
        })),
    ))
}

pub(crate) fn detect_html_entry_surface_zone(
    entry_surface: Option<&Value>,
) -> Option<BlindZoneSummary> {
    let unresolved = optional_array(
        entry_surface.and_then(|entry_surface| entry_surface.get("unresolvedHtmlEntrypoints")),
    )?;
    if unresolved.is_empty() {
        return None;
    }
    let mut examples = unresolved.to_vec();
    examples.sort_by_key(html_entry_sort_key);
    Some(zone(
        "html-entry-surface",
        BlindZoneSeverity::ConfidenceGap,
        "Some HTML module script entrypoints could not be mapped to repository files; module reachability and HTML-entry policy claims are confidence-limited.",
        Some(json!({
            "unresolvedHtmlEntrypoints": unresolved.len(),
            "examples": examples
                .iter()
                .take(5)
                .map(|entry| json!({
                    "htmlFile": entry.get("htmlFile").cloned().unwrap_or(Value::Null),
                    "src": entry.get("src").cloned().unwrap_or(Value::Null),
                    "candidateFile": entry.get("resolvedFile").cloned().unwrap_or(Value::Null),
                    "reason": entry
                        .get("reason")
                        .cloned()
                        .unwrap_or_else(|| json!("html-module-script-target-missing")),
                }))
                .collect::<Vec<_>>(),
        })),
    ))
}

fn cjs_call_sort_key(value: &Value) -> String {
    format!(
        "{}|{:0>6}|{}",
        value
            .get("consumerFile")
            .and_then(Value::as_str)
            .unwrap_or(""),
        value
            .get("line")
            .and_then(number_u64)
            .map(|line| line.to_string())
            .unwrap_or_default(),
        value.get("kind").and_then(Value::as_str).unwrap_or("")
    )
}

fn html_entry_sort_key(value: &Value) -> String {
    format!(
        "{}|{}|{}",
        value.get("htmlFile").and_then(Value::as_str).unwrap_or(""),
        value.get("src").and_then(Value::as_str).unwrap_or(""),
        value
            .get("resolvedFile")
            .and_then(Value::as_str)
            .unwrap_or("")
    )
}
