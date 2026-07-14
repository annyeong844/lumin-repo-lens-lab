use serde_json::Value;

use super::support::{arr, get, n, object, plural};

fn artifact_name(file_path: Option<&Value>) -> Option<String> {
    let path = file_path?.as_str()?;
    let mut parts = path
        .replace('\\', "/")
        .split('/')
        .map(String::from)
        .collect::<Vec<_>>();
    let start = parts.len().saturating_sub(2);
    Some(parts.drain(start..).collect::<Vec<_>>().join("/"))
}

pub(super) fn summarize_lifecycle_command(manifest: &Value) -> Vec<String> {
    let mut out = Vec::new();

    if get(manifest, "preWrite")
        .and_then(|pre| get(pre, "requested"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        let pre = get(manifest, "preWrite").unwrap_or(&Value::Null);
        if get(pre, "ran").and_then(Value::as_bool) == Some(true) {
            let specific = artifact_name(get(pre, "advisoryPath"))
                .unwrap_or_else(|| "the invocation-specific advisory".to_string());
            let latest = artifact_name(get(pre, "latestAdvisoryPath"))
                .unwrap_or_else(|| "pre-write-advisory.latest.json".to_string());
            out.push(format!("- Pre-write ran and wrote an advisory. Use `{specific}` for the matching post-write check; `{latest}` is only the latest pointer."));
        } else {
            out.push(format!(
                "- Pre-write did not run: {}.",
                get(pre, "reason")
                    .and_then(Value::as_str)
                    .unwrap_or("reason unavailable")
            ));
        }
    }

    if get(manifest, "postWrite")
        .and_then(|post| get(post, "requested"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        let post = get(manifest, "postWrite").unwrap_or(&Value::Null);
        if get(post, "ran").and_then(Value::as_bool) == Some(true) {
            let baseline_status = get(post, "baselineStatus")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let scan_range_parity = get(post, "scanRangeParity")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let after_complete = get(post, "afterComplete").and_then(Value::as_bool) == Some(true);
            if baseline_status != "available" || scan_range_parity != "ok" || !after_complete {
                out.push(format!("- Post-write ran, but delta confidence is limited: baseline={baseline_status}, scanRange={scan_range_parity}, afterComplete={after_complete}. Read `post-write-delta.latest.json` before closing."));
            } else {
                let silent_new = n(get(post, "silentNew"));
                out.push(format!(
                    "- Post-write type-escape delta found {silent_new} {}. This is not a full behavior verdict.",
                    plural(silent_new, "new unplanned any-like escape", None)
                ));
            }
            let unexpected_new_files = n(get(post, "unexpectedNewFileCount"));
            let planned_missing_files = n(get(post, "plannedMissingFileCount"));
            if unexpected_new_files > 0 || planned_missing_files > 0 {
                out.push(format!("- Post-write file delta needs review: {unexpected_new_files} unexpected new {}, {planned_missing_files} planned missing {}. Read `post-write-delta.latest.json` before closing.",
                    plural(unexpected_new_files, "file", None),
                    plural(planned_missing_files, "file", None)
                ));
            }
        } else {
            out.push(format!(
                "- Post-write did not run: {}.",
                get(post, "reason")
                    .and_then(Value::as_str)
                    .unwrap_or("reason unavailable")
            ));
        }
    }

    if get(manifest, "canonDraft")
        .and_then(|draft| get(draft, "requested"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        let draft = get(manifest, "canonDraft").unwrap_or(&Value::Null);
        let draft_paths = arr(get(draft, "draftPaths"));
        let draft_count = draft_paths.len() as i64;
        if get(draft, "ran").and_then(Value::as_bool) == Some(true) && draft_count > 0 {
            let shown = draft_paths
                .iter()
                .take(3)
                .filter_map(|path| artifact_name(Some(path)))
                .collect::<Vec<_>>()
                .join(", ");
            let more = if draft_count > 3 {
                format!(", plus {} more", draft_count - 3)
            } else {
                String::new()
            };
            out.push(format!(
                "- Canon draft wrote {draft_count} proposal {} under canonical-draft/. Review manually before promotion.{}",
                plural(draft_count, "file", None),
                if shown.is_empty() {
                    String::new()
                } else {
                    format!(" Drafts: {shown}{more}.")
                }
            ));
        } else if get(draft, "ran").and_then(Value::as_bool) == Some(true) {
            out.push("- Canon draft ran, but no proposal path was recorded. Check per-source status before promotion.".to_string());
        } else {
            out.push(format!(
                "- Canon draft did not write proposals: {}.",
                get(draft, "reason")
                    .and_then(Value::as_str)
                    .unwrap_or("all requested sources failed")
            ));
        }
    }

    if get(manifest, "checkCanon")
        .and_then(|check| get(check, "requested"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        let check = get(manifest, "checkCanon").unwrap_or(&Value::Null);
        let summary = get(check, "summary").unwrap_or(&Value::Null);
        let drift_count = n(get(summary, "driftCount"));
        let checked = n(get(summary, "sourcesChecked"));
        let skipped = n(get(summary, "sourcesSkipped"));
        let failed = n(get(summary, "sourcesFailed"));
        if get(check, "ran").and_then(Value::as_bool) != Some(true) {
            out.push(format!(
                "- Check-canon did not run: {}.",
                get(check, "reason")
                    .and_then(Value::as_str)
                    .unwrap_or("reason unavailable")
            ));
        } else if checked == 0 {
            out.push(format!(
                "- Check-canon could not compare promoted canon yet: {skipped} {}, {failed} failed.",
                plural(skipped, "area", None)
            ));
        } else if drift_count > 0 {
            let drift_sources = object(get(check, "driftCounts"))
                .map(|object| object.values().filter(|count| n(Some(count)) > 0).count() as i64)
                .unwrap_or(0);
            out.push(format!(
                "- Check-canon found {drift_count} drift {} across {drift_sources}/{checked} checked {}.",
                plural(drift_count, "item", None),
                plural(checked, "area", None)
            ));
        } else {
            let caveat_count = skipped + failed;
            out.push(
                format!(
                    "- Check-canon is clean across {checked} checked {}.{}",
                    plural(checked, "area", None),
                    if caveat_count > 0 {
                        format!(
                            " {caveat_count} {} could not be checked.",
                            plural(caveat_count, "area", None)
                        )
                    } else {
                        String::new()
                    }
                )
                .trim()
                .to_string(),
            );
        }
    }

    out
}
