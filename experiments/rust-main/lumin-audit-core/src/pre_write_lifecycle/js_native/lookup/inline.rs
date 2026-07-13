use super::*;

pub(super) fn lookup(
    refactor_sources: &[Value],
    inline_patterns: &Value,
    evidence_files: &Value,
) -> Value {
    let Some(groups) = inline_patterns.get("groups").and_then(Value::as_array) else {
        return json!({
            "kind": "inline-pattern",
            "result": "UNAVAILABLE",
            "reason": "missing-artifact",
            "artifact": "pre-write-evidence#inlinePatterns",
            "citations": ["[확인 불가, current-run inline pattern evidence is absent]"],
        });
    };
    let mut matched = groups
        .iter()
        .filter(|group| {
            group
                .get("occurrences")
                .and_then(Value::as_array)
                .is_some_and(|occurrences| {
                    refactor_sources.iter().any(|source| {
                        occurrences
                            .iter()
                            .any(|occurrence| occurrence_matches(source, occurrence))
                    })
                })
        })
        .cloned()
        .collect::<Vec<_>>();
    matched.sort_by(|left, right| {
        right
            .get("size")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .cmp(&left.get("size").and_then(Value::as_u64).unwrap_or(0))
            .then_with(|| string_at(left, "patternHash").cmp(string_at(right, "patternHash")))
    });
    let scanned_files = evidence_files
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    let affected_sources = refactor_sources
        .iter()
        .filter_map(|source| source.get("file").and_then(Value::as_str))
        .filter(|file| !scanned_files.contains(file))
        .collect::<Vec<_>>();
    let relevant_diagnostics = inline_patterns
        .get("diagnostics")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|diagnostic| {
            diagnostic
                .get("file")
                .and_then(Value::as_str)
                .is_some_and(|file| {
                    refactor_sources
                        .iter()
                        .any(|source| source.get("file").and_then(Value::as_str) == Some(file))
                })
        })
        .cloned()
        .collect::<Vec<_>>();
    if matched.is_empty() && (!affected_sources.is_empty() || !relevant_diagnostics.is_empty()) {
        return json!({
            "kind": "inline-pattern",
            "result": "UNAVAILABLE",
            "reason": "refactor-source-evidence-incomplete",
            "artifact": "pre-write-evidence#inlinePatterns",
            "unscannedFiles": affected_sources,
            "diagnostics": relevant_diagnostics,
            "citations": ["[확인 불가, one or more refactorSources were not scanned or were skipped; absence is not grounded]"],
        });
    }
    let mut citations = vec![if matched.is_empty() {
        "[grounded, current-run inlinePatterns groups contain no pattern intersecting refactorSources]".to_string()
    } else {
        format!(
            "[grounded, current-run inlinePatterns groups intersect {} refactor source{}]",
            refactor_sources.len(),
            if refactor_sources.len() == 1 { "" } else { "s" }
        )
    }];
    if !affected_sources.is_empty() || !relevant_diagnostics.is_empty() {
        citations.push("[degraded, some requested refactor source evidence was unavailable; positive inline-pattern matches remain grounded]".to_string());
    }
    json!({
        "kind": "inline-pattern",
        "result": if matched.is_empty() { "NO_INLINE_PATTERN_MATCH" } else { "INLINE_PATTERN_MATCH" },
        "groups": matched,
        "citations": citations,
    })
}

fn occurrence_matches(source: &Value, occurrence: &Value) -> bool {
    if source.get("file").and_then(Value::as_str) != occurrence.get("file").and_then(Value::as_str)
    {
        return false;
    }
    let Some(lines) = source.get("lines").and_then(Value::as_array) else {
        return true;
    };
    let start = occurrence.get("line").and_then(Value::as_u64);
    let end = occurrence.get("endLine").and_then(Value::as_u64).or(start);
    match (start, end) {
        (Some(start), Some(end)) => lines
            .iter()
            .filter_map(Value::as_u64)
            .any(|line| (start..=end).contains(&line)),
        _ => false,
    }
}
