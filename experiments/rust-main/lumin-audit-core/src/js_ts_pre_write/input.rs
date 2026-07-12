use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::js_ts_extract::JsTsExtractFileResult;
use crate::scan_scope::{collect_source_files, to_repo_relative, ScanScopeOptions};

use super::cache::extract_with_cache;
use super::protocol::{
    JsTsPreWriteEvidenceRequest, JsTsPreWriteSourceFile,
    JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION,
};

#[derive(Debug)]
pub(super) struct SourceRow {
    pub(super) relative_path: String,
    pub(super) extracted: JsTsExtractFileResult,
}

pub(super) struct PreparedEvidenceInput {
    pub(super) root: PathBuf,
    pub(super) evidence_artifact: String,
    pub(super) any_inventory_artifact: String,
    pub(super) generated: String,
    pub(super) include_tests: bool,
    pub(super) excludes: Vec<String>,
    pub(super) dependency_roots: BTreeSet<String>,
    pub(super) incremental: Value,
    pub(super) rows: Vec<SourceRow>,
    pub(super) path_map: BTreeMap<String, String>,
}

pub(super) fn prepare(request: JsTsPreWriteEvidenceRequest) -> Result<PreparedEvidenceInput> {
    validate_request(&request)?;

    let JsTsPreWriteEvidenceRequest {
        schema_version: _,
        root,
        evidence_artifact,
        any_inventory_artifact,
        generated,
        include_tests,
        excludes,
        dependency_roots,
        discover_files,
        mut files,
        incremental,
    } = request;
    let dependency_roots = dependency_roots.into_iter().collect::<BTreeSet<_>>();
    if discover_files {
        files = discover_js_ts_source_files(&root, include_tests, &excludes)?;
    } else {
        canonicalize_explicit_source_files(&root, &mut files)?;
    }

    let path_map = files
        .iter()
        .map(|file| {
            (
                normalized_path(&file.file_path),
                normalize_slashes(&file.artifact_file_path),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let source_files = files
        .iter()
        .map(|file| file.file_path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let (extracted, incremental) = extract_with_cache(
        &root,
        files,
        source_files,
        include_tests,
        &excludes,
        &incremental,
    )?;
    if extracted.len() != path_map.len() {
        bail!(
            "js-ts-pre-write-evidence: extractor returned {} rows for {} files",
            extracted.len(),
            path_map.len()
        );
    }

    let mut rows = Vec::with_capacity(extracted.len());
    let mut seen = BTreeSet::new();
    for file in extracted {
        let normalized = normalize_slashes(&file.file_path);
        let relative_path = path_map.get(&normalized).with_context(|| {
            format!(
                "js-ts-pre-write-evidence: extractor returned out-of-scope file {}",
                file.file_path
            )
        })?;
        if !seen.insert(normalized) {
            bail!(
                "js-ts-pre-write-evidence: extractor returned duplicate file {}",
                file.file_path
            );
        }
        if file
            .error
            .as_deref()
            .is_some_and(|error| error.starts_with("failed to read source:"))
        {
            bail!(
                "js-ts-pre-write-evidence: failed to read required source {}: {}",
                relative_path,
                file.error.as_deref().unwrap_or("unknown read error")
            );
        }
        rows.push(SourceRow {
            relative_path: relative_path.clone(),
            extracted: file,
        });
    }
    rows.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    Ok(PreparedEvidenceInput {
        root,
        evidence_artifact,
        any_inventory_artifact,
        generated,
        include_tests,
        excludes,
        dependency_roots,
        incremental,
        rows,
        path_map,
    })
}

pub(super) fn package_root(specifier: &str) -> Option<String> {
    if specifier.is_empty()
        || specifier.starts_with('.')
        || specifier.starts_with('/')
        || is_windows_absolute_like(specifier)
    {
        return None;
    }
    if specifier.starts_with('@') {
        let mut parts = specifier.split('/');
        let scope = parts.next()?;
        let package = parts.next()?;
        if package.is_empty() {
            return None;
        }
        return Some(format!("{scope}/{package}"));
    }
    specifier.split('/').next().map(str::to_string)
}

pub(super) fn normalize_slashes(value: &str) -> String {
    value.replace('\\', "/")
}

fn validate_request(request: &JsTsPreWriteEvidenceRequest) -> Result<()> {
    if request.schema_version != JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION {
        bail!(
            "js-ts-pre-write-evidence: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if request.root.as_os_str().is_empty() || !request.root.is_absolute() {
        bail!("js-ts-pre-write-evidence: root must be an absolute path");
    }
    validate_artifact_path(&request.evidence_artifact)?;
    validate_artifact_path(&request.any_inventory_artifact)?;
    if request.generated.trim().is_empty() {
        bail!("js-ts-pre-write-evidence: generated must be a non-empty string");
    }
    if request.discover_files && !request.files.is_empty() {
        bail!("js-ts-pre-write-evidence: discoverFiles and explicit files are mutually exclusive");
    }
    let mut previous_dependency = None::<&str>;
    for dependency in &request.dependency_roots {
        if package_root(dependency).as_deref() != Some(dependency.as_str()) {
            bail!(
                "js-ts-pre-write-evidence: dependencyRoots must contain normalized package roots"
            );
        }
        if previous_dependency.is_some_and(|previous| previous >= dependency.as_str()) {
            bail!("js-ts-pre-write-evidence: dependencyRoots must be strictly sorted");
        }
        previous_dependency = Some(dependency);
    }
    let mut previous = None::<&str>;
    let mut absolute_paths = BTreeSet::new();
    for file in &request.files {
        validate_artifact_path(&file.artifact_file_path)?;
        if !file.file_path.is_absolute() || !file.file_path.starts_with(&request.root) {
            bail!(
                "js-ts-pre-write-evidence: filePath must stay inside root: {}",
                file.file_path.display()
            );
        }
        if let Some(previous) = previous {
            if previous >= file.artifact_file_path.as_str() {
                bail!(
                    "js-ts-pre-write-evidence: files must be strictly sorted by artifactFilePath"
                );
            }
        }
        previous = Some(&file.artifact_file_path);
        if !absolute_paths.insert(normalized_path(&file.file_path)) {
            bail!(
                "js-ts-pre-write-evidence: duplicate filePath {}",
                file.file_path.display()
            );
        }
    }
    Ok(())
}

fn canonicalize_explicit_source_files(
    root: &Path,
    files: &mut [JsTsPreWriteSourceFile],
) -> Result<()> {
    let canonical_root = fs::canonicalize(root).with_context(|| {
        format!(
            "js-ts-pre-write-evidence: failed to canonicalize root {}",
            root.display()
        )
    })?;
    for file in files {
        let canonical_file = fs::canonicalize(&file.file_path).with_context(|| {
            format!(
                "js-ts-pre-write-evidence: failed to read required source {}",
                file.artifact_file_path
            )
        })?;
        if !canonical_file.starts_with(&canonical_root) {
            bail!(
                "js-ts-pre-write-evidence: filePath must stay inside root: {}",
                file.file_path.display()
            );
        }
        file.file_path = canonical_file;
    }
    Ok(())
}

fn discover_js_ts_source_files(
    root: &Path,
    include_tests: bool,
    excludes: &[String],
) -> Result<Vec<JsTsPreWriteSourceFile>> {
    let files = collect_source_files(
        root,
        &ScanScopeOptions {
            include_tests,
            exclude: excludes.to_vec(),
            languages: ["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            directory: false,
        },
    )?;
    files
        .into_iter()
        .map(|file_path| {
            let artifact_file_path = to_repo_relative(root, &file_path.to_string_lossy())
                .with_context(|| {
                    format!(
                        "js-ts-pre-write-evidence: discovered file escaped root: {}",
                        file_path.display()
                    )
                })?;
            Ok(JsTsPreWriteSourceFile {
                file_path,
                artifact_file_path,
            })
        })
        .collect()
}

fn validate_artifact_path(value: &str) -> Result<()> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        bail!("js-ts-pre-write-evidence: artifactFilePath must be a safe repo-relative path");
    }
    Ok(())
}

fn is_windows_absolute_like(value: &str) -> bool {
    value.len() >= 3 && value.as_bytes()[1] == b':' && matches!(value.as_bytes()[2], b'/' | b'\\')
}

fn normalized_path(path: &Path) -> String {
    normalize_slashes(&path.to_string_lossy())
}
