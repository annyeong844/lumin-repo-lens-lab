use super::JsTsPreWriteSourceFile;
use crate::js_ts_extract::{
    build_js_ts_extract_response, JsTsExtractFileResult, JsTsExtractInputFile, JsTsExtractRequest,
    JS_TS_EXTRACT_REQUEST_SCHEMA_VERSION,
};
use anyhow::{bail, Context, Result};
use lumin_rust_common::{sha256_bytes, sha256_text};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const CACHE_SCHEMA_VERSION: u32 = 1;
const CACHE_PROFILE_VERSION: &str =
    "js-ts-pre-write-oxc-facts.v1+oxc-0.139.0+audit-core-bridge-v44";
const CACHE_FILE_NAME: &str = "facts.json";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JsTsPreWriteIncrementalRequest {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub cache_root: Option<PathBuf>,
    #[serde(default)]
    pub clear: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FileIdentity {
    mode: String,
    value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CacheEntry {
    identity: FileIdentity,
    result: JsTsExtractFileResult,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CacheFile {
    schema_version: u32,
    profile_version: String,
    root_fingerprint: String,
    source_set_fingerprint: String,
    scan_context_fingerprint: String,
    entries: BTreeMap<String, CacheEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CacheFileRef<'a> {
    schema_version: u32,
    profile_version: &'static str,
    root_fingerprint: &'a str,
    source_set_fingerprint: &'a str,
    scan_context_fingerprint: &'a str,
    entries: BTreeMap<&'a str, CacheEntryRef<'a>>,
}

#[derive(Serialize)]
struct CacheEntryRef<'a> {
    identity: &'a FileIdentity,
    result: &'a JsTsExtractFileResult,
}

struct CurrentFile {
    absolute_path: String,
    artifact_path: String,
    identity: FileIdentity,
    source: Option<String>,
}

#[derive(Default)]
struct CacheObservations {
    changed_files: usize,
    reused_files: usize,
    dropped_files: usize,
    invalidated_files: usize,
    git_blob_files: usize,
    content_hash_files: usize,
    load_status: String,
    write_status: String,
    reason: Option<String>,
}

pub(super) fn extract_with_cache(
    root: &Path,
    files: Vec<JsTsPreWriteSourceFile>,
    source_files: Vec<String>,
    include_tests: bool,
    excludes: &[String],
    request: &JsTsPreWriteIncrementalRequest,
) -> Result<(Vec<JsTsExtractFileResult>, Value)> {
    if !request.enabled {
        let changed_files = files.len();
        let extracted = extract_inputs(
            files.into_iter().map(input_without_source).collect(),
            source_files,
        )?;
        return Ok((
            extracted,
            incremental_json(
                request,
                None,
                CacheObservations {
                    changed_files,
                    load_status: "disabled".to_string(),
                    write_status: "disabled".to_string(),
                    reason: Some("disabled-by-request".to_string()),
                    ..CacheObservations::default()
                },
            ),
        ));
    }

    let cache_root = request
        .cache_root
        .as_deref()
        .context("js-ts-pre-write-evidence: incremental cacheRoot is required when enabled")?;
    if !cache_root.is_absolute() {
        bail!("js-ts-pre-write-evidence: incremental cacheRoot must be absolute");
    }

    let root_fingerprint = root_fingerprint(root)?;
    let source_set_fingerprint = source_set_fingerprint(&files);
    let scan_context_fingerprint = scan_context_fingerprint(include_tests, excludes);
    let cache_path = cache_path(cache_root, &root_fingerprint);
    if request.clear {
        remove_cache_file(&cache_path)?;
    }

    let (mut prior, mut load_status) = load_cache(&cache_path);
    let compatible = prior.as_ref().is_some_and(|cache| {
        cache.schema_version == CACHE_SCHEMA_VERSION
            && cache.profile_version == CACHE_PROFILE_VERSION
            && cache.root_fingerprint == root_fingerprint
            && cache.source_set_fingerprint == source_set_fingerprint
            && cache.scan_context_fingerprint == scan_context_fingerprint
    });
    if prior.is_some() && !compatible {
        load_status = "ignored-incompatible".to_string();
    }
    let clean_git_blobs = clean_git_blob_ids(root);
    let mut observations = CacheObservations {
        load_status,
        write_status: "pending".to_string(),
        ..CacheObservations::default()
    };
    let mut current = Vec::with_capacity(files.len());
    for file in files {
        let artifact_path = normalize_slashes(&file.artifact_file_path);
        let absolute_path = file.file_path.to_string_lossy().to_string();
        let (identity, source) = if let Some(blob) = clean_git_blobs
            .as_ref()
            .and_then(|blobs| blobs.get(&artifact_path))
        {
            observations.git_blob_files += 1;
            (
                FileIdentity {
                    mode: "git-blob".to_string(),
                    value: blob.clone(),
                },
                None,
            )
        } else {
            let bytes = fs::read(&file.file_path).with_context(|| {
                format!("js-ts-pre-write-evidence: failed to read required source {artifact_path}")
            })?;
            let identity = FileIdentity {
                mode: "sha256".to_string(),
                value: sha256_bytes(&bytes),
            };
            let source = String::from_utf8(bytes).with_context(|| {
                format!("js-ts-pre-write-evidence: required source is not UTF-8: {artifact_path}")
            })?;
            observations.content_hash_files += 1;
            (identity, Some(source))
        };
        current.push(CurrentFile {
            absolute_path,
            artifact_path,
            identity,
            source,
        });
    }

    let current_paths = current
        .iter()
        .map(|file| file.artifact_path.as_str())
        .collect::<BTreeSet<_>>();
    observations.dropped_files = prior
        .as_ref()
        .map(|cache| {
            cache
                .entries
                .keys()
                .filter(|path| !current_paths.contains(path.as_str()))
                .count()
        })
        .unwrap_or(0);
    if !compatible {
        observations.invalidated_files = prior
            .as_ref()
            .map(|cache| {
                cache
                    .entries
                    .keys()
                    .filter(|path| current_paths.contains(path.as_str()))
                    .count()
            })
            .unwrap_or(0);
    }
    let mut prior_entries = if compatible {
        prior.take().map(|cache| cache.entries).unwrap_or_default()
    } else {
        BTreeMap::new()
    };

    let mut results = BTreeMap::<String, JsTsExtractFileResult>::new();
    let mut pending = Vec::new();
    let mut identities = BTreeMap::new();
    let mut absolute_to_artifact = BTreeMap::new();
    for file in current {
        let prior_entry = prior_entries.remove(&file.artifact_path);
        let had_prior_entry = prior_entry.is_some();
        let reusable = prior_entry.filter(|entry| {
            entry.identity == file.identity
                && normalize_slashes(&entry.result.file_path)
                    == normalize_slashes(&file.absolute_path)
        });
        absolute_to_artifact.insert(
            normalize_slashes(&file.absolute_path),
            file.artifact_path.clone(),
        );
        if let Some(entry) = reusable {
            observations.reused_files += 1;
            identities.insert(file.artifact_path.clone(), file.identity);
            results.insert(file.artifact_path, entry.result);
            continue;
        }
        observations.changed_files += 1;
        if compatible && had_prior_entry {
            observations.invalidated_files += 1;
        }
        identities.insert(file.artifact_path.clone(), file.identity);
        pending.push(JsTsExtractInputFile {
            file_path: file.absolute_path,
            artifact_file_path: Some(file.artifact_path),
            source: file.source,
        });
    }

    for result in extract_inputs(pending, source_files)? {
        let artifact_path = absolute_to_artifact
            .get(&normalize_slashes(&result.file_path))
            .cloned()
            .with_context(|| {
                format!(
                    "js-ts-pre-write-evidence: extractor returned out-of-scope file {}",
                    result.file_path
                )
            })?;
        if results.insert(artifact_path.clone(), result).is_some() {
            bail!("js-ts-pre-write-evidence: duplicate cached/extracted file {artifact_path}");
        }
    }
    if results.len() != identities.len() {
        bail!(
            "js-ts-pre-write-evidence: cache/extractor returned {} rows for {} files",
            results.len(),
            identities.len()
        );
    }

    if observations.changed_files == 0
        && observations.dropped_files == 0
        && observations.invalidated_files == 0
        && observations.load_status == "ok"
    {
        observations.write_status = "unchanged".to_string();
    } else {
        let entries = results
            .iter()
            .map(|(artifact_path, result)| {
                Ok((
                    artifact_path.as_str(),
                    CacheEntryRef {
                        identity: identities.get(artifact_path).with_context(|| {
                            format!(
                                "js-ts-pre-write-evidence: missing cache identity for {artifact_path}"
                            )
                        })?,
                        result,
                    },
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        let next = CacheFileRef {
            schema_version: CACHE_SCHEMA_VERSION,
            profile_version: CACHE_PROFILE_VERSION,
            root_fingerprint: &root_fingerprint,
            source_set_fingerprint: &source_set_fingerprint,
            scan_context_fingerprint: &scan_context_fingerprint,
            entries,
        };
        match write_cache(&cache_path, &next) {
            Ok(()) => observations.write_status = "written".to_string(),
            Err(error) => {
                observations.write_status = "failed".to_string();
                observations.reason = Some(format!("cache-write-failed: {error}"));
            }
        }
    }

    Ok((
        results.into_values().collect(),
        incremental_json(request, Some(&cache_path), observations),
    ))
}

fn extract_inputs(
    files: Vec<JsTsExtractInputFile>,
    source_files: Vec<String>,
) -> Result<Vec<JsTsExtractFileResult>> {
    Ok(build_js_ts_extract_response(JsTsExtractRequest {
        schema_version: JS_TS_EXTRACT_REQUEST_SCHEMA_VERSION.to_string(),
        files,
        source_files,
    })?
    .files)
}

fn input_without_source(file: JsTsPreWriteSourceFile) -> JsTsExtractInputFile {
    JsTsExtractInputFile {
        file_path: file.file_path.to_string_lossy().to_string(),
        artifact_file_path: Some(normalize_slashes(&file.artifact_file_path)),
        source: None,
    }
}

fn load_cache(path: &Path) -> (Option<CacheFile>, String) {
    if !path.exists() {
        return (None, "empty".to_string());
    }
    match fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<CacheFile>(&bytes).ok())
    {
        Some(cache) => (Some(cache), "ok".to_string()),
        None => (None, "ignored-malformed".to_string()),
    }
}

fn write_cache<T: Serialize + ?Sized>(path: &Path, cache: &T) -> Result<()> {
    let parent = path
        .parent()
        .context("js-ts-pre-write-evidence: cache path has no parent")?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!("{CACHE_FILE_NAME}.{}.tmp", std::process::id()));
    let file = File::create(&temp)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, cache)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    if let Err(error) = fs::rename(&temp, path) {
        let _ = fs::remove_file(&temp);
        return Err(error.into());
    }
    Ok(())
}

fn remove_cache_file(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn cache_path(cache_root: &Path, root_fingerprint: &str) -> PathBuf {
    cache_root
        .join("incremental")
        .join(root_fingerprint.trim_start_matches("sha256:"))
        .join("js-ts-pre-write-evidence")
        .join(CACHE_FILE_NAME)
}

fn root_fingerprint(root: &Path) -> Result<String> {
    let canonical = fs::canonicalize(root).with_context(|| {
        format!(
            "js-ts-pre-write-evidence: failed to canonicalize root {}",
            root.display()
        )
    })?;
    Ok(sha256_text(&format!(
        "js-ts-pre-write-root.v1\0{}",
        normalize_slashes(&canonical.to_string_lossy())
    )))
}

fn source_set_fingerprint(files: &[JsTsPreWriteSourceFile]) -> String {
    let mut text = String::from("js-ts-pre-write-source-set.v1\0");
    for file in files {
        text.push_str(&normalize_slashes(&file.artifact_file_path));
        text.push('\0');
    }
    sha256_text(&text)
}

fn scan_context_fingerprint(include_tests: bool, excludes: &[String]) -> String {
    let mut excludes = excludes.iter().map(String::as_str).collect::<Vec<_>>();
    excludes.sort_unstable();
    let mut text = format!("js-ts-pre-write-scan-context.v1\0include-tests={include_tests}\0");
    for exclude in excludes {
        text.push_str(exclude);
        text.push('\0');
    }
    sha256_text(&text)
}

fn clean_git_blob_ids(root: &Path) -> Option<BTreeMap<String, String>> {
    for invocation in git_invocations(root) {
        if let Some(blobs) = clean_git_blob_ids_with(root, &invocation) {
            return Some(blobs);
        }
    }
    None
}

struct GitInvocation {
    program: PathBuf,
    root_arg: String,
    expected_host_root: Option<String>,
}

fn git_invocations(root: &Path) -> Vec<GitInvocation> {
    let mut invocations = Vec::new();
    if let Some(host_root) = wsl_mount_to_windows_path(root) {
        invocations.push(GitInvocation {
            program: PathBuf::from("git.exe"),
            root_arg: host_root.clone(),
            expected_host_root: Some(host_root.clone()),
        });
        for program in [
            "/mnt/c/Program Files/Git/cmd/git.exe",
            "/mnt/c/Program Files/Git/bin/git.exe",
            "/mnt/c/Program Files (x86)/Git/cmd/git.exe",
        ] {
            let program = PathBuf::from(program);
            if program.is_file() {
                invocations.push(GitInvocation {
                    program,
                    root_arg: host_root.clone(),
                    expected_host_root: Some(host_root.clone()),
                });
            }
        }
    }
    invocations.push(GitInvocation {
        program: PathBuf::from("git"),
        root_arg: root.to_string_lossy().to_string(),
        expected_host_root: None,
    });
    invocations
}

fn clean_git_blob_ids_with(
    root: &Path,
    invocation: &GitInvocation,
) -> Option<BTreeMap<String, String>> {
    if !git_root_matches(root, invocation) {
        return None;
    }
    let status = git_output(
        invocation,
        [
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignored=no",
            "--",
        ],
    )?;
    let dirty = parse_dirty_paths(&status)?;
    let staged = git_output(invocation, ["ls-files", "-v", "--stage", "-z", "--"])?;
    parse_stage_zero_blobs(&staged, &dirty)
}

fn git_root_matches(root: &Path, invocation: &GitInvocation) -> bool {
    let Some(output) = git_output(invocation, ["rev-parse", "--show-toplevel"]) else {
        return false;
    };
    let Ok(text) = String::from_utf8(output) else {
        return false;
    };
    if let Some(expected) = invocation.expected_host_root.as_deref() {
        return normalized_host_path(text.trim()) == normalized_host_path(expected);
    }
    let Ok(git_root) = fs::canonicalize(text.trim()) else {
        return false;
    };
    let Ok(root) = fs::canonicalize(root) else {
        return false;
    };
    git_root == root
}

fn git_output<const N: usize>(invocation: &GitInvocation, args: [&str; N]) -> Option<Vec<u8>> {
    let output = Command::new(&invocation.program)
        .args([
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.untrackedCache=false",
        ])
        .arg("-C")
        .arg(&invocation.root_arg)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .ok()?;
    output.status.success().then_some(output.stdout)
}

fn wsl_mount_to_windows_path(root: &Path) -> Option<String> {
    let text = normalize_slashes(&root.to_string_lossy());
    let rest = text.strip_prefix("/mnt/")?;
    let mut chars = rest.chars();
    let drive = chars.next()?;
    if !drive.is_ascii_alphabetic() || chars.next()? != '/' {
        return None;
    }
    Some(format!(
        "{}:/{}",
        drive.to_ascii_uppercase(),
        chars.as_str()
    ))
}

fn normalized_host_path(value: &str) -> String {
    normalize_slashes(value)
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

fn parse_dirty_paths(bytes: &[u8]) -> Option<BTreeSet<String>> {
    let entries = bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<_>>();
    let mut dirty = BTreeSet::new();
    let mut index = 0;
    while index < entries.len() {
        let entry = entries[index];
        if entry.len() < 4 || entry[2] != b' ' {
            return None;
        }
        let status = &entry[..2];
        let path = std::str::from_utf8(&entry[3..]).ok()?;
        dirty.insert(normalize_slashes(path));
        if status.iter().any(|code| matches!(*code, b'R' | b'C')) {
            index += 1;
            let old_path = std::str::from_utf8(entries.get(index)?).ok()?;
            dirty.insert(normalize_slashes(old_path));
        }
        index += 1;
    }
    Some(dirty)
}

fn parse_stage_zero_blobs(
    bytes: &[u8],
    dirty: &BTreeSet<String>,
) -> Option<BTreeMap<String, String>> {
    let mut blobs = BTreeMap::new();
    for entry in bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        let tab = entry.iter().position(|byte| *byte == b'\t')?;
        let metadata = std::str::from_utf8(&entry[..tab]).ok()?;
        let mut fields = metadata.split_whitespace();
        let tag = fields.next()?;
        let mode = fields.next()?;
        let oid = fields.next()?;
        let stage = fields.next()?;
        if fields.next().is_some() || tag != "H" || stage != "0" || !mode.starts_with("100") {
            continue;
        }
        if !oid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let path = normalize_slashes(std::str::from_utf8(&entry[tab + 1..]).ok()?);
        if !dirty.contains(&path) {
            blobs.insert(path, format!("git:{oid}"));
        }
    }
    Some(blobs)
}

fn incremental_json(
    request: &JsTsPreWriteIncrementalRequest,
    cache_path: Option<&Path>,
    observations: CacheObservations,
) -> Value {
    let identity_mode = match (
        observations.git_blob_files > 0,
        observations.content_hash_files > 0,
    ) {
        (true, true) => json!("git-blob+sha256"),
        (true, false) => json!("git-blob"),
        (false, true) => json!("sha256"),
        (false, false) => Value::Null,
    };
    json!({
        "enabled": request.enabled,
        "identityMode": identity_mode,
        "cacheVersion": CACHE_SCHEMA_VERSION,
        "cacheProfileVersion": CACHE_PROFILE_VERSION,
        "cacheRoot": request.cache_root,
        "cacheFile": cache_path,
        "changedFiles": observations.changed_files,
        "reusedFiles": observations.reused_files,
        "droppedFiles": observations.dropped_files,
        "invalidatedFiles": observations.invalidated_files,
        "gitBlobFiles": observations.git_blob_files,
        "contentHashFiles": observations.content_hash_files,
        "loadStatus": observations.load_status,
        "writeStatus": observations.write_status,
        "reason": observations.reason,
    })
}

fn normalize_slashes(value: &str) -> String {
    value.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_stage_zero_git_blobs() -> Result<()> {
        let dirty = BTreeSet::from(["src/dirty.ts".to_string()]);
        let rows = b"H 100644 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 0\tsrc/clean.ts\0\
H 100644 bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb 0\tsrc/dirty.ts\0";
        let blobs = parse_stage_zero_blobs(rows, &dirty).context("valid rows")?;
        assert_eq!(
            blobs.get("src/clean.ts").map(String::as_str),
            Some("git:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
        assert!(!blobs.contains_key("src/dirty.ts"));
        Ok(())
    }

    #[test]
    fn excludes_skip_worktree_and_assume_unchanged_paths() -> Result<()> {
        let rows = b"H 100644 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 0\tsrc/clean.ts\0\
S 100644 bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb 0\tsrc/skip.ts\0\
h 100644 cccccccccccccccccccccccccccccccccccccccc 0\tsrc/assumed.ts\0";
        let blobs = parse_stage_zero_blobs(rows, &BTreeSet::new()).context("valid rows")?;
        assert_eq!(
            blobs.keys().cloned().collect::<Vec<_>>(),
            ["src/clean.ts".to_string()]
        );
        Ok(())
    }

    #[test]
    fn rename_status_marks_both_paths_dirty() -> Result<()> {
        let rows = b"R  src/new.ts\0src/old.ts\0?? src/untracked.ts\0";
        let dirty = parse_dirty_paths(rows).context("valid status")?;
        assert_eq!(
            dirty,
            BTreeSet::from([
                "src/new.ts".to_string(),
                "src/old.ts".to_string(),
                "src/untracked.ts".to_string(),
            ])
        );
        Ok(())
    }

    #[test]
    fn translates_wsl_drive_mounts_for_host_git() {
        assert_eq!(
            wsl_mount_to_windows_path(Path::new("/mnt/c/Users/name/repo")),
            Some("C:/Users/name/repo".to_string())
        );
        assert_eq!(
            wsl_mount_to_windows_path(Path::new("/home/name/repo")),
            None
        );
    }
}
