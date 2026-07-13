use super::protocol::{FileDelta, FileDeltaSummary};
use std::collections::BTreeSet;
use std::path::{Component, Path};

pub fn compute_file_delta(
    root: &Path,
    planned_files: &[String],
    before_files: Option<&[String]>,
    after_files: Option<&[String]>,
    after_scan_failure: Option<String>,
) -> FileDelta {
    let planned = normalize_list(root, planned_files);
    if let Some(reason) = after_scan_failure {
        return incomplete("after-scan-failed", planned, Some(reason));
    }
    let Some(after_files) = after_files else {
        return incomplete("after-missing", planned, None);
    };
    let after = normalize_list(root, after_files);
    let after_set = after.iter().cloned().collect::<BTreeSet<_>>();
    let planned_observed = intersect(&planned, &after_set);
    let planned_missing = minus(&planned, &after_set);
    let Some(before_files) = before_files else {
        return FileDelta {
            status: "baseline-missing".to_string(),
            reason: None,
            planned_files: planned,
            before_count: None,
            after_count: Some(after.len()),
            new_files: None,
            removed: None,
            planned_new: None,
            unexpected_new: None,
            planned_observed: Some(planned_observed),
            planned_missing: Some(planned_missing),
            summary: None,
        };
    };
    let before = normalize_list(root, before_files);
    let before_set = before.iter().cloned().collect::<BTreeSet<_>>();
    let planned_set = planned.iter().cloned().collect::<BTreeSet<_>>();
    let new_files = minus(&after, &before_set);
    let removed = minus(&before, &after_set);
    let planned_new = intersect(&new_files, &planned_set);
    let unexpected_new = minus(&new_files, &planned_set);
    let summary = FileDeltaSummary {
        new_files: new_files.len() as u64,
        removed: removed.len() as u64,
        planned_new: planned_new.len() as u64,
        unexpected_new: unexpected_new.len() as u64,
        planned_observed: planned_observed.len() as u64,
        planned_missing: planned_missing.len() as u64,
    };
    FileDelta {
        status: "computed".to_string(),
        reason: None,
        planned_files: planned,
        before_count: Some(before.len()),
        after_count: Some(after.len()),
        new_files: Some(new_files),
        removed: Some(removed),
        planned_new: Some(planned_new),
        unexpected_new: Some(unexpected_new),
        planned_observed: Some(planned_observed),
        planned_missing: Some(planned_missing),
        summary: Some(summary),
    }
}

pub fn repo_relative_file_list(root: &Path, files: &[std::path::PathBuf]) -> Vec<String> {
    let values = files
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    normalize_list(root, &values)
}

fn incomplete(status: &str, planned_files: Vec<String>, reason: Option<String>) -> FileDelta {
    FileDelta {
        status: status.to_string(),
        reason,
        planned_files,
        before_count: None,
        after_count: None,
        new_files: None,
        removed: None,
        planned_new: None,
        unexpected_new: None,
        planned_observed: None,
        planned_missing: None,
        summary: None,
    }
}

fn normalize_list(root: &Path, values: &[String]) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| normalize_repo_relative_path(root, value))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn normalize_repo_relative_path(root: &Path, value: &str) -> Option<String> {
    let raw = value.trim();
    if raw.is_empty() {
        return None;
    }
    let path = Path::new(raw);
    let relative = if path.is_absolute() {
        match path.strip_prefix(root) {
            Ok(relative) => relative.to_path_buf(),
            Err(_) => {
                let canonical_root = std::fs::canonicalize(root).ok()?;
                path.strip_prefix(canonical_root).ok()?.to_path_buf()
            }
        }
    } else {
        path.to_path_buf()
    };
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::CurDir | Component::RootDir => {}
            Component::Normal(value) => parts.push(value.to_string_lossy().to_string()),
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::Prefix(_) => return None,
        }
    }
    let normalized = parts.join("/");
    (!normalized.is_empty()).then_some(normalized)
}

fn minus(left: &[String], right: &BTreeSet<String>) -> Vec<String> {
    left.iter()
        .filter(|item| !right.contains(*item))
        .cloned()
        .collect()
}

fn intersect(left: &[String], right: &BTreeSet<String>) -> Vec<String> {
    left.iter()
        .filter(|item| right.contains(*item))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_planned_and_unexpected_file_sets() {
        let root = Path::new("C:/repo");
        let delta = compute_file_delta(
            root,
            &["src/planned.ts".to_string(), "src/missing.ts".to_string()],
            Some(&["src/old.ts".to_string()]),
            Some(&[
                "src/planned.ts".to_string(),
                "src/unexpected.ts".to_string(),
            ]),
            None,
        );
        assert_eq!(delta.status, "computed");
        assert_eq!(delta.planned_new, Some(vec!["src/planned.ts".to_string()]));
        assert_eq!(
            delta.unexpected_new,
            Some(vec!["src/unexpected.ts".to_string()])
        );
        assert_eq!(
            delta.planned_missing,
            Some(vec!["src/missing.ts".to_string()])
        );
    }
}
