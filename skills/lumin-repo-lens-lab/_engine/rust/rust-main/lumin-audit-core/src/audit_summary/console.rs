use serde_json::Value;

use super::support::{format_unresolved_reason_counts, get};

fn shorten_console_line(line: &str, max: usize) -> String {
    let normalized = line.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized = normalized.trim();
    if normalized.chars().count() > max {
        format!(
            "{}…",
            normalized
                .chars()
                .take(max.saturating_sub(1))
                .collect::<String>()
        )
    } else {
        normalized.to_string()
    }
}

fn collect_summary_section_lines(markdown: &str, heading: &str, limit: usize) -> Vec<String> {
    let lines = markdown.lines().collect::<Vec<_>>();
    let Some(start) = lines.iter().position(|line| line.trim() == heading) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in lines.iter().skip(start + 1) {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }
        if is_list_item(trimmed) {
            out.push(shorten_console_line(trimmed, 150));
            if out.len() >= limit {
                break;
            }
        }
    }
    out
}

fn is_list_item(line: &str) -> bool {
    if line.starts_with("- ") {
        return true;
    }
    let mut chars = line.chars().peekable();
    let mut saw_digit = false;
    while matches!(chars.peek(), Some(ch) if ch.is_ascii_digit()) {
        saw_digit = true;
        chars.next();
    }
    saw_digit && chars.next() == Some('.') && chars.next() == Some(' ')
}

pub fn format_blind_zones_console_summary(zones: &[Value]) -> Option<String> {
    if zones.is_empty() {
        return None;
    }
    let base_scope_count = zones
        .iter()
        .filter(|zone| get(zone, "area").and_then(Value::as_str) == Some("base-audit"))
        .count();
    let analysis_zone_count = zones.len().saturating_sub(base_scope_count);
    let mut parts = Vec::new();
    for (severity, label) in [
        ("scan-gap", "scan-gap"),
        ("precision-gap", "precision-gap"),
        ("confidence-gap", "confidence-gap"),
    ] {
        let count = zones
            .iter()
            .filter(|zone| {
                get(zone, "area").and_then(Value::as_str) != Some("base-audit")
                    && get(zone, "severity").and_then(Value::as_str) == Some(severity)
            })
            .count();
        if count > 0 {
            parts.push(format!("{count} {label}"));
        }
    }
    let analysis_summary = if analysis_zone_count == 0 {
        "blindZones: none in current lifecycle evidence".to_string()
    } else if parts.is_empty() {
        format!("blindZones: {analysis_zone_count} unclassified")
    } else {
        format!("blindZones: {}", parts.join(", "))
    };
    let resolver_summary = zones
        .iter()
        .find(|zone| get(zone, "area").and_then(Value::as_str) == Some("resolver"))
        .and_then(|zone| {
            format_unresolved_reason_counts(zone.pointer("/details/topUnresolvedReasons"), 3)
        })
        .map(|reasons| format!("; resolver reasons: {reasons}"))
        .unwrap_or_default();
    let base_summary = if base_scope_count > 0 {
        "; baseEvidence: not refreshed (lifecycle-only)"
    } else {
        ""
    };
    Some(format!(
        "{analysis_summary}{resolver_summary}{base_summary}"
    ))
}

pub fn render_summary_console_preview(markdown: &str) -> Option<String> {
    let sections = [
        (
            "Command Result",
            collect_summary_section_lines(markdown, "## Command Result", 3),
        ),
        (
            "Read First",
            collect_summary_section_lines(markdown, "## Read First", 2),
        ),
        (
            "Measured Cues",
            collect_summary_section_lines(markdown, "## Measured Cues (Unranked)", 3),
        ),
        (
            "Living Audit Tracking",
            collect_summary_section_lines(markdown, "## Living Audit Tracking", 2),
        ),
        (
            "Guardrails",
            collect_summary_section_lines(markdown, "## Guardrails", 2),
        ),
    ];
    let sections = sections
        .into_iter()
        .filter(|(_, lines)| !lines.is_empty())
        .collect::<Vec<_>>();
    if sections.is_empty() {
        return None;
    }
    let mut out = vec!["[audit-repo] artifact brief preview:".to_string()];
    for (label, lines) in sections {
        out.push(format!("[audit-repo]   {label}:"));
        out.extend(
            lines
                .into_iter()
                .map(|line| format!("[audit-repo]     {line}")),
        );
    }
    Some(out.join("\n"))
}
