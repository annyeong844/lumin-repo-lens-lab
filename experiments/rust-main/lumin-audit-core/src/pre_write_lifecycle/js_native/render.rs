use serde_json::Value;

pub(super) fn handoff_markdown(advisory: &Value) -> String {
    let mut lines = vec![
        "## pre-write advisory (canonical/pre-write-gate §5)".to_string(),
        String::new(),
    ];
    if let Some(path) = advisory
        .pointer("/artifactPaths/invocationSpecific")
        .and_then(Value::as_str)
    {
        lines.push(format!(
            "Post-write handoff: `--pre-write-advisory {path}`."
        ));
        lines.push(format!("Full current-run advisory: `{path}`."));
        lines.push(String::new());
    }
    lines.push(format!(
        "Summary: cueCards=`{}`, suppressedCues=`{}`, lookups=`{}`, unavailableEvidence=`{}`, drift=`{}`, plannedTypeEscapes=`{}`.",
        array_len(advisory, "/cueCards"),
        array_len(advisory, "/suppressedCues"),
        array_len(advisory, "/lookups"),
        array_len(advisory, "/unavailableEvidence"),
        array_len(advisory, "/drift"),
        array_len(advisory, "/intent/plannedTypeEscapes"),
    ));
    lines.push(
        "Read the invocation-specific JSON selectively before editing; stdout does not duplicate its per-candidate rows."
            .to_string(),
    );
    lines.join("\n")
}

fn array_len(value: &Value, pointer: &str) -> usize {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
}

pub(super) fn markdown(advisory: &Value) -> String {
    let mut lines = vec![
        "## pre-write advisory (canonical/pre-write-gate §5)".to_string(),
        String::new(),
    ];
    if let Some(path) = advisory
        .pointer("/artifactPaths/invocationSpecific")
        .and_then(Value::as_str)
    {
        lines.push(format!(
            "Post-write handoff: `--pre-write-advisory {path}`."
        ));
        lines.push("Use this invocation-specific path for the matching post-write check; `pre-write-advisory.latest.json` is only a convenience pointer and can be overwritten by another pre-write run.".to_string());
        lines.push(String::new());
    }
    render_evidence_availability(advisory, &mut lines);
    render_cues(advisory, &mut lines);
    render_legacy_lookups(advisory, &mut lines);
    render_drift(advisory, &mut lines);
    render_planned_escapes(advisory, &mut lines);
    lines.join("\n")
}

fn render_evidence_availability(advisory: &Value, lines: &mut Vec<String>) {
    let Some(availability) = advisory
        .get("evidenceAvailability")
        .filter(|value| value.is_object())
    else {
        return;
    };
    lines.push("### Evidence availability".to_string());
    lines.push(String::new());
    lines.push(format!(
        "- Status: `{}`; current-run Rust compact evidence: `{}`.",
        availability
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        availability
            .get("freshAudit")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    ));
    for artifact in availability
        .get("artifacts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        lines.push(format!(
            "- `{}`: `{}`{}",
            artifact
                .get("artifact")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            artifact
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            artifact
                .get("reason")
                .and_then(Value::as_str)
                .map(|reason| format!(" — {reason}"))
                .unwrap_or_default()
        ));
    }
    lines.push(String::new());
}

fn render_cues(advisory: &Value, lines: &mut Vec<String>) {
    let cards = advisory
        .get("cueCards")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    for (tier, title) in [
        ("SAFE_CUE", "### Grounded reuse cues"),
        ("AGENT_REVIEW_CUE", "### Agent review cues"),
    ] {
        let selected = cards
            .iter()
            .filter(|card| card.get("renderTier").and_then(Value::as_str) == Some(tier))
            .collect::<Vec<_>>();
        if selected.is_empty() {
            continue;
        }
        lines.push(title.to_string());
        lines.push(String::new());
        for card in selected {
            let identity = card
                .pointer("/candidate/identity")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            lines.push(format!("- `{identity}`"));
            for cue in card
                .get("cues")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                lines.push(format!(
                    "  - {} (`{}`; `{}`)",
                    cue.get("claim")
                        .and_then(Value::as_str)
                        .unwrap_or("review evidence"),
                    cue.get("evidenceLane")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown"),
                    cue.get("confidence")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                ));
            }
        }
        lines.push(String::new());
    }

    let unavailable = advisory
        .get("unavailableEvidence")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    if !unavailable.is_empty() {
        lines.push("### Unavailable evidence".to_string());
        lines.push(String::new());
        for item in unavailable {
            lines.push(format!(
                "- `{}`: `{}` ({})",
                item.get("evidenceLane")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                item.get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("UNAVAILABLE"),
                item.get("artifact")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown artifact")
            ));
        }
        lines.push(String::new());
    }
}

fn render_legacy_lookups(advisory: &Value, lines: &mut Vec<String>) {
    let lookups = advisory
        .get("lookups")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let mut rows = Vec::new();
    for lookup in lookups {
        let kind = lookup
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let result = lookup
            .get("result")
            .and_then(Value::as_str)
            .unwrap_or("UNKNOWN");
        let subject = if kind == "inline-pattern" {
            "refactorSources"
        } else {
            match kind {
                "name" => lookup.get("intentName"),
                "file" => lookup.get("intentFile"),
                "dependency" => lookup.get("depName"),
                "shape" => lookup.get("shapeHash"),
                _ => None,
            }
            .and_then(Value::as_str)
            .unwrap_or("unknown")
        };
        rows.push(format!("- `{kind}` `{subject}`: `{result}`"));
    }
    if !rows.is_empty() {
        lines.push("### Lookup results".to_string());
        lines.push(String::new());
        lines.extend(rows);
        lines.push(String::new());
    }
}

fn render_drift(advisory: &Value, lines: &mut Vec<String>) {
    let drift = advisory
        .get("drift")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    if drift.is_empty() {
        return;
    }
    lines.push("### CANONICAL DRIFT:".to_string());
    lines.push(String::new());
    for entry in drift {
        lines.push(format!(
            "- `{}`: `{}`; canonical owner `{}`.",
            entry
                .get("intentName")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            entry
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            entry
                .get("canonicalOwner")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        ));
    }
    lines.push(String::new());
}

fn render_planned_escapes(advisory: &Value, lines: &mut Vec<String>) {
    lines.push("### Planned type escapes".to_string());
    lines.push(String::new());
    let escapes = advisory
        .pointer("/intent/plannedTypeEscapes")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    if escapes.is_empty() {
        lines.push("- None declared.".to_string());
    } else {
        for (index, escape) in escapes.iter().enumerate() {
            lines.push(format!(
                "- Planned escape #{}: `{}` at `{}`.",
                index + 1,
                escape
                    .get("escapeKind")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                escape
                    .get("locationHint")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            ));
            if let Some(reason) = escape.get("reason").and_then(Value::as_str) {
                lines.push(format!("  reason: {reason}"));
            }
        }
    }
    lines.push(String::new());
}

#[cfg(test)]
mod tests {
    use super::handoff_markdown;
    use serde_json::json;

    #[test]
    fn handoff_reports_complete_counts_without_rendering_candidate_rows() {
        let advisory = json!({
            "artifactPaths": {
                "invocationSpecific": "/repo/.audit/pre-write-advisory.INV-1.json",
            },
            "cueCards": [
                {"candidate": {"identity": "src/a.ts::candidateOne"}},
                {"candidate": {"identity": "src/b.ts::candidateTwo"}},
            ],
            "suppressedCues": [{"reason": "generated-only"}],
            "lookups": [{"kind": "name"}, {"kind": "file"}, {"kind": "shape"}],
            "unavailableEvidence": [{"evidenceLane": "inline-extraction"}],
            "drift": [{"intentName": "oldOwner"}],
            "intent": {
                "plannedTypeEscapes": [{"escapeKind": "as-unknown-as-T"}],
            },
        });

        let handoff = handoff_markdown(&advisory);

        assert!(handoff.contains("pre-write-advisory.INV-1.json"));
        assert!(handoff.contains(
            "cueCards=`2`, suppressedCues=`1`, lookups=`3`, unavailableEvidence=`1`, drift=`1`, plannedTypeEscapes=`1`"
        ));
        assert!(!handoff.contains("candidateOne"));
        assert!(!handoff.contains("candidateTwo"));
    }
}
