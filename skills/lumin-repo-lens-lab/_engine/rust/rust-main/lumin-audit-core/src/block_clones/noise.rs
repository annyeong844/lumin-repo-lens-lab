use std::collections::{BTreeMap, HashSet};

use super::groups::{compare_groups, BlockCloneGroup};
use super::policy::Thresholds;

#[derive(Debug)]
pub(super) struct NoisePolicyResult {
    pub(super) groups: Vec<BlockCloneGroup>,
    pub(super) review_group_count: usize,
    pub(super) muted_group_count: usize,
    pub(super) muted_by_reason: BTreeMap<String, usize>,
    pub(super) candidate_cap_saturated: bool,
    pub(super) review_cap_saturated: bool,
    pub(super) muted_cap_saturated: bool,
}

fn is_test_file(file: &str) -> bool {
    let rel = slash_path(file).to_lowercase();
    let base = posix_basename(&rel);
    rel.starts_with("tests/")
        || rel.contains("/tests/")
        || base.starts_with("test-")
        || has_test_or_spec_extension(base)
}

fn has_test_or_spec_extension(base: &str) -> bool {
    const EXTENSIONS: &[&str] = &[
        "js", "jsx", "ts", "tsx", "mjs", "mjsx", "mts", "mtsx", "cjs", "cjsx", "cts", "ctsx",
    ];
    EXTENSIONS.iter().any(|ext| {
        base.ends_with(&format!(".test.{ext}")) || base.ends_with(&format!(".spec.{ext}"))
    })
}

fn test_mirror_entry(file: &str) -> Option<(String, &'static str)> {
    let rel = slash_path(file).to_lowercase();
    if !is_test_file(&rel) {
        return None;
    }
    let base = posix_basename(&rel);
    let dir = posix_dirname(&rel);
    if let Some(stripped) = base.strip_prefix("test-") {
        return Some((format!("{dir}/{}", strip_js_extension(stripped)), "node"));
    }
    if let Some(stem) = strip_test_spec_extension(base) {
        return Some((format!("{dir}/{stem}"), "vitest"));
    }
    None
}

fn has_node_vitest_mirror_pair(files: &[String]) -> bool {
    let mut kinds_by_key = BTreeMap::<String, HashSet<&'static str>>::new();
    for file in files {
        let Some((key, kind)) = test_mirror_entry(file) else {
            continue;
        };
        kinds_by_key.entry(key).or_default().insert(kind);
    }
    kinds_by_key
        .values()
        .any(|kinds| kinds.contains("node") && kinds.contains("vitest"))
}

fn classify_noise(group: &BlockCloneGroup) -> (&'static str, Option<&'static str>) {
    let mut files = group
        .instances
        .iter()
        .map(|instance| slash_path(&instance.file))
        .filter(|file| !file.is_empty())
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    if files.is_empty() {
        return ("review", None);
    }
    let all_test = files.iter().all(|file| is_test_file(file));
    if all_test && has_node_vitest_mirror_pair(&files) {
        return ("muted", Some("node-vitest-mirror-pair"));
    }
    if files.len() == 1 {
        return ("muted", Some("same-file-repeat"));
    }
    if all_test {
        return ("muted", Some("test-scaffold-repeat"));
    }
    ("review", None)
}

pub(super) fn apply_noise_policy(
    groups: Vec<BlockCloneGroup>,
    thresholds: &Thresholds,
) -> NoisePolicyResult {
    let mut ranked_candidates = groups;
    ranked_candidates.sort_by(compare_groups);
    let candidate_cap_saturated = ranked_candidates.len() > thresholds.max_candidate_groups;
    ranked_candidates.truncate(thresholds.max_candidate_groups);

    let mut classified = ranked_candidates
        .into_iter()
        .map(|mut group| {
            let (visibility, mute_reason) = classify_noise(&group);
            group.visibility = Some(visibility.to_string());
            group.mute_reason = mute_reason.map(str::to_string);
            group
        })
        .collect::<Vec<_>>();

    let mut review = classified
        .iter()
        .filter(|group| group.visibility.as_deref() != Some("muted"))
        .cloned()
        .collect::<Vec<_>>();
    let mut muted = classified
        .drain(..)
        .filter(|group| group.visibility.as_deref() == Some("muted"))
        .collect::<Vec<_>>();
    review.sort_by(compare_groups);
    muted.sort_by(compare_groups);

    let review_cap_saturated = review.len() > thresholds.max_review_groups;
    let muted_cap_saturated = muted.len() > thresholds.max_muted_groups;
    review.truncate(thresholds.max_review_groups);
    muted.truncate(thresholds.max_muted_groups);

    if let Some(max_groups) = thresholds.max_groups {
        review.truncate(max_groups);
        let remaining = max_groups.saturating_sub(review.len());
        muted.truncate(remaining);
    }

    let mut muted_by_reason = BTreeMap::<String, usize>::new();
    for group in &muted {
        if let Some(reason) = &group.mute_reason {
            *muted_by_reason.entry(reason.clone()).or_insert(0) += 1;
        }
    }

    let review_group_count = review.len();
    let muted_group_count = muted.len();
    review.extend(muted);
    NoisePolicyResult {
        groups: review,
        review_group_count,
        muted_group_count,
        muted_by_reason,
        candidate_cap_saturated,
        review_cap_saturated,
        muted_cap_saturated,
    }
}

fn slash_path(value: &str) -> String {
    value.replace('\\', "/")
}

fn posix_basename(path: &str) -> &str {
    path.rsplit_once('/').map(|(_, base)| base).unwrap_or(path)
}

fn posix_dirname(path: &str) -> &str {
    path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or(".")
}

fn strip_js_extension(base: &str) -> String {
    const EXTENSIONS: &[&str] = &[
        ".mjsx", ".mtsx", ".cjsx", ".ctsx", ".jsx", ".tsx", ".mjs", ".mts", ".cjs", ".cts", ".js",
        ".ts",
    ];
    for extension in EXTENSIONS {
        if let Some(stripped) = base.strip_suffix(extension) {
            return stripped.to_string();
        }
    }
    base.to_string()
}

fn strip_test_spec_extension(base: &str) -> Option<String> {
    for marker in [".test.", ".spec."] {
        let Some(index) = base.rfind(marker) else {
            continue;
        };
        let extension = &base[index + marker.len()..];
        if is_js_extension(extension) {
            return Some(base[..index].to_string());
        }
    }
    None
}

fn is_js_extension(extension: &str) -> bool {
    matches!(
        extension,
        "js" | "jsx"
            | "ts"
            | "tsx"
            | "mjs"
            | "mjsx"
            | "mts"
            | "mtsx"
            | "cjs"
            | "cjsx"
            | "cts"
            | "ctsx"
    )
}
