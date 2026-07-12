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
use std::time::Instant;

use super::single_flight::elapsed_ms;

const CACHE_SCHEMA_VERSION: u32 = 1;
const CACHE_PROFILE_VERSION: &str =
    "js-ts-pre-write-oxc-facts.v4+oxc-0.139.0+audit-core-bridge-v48";
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

#[derive(Default)]
struct CacheObservations {
    changed_files: usize,
    reused_files: usize,
    dropped_files: usize,
    invalidated_files: usize,
    content_hash_files: usize,
    load_status: String,
    write_status: String,
    reason: Option<String>,
    timing: CacheTiming,
}

#[derive(Default)]
struct CacheTiming {
    cache_load_ms: u64,
    source_read_hash_ms: u64,
    parse_ms: u64,
    cache_write_ms: u64,
    extraction_ms: u64,
}

pub(super) fn extract_with_cache(
    root: &Path,
    files: Vec<JsTsPreWriteSourceFile>,
    source_files: Vec<String>,
    include_tests: bool,
    excludes: &[String],
    request: &JsTsPreWriteIncrementalRequest,
) -> Result<(Vec<JsTsExtractFileResult>, Value)> {
    let extraction_started = Instant::now();
    if !request.enabled {
        let changed_files = files.len();
        let parse_started = Instant::now();
        let extracted = extract_inputs(
            files.into_iter().map(input_without_source).collect(),
            source_files,
        )?;
        let parse_ms = elapsed_ms(parse_started);
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
                    timing: CacheTiming {
                        parse_ms,
                        extraction_ms: elapsed_ms(extraction_started),
                        ..CacheTiming::default()
                    },
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

    let cache_load_started = Instant::now();
    let (mut prior, mut load_status) = load_cache(&cache_path);
    let cache_load_ms = elapsed_ms(cache_load_started);
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
    let mut observations = CacheObservations {
        load_status,
        write_status: "pending".to_string(),
        timing: CacheTiming {
            cache_load_ms,
            ..CacheTiming::default()
        },
        ..CacheObservations::default()
    };
    let current_paths = files
        .iter()
        .map(|file| normalize_slashes(&file.artifact_file_path))
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
    let source_read_hash_started = Instant::now();
    for file in files {
        let artifact_path = normalize_slashes(&file.artifact_file_path);
        let absolute_path = file.file_path.to_string_lossy().to_string();
        let bytes = fs::read(&file.file_path).with_context(|| {
            format!("js-ts-pre-write-evidence: failed to read required source {artifact_path}")
        })?;
        let identity = FileIdentity {
            mode: "sha256".to_string(),
            value: sha256_bytes(&bytes),
        };
        observations.content_hash_files += 1;
        let prior_entry = prior_entries.remove(&artifact_path);
        let had_prior_entry = prior_entry.is_some();
        let reusable = prior_entry.filter(|entry| {
            entry.identity == identity
                && normalize_slashes(&entry.result.file_path) == normalize_slashes(&absolute_path)
        });
        absolute_to_artifact.insert(normalize_slashes(&absolute_path), artifact_path.clone());
        if let Some(entry) = reusable {
            observations.reused_files += 1;
            identities.insert(artifact_path.clone(), identity);
            results.insert(artifact_path, entry.result);
            continue;
        }
        observations.changed_files += 1;
        if compatible && had_prior_entry {
            observations.invalidated_files += 1;
        }
        identities.insert(artifact_path.clone(), identity);
        let source = String::from_utf8(bytes).with_context(|| {
            format!("js-ts-pre-write-evidence: required source is not UTF-8: {artifact_path}")
        })?;
        pending.push(JsTsExtractInputFile {
            file_path: absolute_path,
            artifact_file_path: Some(artifact_path),
            source: Some(source),
        });
    }
    observations.timing.source_read_hash_ms = elapsed_ms(source_read_hash_started);

    let parse_started = Instant::now();
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
    observations.timing.parse_ms = elapsed_ms(parse_started);
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
        let cache_write_started = Instant::now();
        match write_cache(&cache_path, &next) {
            Ok(()) => observations.write_status = "written".to_string(),
            Err(error) => {
                observations.write_status = "failed".to_string();
                observations.reason = Some(format!("cache-write-failed: {error}"));
            }
        }
        observations.timing.cache_write_ms = elapsed_ms(cache_write_started);
    }
    observations.timing.extraction_ms = elapsed_ms(extraction_started);

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

fn incremental_json(
    request: &JsTsPreWriteIncrementalRequest,
    cache_path: Option<&Path>,
    observations: CacheObservations,
) -> Value {
    let identity_mode = if observations.content_hash_files > 0 {
        json!("sha256")
    } else {
        Value::Null
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
        "gitBlobFiles": 0,
        "contentHashFiles": observations.content_hash_files,
        "loadStatus": observations.load_status,
        "writeStatus": observations.write_status,
        "reason": observations.reason,
        "timing": {
            "cacheLoadMs": observations.timing.cache_load_ms,
            "sourceReadHashMs": observations.timing.source_read_hash_ms,
            "parseMs": observations.timing.parse_ms,
            "cacheWriteMs": observations.timing.cache_write_ms,
            "extractionMs": observations.timing.extraction_ms,
        },
    })
}

fn normalize_slashes(value: &str) -> String {
    value.replace('\\', "/")
}
