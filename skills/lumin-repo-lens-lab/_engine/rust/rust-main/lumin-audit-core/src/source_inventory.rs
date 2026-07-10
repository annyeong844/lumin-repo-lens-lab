use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(crate) const SOURCE_INVENTORY_SCHEMA_VERSION: &str = "lumin-source-inventory.v2";
pub(crate) const SOURCE_INVENTORY_POLICY_VERSION: &str = "lumin-source-walk.v1";
pub(crate) const SOURCE_INVENTORY_FILE_NAME: &str = "source-inventory.json";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceInventoryArtifact {
    schema_version: String,
    producer: String,
    run_id: String,
    root: PathBuf,
    path_mode: String,
    walk_scope: SourceInventoryWalkScope,
    analysis_scope: SourceInventoryAnalysisScope,
    file_count: usize,
    counts_by_language: BTreeMap<String, usize>,
    files: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceInventoryWalkScope {
    include_tests: bool,
    exclude: Vec<String>,
    languages: Vec<String>,
    policy_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceInventoryAnalysisScope {
    include_tests: bool,
    exclude: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct ValidatedSourceInventory {
    path: PathBuf,
    files: Vec<String>,
    counts_by_language: BTreeMap<String, usize>,
}

impl ValidatedSourceInventory {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn file_count_for_extensions(&self, extensions: &[&str]) -> usize {
        self.files
            .iter()
            .filter(|file| extensions.iter().any(|extension| file.ends_with(extension)))
            .count()
    }

    pub(crate) fn is_rust_only(&self) -> bool {
        self.counts_by_language.get("rs").copied().unwrap_or(0) > 0
            && self
                .counts_by_language
                .iter()
                .all(|(language, count)| language == "rs" || *count == 0)
    }
}

pub(crate) fn load_source_inventory(
    path: &Path,
    expected_run_id: &str,
    expected_root: &Path,
    expected_include_tests: bool,
    expected_excludes: &[String],
) -> Result<ValidatedSourceInventory> {
    validate_run_id(expected_run_id)?;
    let bytes = fs::read(path).with_context(|| {
        format!(
            "source inventory missing or unreadable at {}",
            path.display()
        )
    })?;
    let artifact: SourceInventoryArtifact = serde_json::from_slice(&bytes)
        .with_context(|| format!("source inventory malformed at {}", path.display()))?;

    if artifact.schema_version != SOURCE_INVENTORY_SCHEMA_VERSION {
        bail!(
            "source inventory has unsupported schemaVersion '{}'",
            artifact.schema_version
        );
    }
    if artifact.producer != "triage-repo.mjs" {
        bail!(
            "source inventory has unsupported producer '{}'",
            artifact.producer
        );
    }
    validate_run_id(&artifact.run_id)?;
    if artifact.run_id != expected_run_id {
        bail!(
            "source inventory runId mismatch: expected '{}', observed '{}'",
            expected_run_id,
            artifact.run_id
        );
    }
    if artifact.path_mode != "repo-relative" {
        bail!(
            "source inventory has unsupported pathMode '{}'",
            artifact.path_mode
        );
    }
    if artifact.root != expected_root {
        bail!(
            "source inventory root mismatch: expected '{}', observed '{}'",
            expected_root.display(),
            artifact.root.display()
        );
    }
    if !artifact.walk_scope.include_tests {
        bail!("source inventory walkScope must include tests");
    }
    if artifact.walk_scope.policy_version != SOURCE_INVENTORY_POLICY_VERSION {
        bail!(
            "source inventory has unsupported walk policy '{}'",
            artifact.walk_scope.policy_version
        );
    }
    if artifact.walk_scope.exclude != expected_excludes {
        bail!(
            "source inventory walkScope exclude mismatch: expected {:?}, observed {:?}",
            expected_excludes,
            artifact.walk_scope.exclude
        );
    }
    if artifact.analysis_scope.include_tests != expected_include_tests {
        bail!(
            "source inventory analysisScope.includeTests mismatch: expected {}, observed {}",
            expected_include_tests,
            artifact.analysis_scope.include_tests
        );
    }
    if artifact.analysis_scope.exclude != expected_excludes {
        bail!(
            "source inventory analysisScope exclude mismatch: expected {:?}, observed {:?}",
            expected_excludes,
            artifact.analysis_scope.exclude
        );
    }
    if artifact.walk_scope.languages.is_empty() {
        bail!("source inventory languages must not be empty");
    }
    validate_sorted_unique(&artifact.walk_scope.languages, "languages")?;
    if artifact.file_count != artifact.files.len() {
        bail!(
            "source inventory fileCount {} does not match files length {}",
            artifact.file_count,
            artifact.files.len()
        );
    }
    validate_sorted_unique(&artifact.files, "files")?;
    for file in &artifact.files {
        validate_repo_relative_path(file)?;
    }
    validate_language_counts(
        &artifact.walk_scope.languages,
        &artifact.counts_by_language,
        &artifact.files,
    )?;

    Ok(ValidatedSourceInventory {
        path: path.to_path_buf(),
        files: artifact.files,
        counts_by_language: artifact.counts_by_language,
    })
}

fn validate_run_id(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        bail!("source inventory runId must contain 1-128 safe identifier characters");
    }
    Ok(())
}

fn validate_sorted_unique(values: &[String], field: &str) -> Result<()> {
    for pair in values.windows(2) {
        if pair[0] >= pair[1] {
            bail!(
                "source inventory {field} must be strictly sorted and unique near '{}'",
                pair[1]
            );
        }
    }
    Ok(())
}

fn validate_language_counts(
    languages: &[String],
    observed: &BTreeMap<String, usize>,
    files: &[String],
) -> Result<()> {
    let mut expected = languages
        .iter()
        .map(|language| (language.clone(), 0usize))
        .collect::<BTreeMap<_, _>>();
    for file in files {
        let Some(language) = Path::new(file).extension().and_then(|value| value.to_str()) else {
            bail!("source inventory file has no language extension: {file}");
        };
        let Some(count) = expected.get_mut(language) else {
            bail!("source inventory file has unsupported language extension: {file}");
        };
        *count += 1;
    }
    if observed != &expected {
        bail!("source inventory countsByLanguage mismatch: expected {expected:?}, observed {observed:?}");
    }
    Ok(())
}

fn validate_repo_relative_path(value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    let drive_prefixed = bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic();
    if value.is_empty()
        || value.contains('\0')
        || value.contains('\\')
        || value.starts_with('/')
        || drive_prefixed
    {
        bail!("source inventory contains unsafe repo-relative path '{value}'");
    }
    if !Path::new(value)
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        bail!("source inventory contains unsafe repo-relative path '{value}'");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_inventory(path: &Path, root: &Path, run_id: &str, files: &[&str]) -> Result<()> {
        let rs_files = files.iter().filter(|file| file.ends_with(".rs")).count();
        let ts_files = files.iter().filter(|file| file.ends_with(".ts")).count();
        fs::write(
            path,
            serde_json::to_vec(&json!({
                "schemaVersion": SOURCE_INVENTORY_SCHEMA_VERSION,
                "producer": "triage-repo.mjs",
                "runId": run_id,
                "root": root,
                "pathMode": "repo-relative",
                "walkScope": {
                    "includeTests": true,
                    "exclude": ["vendor"],
                    "languages": ["rs", "ts"],
                    "policyVersion": SOURCE_INVENTORY_POLICY_VERSION
                },
                "analysisScope": {
                    "includeTests": true,
                    "exclude": ["vendor"]
                },
                "fileCount": files.len(),
                "countsByLanguage": { "rs": rs_files, "ts": ts_files },
                "files": files,
            }))?,
        )?;
        Ok(())
    }

    #[test]
    fn accepts_current_run_inventory_and_counts_languages() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join(SOURCE_INVENTORY_FILE_NAME);
        write_inventory(
            &path,
            temp.path(),
            "current-run",
            &["src/lib.rs", "src/main.ts"],
        )?;

        let inventory = load_source_inventory(
            &path,
            "current-run",
            temp.path(),
            true,
            &["vendor".to_string()],
        )?;
        assert_eq!(inventory.file_count_for_extensions(&[".ts"]), 1);
        assert_eq!(inventory.file_count_for_extensions(&[".rs"]), 1);
        Ok(())
    }

    #[test]
    fn rejects_stale_run_scope_mismatch_and_unsafe_paths() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join(SOURCE_INVENTORY_FILE_NAME);
        write_inventory(&path, temp.path(), "old-run", &["../escape.rs"])?;

        let error = load_source_inventory(
            &path,
            "current-run",
            temp.path(),
            true,
            &["vendor".to_string()],
        )
        .err()
        .context("stale run ids must fail closed before evidence reuse")?;
        assert!(error.to_string().contains("runId mismatch"));

        write_inventory(&path, temp.path(), "current-run", &["../escape.rs"])?;
        let error = load_source_inventory(
            &path,
            "current-run",
            temp.path(),
            true,
            &["vendor".to_string()],
        )
        .err()
        .context("unsafe paths must fail closed")?;
        assert!(error.to_string().contains("unsafe repo-relative path"));

        write_inventory(&path, temp.path(), "current-run", &["src/lib.rs"])?;
        let error = load_source_inventory(
            &path,
            "current-run",
            temp.path(),
            false,
            &["vendor".to_string()],
        )
        .err()
        .context("scope mismatch must fail closed")?;
        assert!(error
            .to_string()
            .contains("analysisScope.includeTests mismatch"));
        Ok(())
    }
}
