use std::fs;
use std::path::{Path, PathBuf};

use lumin_rust_common::{posix_path_has_segment, posix_path_text};
use lumin_rust_source_health::protocol::{
    HealthResponse, PathClassification, SkippedFile, SkippedFileReason,
};
use lumin_rust_source_health::{is_test_like_rust_path, RustFileScanScope};

use super::super::intent::NormalizedIntent;
use domain_cluster::{find_domain_cluster, DomainCluster};
pub(in crate::prewrite) use domain_cluster::{
    DOMAIN_CLUSTER_MAX_EXAMPLES, DOMAIN_CLUSTER_MIN_MATCHES, DOMAIN_CLUSTER_MIN_PREFIX_LEN,
};
pub(in crate::prewrite) use model::{FileLookup, FileLookupResult};

mod domain_cluster;
mod model;

pub(in crate::prewrite) fn lookup_files(
    intent: &NormalizedIntent,
    syntax: &HealthResponse,
    root: &Path,
) -> Vec<FileLookup> {
    intent
        .files
        .iter()
        .map(|file| lookup_file(file, syntax, root))
        .collect()
}

fn lookup_file(intent_file: &str, syntax: &HealthResponse, root: &Path) -> FileLookup {
    let normalized = posix_path_text(intent_file).into_owned();
    let mut citations = Vec::new();
    let mut tags = Vec::new();
    let mut result = FileLookupResult::Unknown;
    let path_is_safe = is_safe_relative_posix_path(&normalized);
    let path_is_excluded = is_excluded_by_source_health_path_policy(&normalized);
    let scan_scope_exclusion = scan_scope_exclusion_reason(&normalized, syntax);
    let path_is_rust = normalized.ends_with(".rs");
    let domain_cluster = (path_is_safe && !path_is_excluded && path_is_rust)
        .then(|| find_domain_cluster(&normalized, syntax))
        .flatten();

    if let Some(file) = syntax.files.get(&normalized) {
        result = FileLookupResult::Exists;
        tags = tags_from_classifications(&file.path.classifications);
        citations.push(format!(
            "[grounded, rust-source-health.files['{normalized}'] present; parse.ok = {}]",
            file.parse.ok
        ));
    } else if let Some(skipped) = skipped_file(&normalized, syntax) {
        citations.push(format!(
            "[확인 불가, reason: rust-source-health.skippedFiles contains '{}' with reason '{}'; file exists but Rust syntax evidence is unavailable]",
            skipped.path,
            skipped_reason(skipped.reason)
        ));
    } else if !path_is_safe {
        citations.push(format!(
            "[확인 불가, reason: '{normalized}' is not a safe repo-relative POSIX path]"
        ));
    } else if path_is_excluded {
        citations.push(format!(
            "[확인 불가, reason: '{normalized}' is outside rust-source-health path policy (target/vendor excluded)]"
        ));
    } else if let Some(reason) = scan_scope_exclusion {
        citations.push(format!(
            "[확인 불가, reason: '{normalized}' is outside rust-source-health input scope ({reason})]"
        ));
    } else if !path_is_rust {
        citations.push(format!(
            "[확인 불가, reason: rust-source-health enumerates Rust .rs files only; '{normalized}' is outside this lane]"
        ));
    } else if let Some(symlink_path) = first_symlink_component(root, &normalized) {
        citations.push(format!(
            "[확인 불가, reason: '{symlink_path}' is a symlink; rust-source-health path policy does not follow symlinked files or directories]"
        ));
    } else {
        result = FileLookupResult::New;
        citations.push(format!(
            "[grounded, rust-source-health.files does not contain '{normalized}'; rust-source-health wrapper completed Rust .rs file enumeration under its path policy]"
        ));
    }

    citations.push(
        "[확인 불가, reason: Rust pre-write file intent carries no planned from->to edge; boundary rules are not evaluated]"
            .to_string(),
    );

    FileLookup::new(normalized, result, tags, domain_cluster, citations)
}

fn skipped_file<'a>(path: &str, syntax: &'a HealthResponse) -> Option<&'a SkippedFile> {
    syntax
        .skipped_files
        .iter()
        .find(|skipped| skipped.path == path)
}

fn tags_from_classifications(classifications: &[PathClassification]) -> Vec<&'static str> {
    if classifications.contains(&PathClassification::Test) {
        vec!["test-only"]
    } else {
        Vec::new()
    }
}

fn skipped_reason(reason: SkippedFileReason) -> &'static str {
    match reason {
        SkippedFileReason::InvalidUtf8 => "invalid-utf8",
    }
}

fn is_excluded_by_source_health_path_policy(path: &str) -> bool {
    posix_path_has_segment(path, "target") || posix_path_has_segment(path, "vendor")
}

fn scan_scope_exclusion_reason(path: &str, syntax: &HealthResponse) -> Option<String> {
    let input = syntax.meta.input.as_ref()?;
    let scan_scope = RustFileScanScope::new(input.include_tests, &input.exclude);
    if !scan_scope.include_tests() && is_test_like_rust_path(path) {
        return Some("includeTests=false".to_string());
    }
    scan_scope
        .exclusion_pattern_for_path(path, false)
        .map(|pattern| format!("exclude contains '{pattern}'"))
}

fn is_safe_relative_posix_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains('\\')
        && !path.contains(':')
        && path
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn first_symlink_component(root: &Path, path: &str) -> Option<String> {
    let mut cursor = PathBuf::from(root);
    let mut relative = Vec::new();
    for segment in path.split('/') {
        cursor.push(segment);
        relative.push(segment);
        let Ok(metadata) = fs::symlink_metadata(&cursor) else {
            return None;
        };
        if metadata.file_type().is_symlink() {
            return Some(relative.join("/"));
        }
    }
    None
}
