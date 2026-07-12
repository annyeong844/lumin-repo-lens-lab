use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use lumin_rust_common::{atomic_write_json, sha256_text};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::analyzer::{
    analyze_source_file_entries_compact, CompactDeadFile, CompactFileAnalysis, CompactSummaryFile,
    SourceFileEntry,
};
use crate::function_clones::FunctionCloneFile;
use crate::protocol::{
    IncrementalMeta, ParserRequest, PARSER_KIND, PARSER_VERSION, POLICY_VERSION, SCHEMA_VERSION,
    SIGNAL_POLICY_ID, SIGNAL_POLICY_VERSION,
};

const CACHE_STORE_SCHEMA_VERSION: u32 = 1;
const CACHE_SHARD_SCHEMA_VERSION: u32 = 1;
const PRODUCER_ID: &str = "rust-source-health-compact";
const PRODUCER_VERSION: u32 = 6;
const FACT_SCHEMA_VERSION: u32 = 1;
const IDENTITY_MODE: &str = "strict-content-hash";

#[derive(Clone)]
pub(crate) struct CompactCacheOptions {
    pub(crate) root: PathBuf,
    pub(crate) cache_root: Option<PathBuf>,
    pub(crate) incremental_enabled: bool,
    pub(crate) clear_incremental_cache: bool,
}

pub(crate) struct CompactCacheRun {
    pub(crate) files: BTreeMap<String, CompactFileAnalysis>,
    pub(crate) incremental: IncrementalMeta,
}

pub(crate) struct PreparedCompactCache {
    store: CacheStore,
    producer_meta: CompactCacheProducerMeta,
    pub(crate) incremental: IncrementalMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompactCacheEntry<T> {
    schema_version: u32,
    key: String,
    identity: CompactCacheIdentity,
    producer_meta: CompactCacheProducerMeta,
    payload: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompactCacheShard<T> {
    schema_version: u32,
    entries: BTreeMap<String, CompactCacheEntry<T>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompactCacheShardMeta {
    schema_version: u32,
    entries: BTreeMap<String, CompactCacheEntryMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompactCacheEntryMeta {
    schema_version: u32,
    key: String,
    identity: CompactCacheIdentity,
    producer_meta: CompactCacheProducerMeta,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompactCacheIdentity {
    rel_path: String,
    language: String,
    content_hash: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompactCacheProducerMeta {
    producer_id: String,
    producer_version: u32,
    fact_schema_version: u32,
    parser_identity: String,
    policy_identity: String,
    artifact_profile: String,
}

struct CacheStore {
    cache_root: PathBuf,
    repo_cache_dir: PathBuf,
    producer_dir: PathBuf,
    summary_dir: PathBuf,
    clone_dir: PathBuf,
    dead_dir: PathBuf,
}

struct ExistingCacheFiles {
    shards: BTreeSet<String>,
    legacy_entries: BTreeSet<String>,
}

#[derive(Debug)]
struct LoadedShard<T> {
    malformed: bool,
    entries: BTreeMap<String, CompactCacheEntry<T>>,
}

#[derive(Debug)]
struct LoadedShardMeta {
    malformed: bool,
    entries: BTreeMap<String, CompactCacheEntryMeta>,
}

struct LoadedLaneMetas {
    summary: LoadedShardMeta,
    clone: LoadedShardMeta,
    dead: LoadedShardMeta,
}

#[derive(Debug, Clone)]
struct FileCachePlan {
    file: SourceFileEntry,
    key: String,
    entry_id: String,
}

pub(crate) fn analyze_compact_with_cache(
    files: &[SourceFileEntry],
    parser: &ParserRequest,
    options: CompactCacheOptions,
) -> Result<CompactCacheRun> {
    let store = open_cache_store(&options.root, options.cache_root.as_deref());
    if !options.incremental_enabled {
        let analyzed = analyze_source_file_entries_compact(files, parser)?;
        return Ok(CompactCacheRun {
            files: analyzed,
            incremental: disabled_incremental_meta(files.len(), &store),
        });
    }

    let prepared = prepare_compact_cache(files, parser, options)?;
    let mut summary_files = load_compact_summary_files_from_cache(files, &prepared)?;
    let mut clone_files = load_compact_clone_files_from_cache(files, &prepared)?;
    let mut dead_files = load_compact_dead_files_from_cache(files, &prepared)?;
    let mut files_by_path = BTreeMap::new();
    for planned in files {
        let summary_file = summary_files.remove(&planned.path).with_context(|| {
            format!(
                "missing Rust source-health summary cache entry for {}",
                planned.path
            )
        })?;
        let clone_file = clone_files.remove(&planned.path).with_context(|| {
            format!(
                "missing Rust source-health clone cache entry for {}",
                planned.path
            )
        })?;
        let dead_file = dead_files.remove(&planned.path).with_context(|| {
            format!(
                "missing Rust source-health dead cache entry for {}",
                planned.path
            )
        })?;
        files_by_path.insert(
            planned.path.clone(),
            CompactFileAnalysis {
                summary_file,
                clone_file,
                dead_file,
            },
        );
    }
    Ok(CompactCacheRun {
        files: files_by_path,
        incremental: prepared.incremental,
    })
}

pub(crate) fn prepare_compact_cache(
    files: &[SourceFileEntry],
    parser: &ParserRequest,
    options: CompactCacheOptions,
) -> Result<PreparedCompactCache> {
    let store = open_cache_store(&options.root, options.cache_root.as_deref());
    if !options.incremental_enabled {
        return Ok(PreparedCompactCache {
            producer_meta: current_producer_meta(parser),
            incremental: disabled_incremental_meta(files.len(), &store),
            store,
        });
    }

    if options.clear_incremental_cache {
        fs::remove_dir_all(&store.repo_cache_dir).ok();
    }

    let producer_meta = current_producer_meta(parser);
    let existing_cache_files = existing_cache_files(&store)?;
    let mut dropped_files = remove_legacy_entry_files(&store, &existing_cache_files.legacy_entries);
    let load_status = if existing_cache_files.shards.is_empty()
        && existing_cache_files.legacy_entries.is_empty()
    {
        "empty"
    } else {
        "ok"
    };
    let files_by_shard = files_by_cache_shard(files);
    let current_shards = files_by_shard.keys().cloned().collect::<BTreeSet<_>>();
    let mut misses = Vec::new();
    let mut invalidated_files = 0usize;
    let mut malformed_shard_seen = false;
    let mut affected_shards = BTreeSet::new();

    for (shard_file, shard_files) in &files_by_shard {
        let metas = load_lane_metas(&store, shard_file)?;
        malformed_shard_seen |= metas.malformed();
        let current_entry_ids = shard_files
            .iter()
            .map(|planned| planned.entry_id.clone())
            .collect::<BTreeSet<_>>();

        for planned in shard_files {
            if metas.malformed() {
                invalidated_files += 1;
                misses.push(planned.file.clone());
                affected_shards.insert(shard_file.clone());
                continue;
            }

            match reusable_lane_metas(&metas, planned, &producer_meta) {
                CacheMetaLookup::Hit => {}
                CacheMetaLookup::Miss { invalidated } => {
                    invalidated_files += usize::from(invalidated);
                    misses.push(planned.file.clone());
                    affected_shards.insert(shard_file.clone());
                }
            }
        }

        if !metas.malformed() {
            let dropped_in_shard = prior_entry_ids(&metas)
                .difference(&current_entry_ids)
                .count();
            if dropped_in_shard > 0 {
                dropped_files += dropped_in_shard;
                affected_shards.insert(shard_file.clone());
            }
        }
    }

    for stale_shard in existing_cache_files.shards.difference(&current_shards) {
        let metas = load_lane_metas(&store, stale_shard)?;
        malformed_shard_seen |= metas.malformed();
        if !metas.malformed() {
            dropped_files += prior_entry_ids(&metas).len();
        }
        remove_lane_shard_files(&store, stale_shard);
    }

    let parsed = analyze_source_file_entries_compact(&misses, parser)?;
    for shard_file in &affected_shards {
        let Some(shard_files) = files_by_shard.get(shard_file) else {
            continue;
        };
        rewrite_lane_shards(&store, shard_file, shard_files, &parsed, &producer_meta)?;
    }

    let changed_files = misses.len();
    let reused_files = files.len().saturating_sub(changed_files);
    let load_status = if malformed_shard_seen {
        "ignored-malformed"
    } else {
        load_status
    };

    Ok(PreparedCompactCache {
        incremental: IncrementalMeta {
            enabled: true,
            reason: None,
            identity_mode: Some(IDENTITY_MODE.to_string()),
            cache_root: Some(store.cache_root.display().to_string()),
            cache_file: Some(store.producer_dir.display().to_string()),
            load_status: Some(load_status.to_string()),
            changed_files,
            reused_files,
            dropped_files,
            invalidated_files,
        },
        producer_meta,
        store,
    })
}

pub(crate) fn load_compact_summary_files_from_cache(
    files: &[SourceFileEntry],
    prepared: &PreparedCompactCache,
) -> Result<BTreeMap<String, CompactSummaryFile>> {
    let mut summary_files = BTreeMap::new();
    for (shard_file, shard_files) in files_by_cache_shard(files) {
        let mut loaded = load_required_lane_shard::<CompactSummaryFile>(
            &prepared.store.summary_dir,
            &shard_file,
            "summary",
        )?;
        for planned in shard_files {
            let entry = loaded.entries.remove(&planned.entry_id).with_context(|| {
                format!(
                    "missing Rust source-health summary cache entry for {}",
                    planned.file.path
                )
            })?;
            ensure_cache_entry_matches(&entry, &planned, &prepared.producer_meta)?;
            summary_files.insert(planned.file.path, entry.payload);
        }
    }
    Ok(summary_files)
}

pub(crate) fn load_compact_clone_files_from_cache(
    files: &[SourceFileEntry],
    prepared: &PreparedCompactCache,
) -> Result<BTreeMap<String, FunctionCloneFile>> {
    let mut clone_files = BTreeMap::new();
    for (shard_file, shard_files) in files_by_cache_shard(files) {
        let mut loaded = load_required_lane_shard::<FunctionCloneFile>(
            &prepared.store.clone_dir,
            &shard_file,
            "clone",
        )?;
        for planned in shard_files {
            let entry = loaded.entries.remove(&planned.entry_id).with_context(|| {
                format!(
                    "missing Rust source-health clone cache entry for {}",
                    planned.file.path
                )
            })?;
            ensure_cache_entry_matches(&entry, &planned, &prepared.producer_meta)?;
            clone_files.insert(planned.file.path, entry.payload);
        }
    }
    Ok(clone_files)
}

pub(crate) fn stream_compact_clone_files_from_cache<F>(
    files: &[SourceFileEntry],
    prepared: &PreparedCompactCache,
    mut consume: F,
) -> Result<()>
where
    F: FnMut(String, FunctionCloneFile) -> Result<()>,
{
    for (shard_file, shard_files) in files_by_cache_shard(files) {
        let mut loaded = load_required_lane_shard::<FunctionCloneFile>(
            &prepared.store.clone_dir,
            &shard_file,
            "clone",
        )?;
        for planned in shard_files {
            let entry = loaded.entries.remove(&planned.entry_id).with_context(|| {
                format!(
                    "missing Rust source-health clone cache entry for {}",
                    planned.file.path
                )
            })?;
            ensure_cache_entry_matches(&entry, &planned, &prepared.producer_meta)?;
            consume(planned.file.path, entry.payload)?;
        }
    }
    Ok(())
}

pub(crate) fn load_compact_dead_files_from_cache(
    files: &[SourceFileEntry],
    prepared: &PreparedCompactCache,
) -> Result<BTreeMap<String, CompactDeadFile>> {
    let mut dead_files = BTreeMap::new();
    for (shard_file, shard_files) in files_by_cache_shard(files) {
        let mut loaded = load_required_lane_shard::<CompactDeadFile>(
            &prepared.store.dead_dir,
            &shard_file,
            "dead",
        )?;
        for planned in shard_files {
            let entry = loaded.entries.remove(&planned.entry_id).with_context(|| {
                format!(
                    "missing Rust source-health dead cache entry for {}",
                    planned.file.path
                )
            })?;
            ensure_cache_entry_matches(&entry, &planned, &prepared.producer_meta)?;
            dead_files.insert(planned.file.path, entry.payload);
        }
    }
    Ok(dead_files)
}

enum CacheMetaLookup {
    Hit,
    Miss { invalidated: bool },
}

impl LoadedLaneMetas {
    fn malformed(&self) -> bool {
        self.summary.malformed || self.clone.malformed || self.dead.malformed
    }
}

fn reusable_lane_metas(
    metas: &LoadedLaneMetas,
    planned: &FileCachePlan,
    producer_meta: &CompactCacheProducerMeta,
) -> CacheMetaLookup {
    let summary = reusable_meta(&metas.summary, planned, producer_meta);
    let clone = reusable_meta(&metas.clone, planned, producer_meta);
    let dead = reusable_meta(&metas.dead, planned, producer_meta);
    if matches!(summary, CacheMetaLookup::Hit)
        && matches!(clone, CacheMetaLookup::Hit)
        && matches!(dead, CacheMetaLookup::Hit)
    {
        return CacheMetaLookup::Hit;
    }

    let present_in_any_lane = metas.summary.entries.contains_key(&planned.entry_id)
        || metas.clone.entries.contains_key(&planned.entry_id)
        || metas.dead.entries.contains_key(&planned.entry_id);
    let invalidated = present_in_any_lane
        || matches!(summary, CacheMetaLookup::Miss { invalidated: true })
        || matches!(clone, CacheMetaLookup::Miss { invalidated: true })
        || matches!(dead, CacheMetaLookup::Miss { invalidated: true });
    CacheMetaLookup::Miss { invalidated }
}

fn reusable_meta(
    loaded: &LoadedShardMeta,
    planned: &FileCachePlan,
    producer_meta: &CompactCacheProducerMeta,
) -> CacheMetaLookup {
    let Some(prior) = loaded.entries.get(&planned.entry_id) else {
        return CacheMetaLookup::Miss { invalidated: false };
    };
    if prior.schema_version != CACHE_STORE_SCHEMA_VERSION || prior.key != planned.key {
        return CacheMetaLookup::Miss { invalidated: true };
    }
    let current_identity = cache_identity(&planned.file);
    if prior.producer_meta != *producer_meta || prior.identity != current_identity {
        return CacheMetaLookup::Miss { invalidated: true };
    }
    CacheMetaLookup::Hit
}

trait CacheEntryMetaView {
    fn schema_version(&self) -> u32;
    fn key(&self) -> &str;
    fn identity(&self) -> &CompactCacheIdentity;
    fn producer_meta(&self) -> &CompactCacheProducerMeta;
}

impl<T> CacheEntryMetaView for CompactCacheEntry<T> {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn key(&self) -> &str {
        &self.key
    }

    fn identity(&self) -> &CompactCacheIdentity {
        &self.identity
    }

    fn producer_meta(&self) -> &CompactCacheProducerMeta {
        &self.producer_meta
    }
}

fn ensure_cache_entry_matches(
    entry: &impl CacheEntryMetaView,
    planned: &FileCachePlan,
    producer_meta: &CompactCacheProducerMeta,
) -> Result<()> {
    if entry.schema_version() != CACHE_STORE_SCHEMA_VERSION
        || entry.key() != planned.key
        || entry.identity() != &cache_identity(&planned.file)
        || entry.producer_meta() != producer_meta
    {
        anyhow::bail!(
            "stale Rust source-health compact cache entry for {}",
            planned.file.path
        );
    }
    Ok(())
}

fn disabled_incremental_meta(file_count: usize, store: &CacheStore) -> IncrementalMeta {
    IncrementalMeta {
        enabled: false,
        reason: Some("disabled-by-flag".to_string()),
        identity_mode: Some(IDENTITY_MODE.to_string()),
        cache_root: Some(store.cache_root.display().to_string()),
        cache_file: Some(store.producer_dir.display().to_string()),
        load_status: None,
        changed_files: file_count,
        reused_files: 0,
        dropped_files: 0,
        invalidated_files: 0,
    }
}

fn rewrite_lane_shards(
    store: &CacheStore,
    shard_file: &str,
    shard_files: &[FileCachePlan],
    parsed: &BTreeMap<String, CompactFileAnalysis>,
    producer_meta: &CompactCacheProducerMeta,
) -> Result<()> {
    let summary_loaded = load_lane_shard::<CompactSummaryFile>(&store.summary_dir, shard_file)?;
    let clone_loaded = load_lane_shard::<FunctionCloneFile>(&store.clone_dir, shard_file)?;
    let dead_loaded = load_lane_shard::<CompactDeadFile>(&store.dead_dir, shard_file)?;

    let mut summary_entries = BTreeMap::new();
    let mut clone_entries = BTreeMap::new();
    let mut dead_entries = BTreeMap::new();

    for planned in shard_files {
        let summary_payload = lane_payload(
            parsed
                .get(&planned.file.path)
                .map(|payload| &payload.summary_file),
            &summary_loaded,
            planned,
            "summary",
        )?;
        let clone_payload = lane_payload(
            parsed
                .get(&planned.file.path)
                .map(|payload| &payload.clone_file),
            &clone_loaded,
            planned,
            "clone",
        )?;
        let dead_payload = lane_payload(
            parsed
                .get(&planned.file.path)
                .map(|payload| &payload.dead_file),
            &dead_loaded,
            planned,
            "dead",
        )?;

        summary_entries.insert(
            planned.entry_id.clone(),
            cache_entry(planned, producer_meta, summary_payload),
        );
        clone_entries.insert(
            planned.entry_id.clone(),
            cache_entry(planned, producer_meta, clone_payload),
        );
        dead_entries.insert(
            planned.entry_id.clone(),
            cache_entry(planned, producer_meta, dead_payload),
        );
    }

    write_lane_shard(&store.summary_dir, shard_file, summary_entries, "summary")?;
    write_lane_shard(&store.clone_dir, shard_file, clone_entries, "clone")?;
    write_lane_shard(&store.dead_dir, shard_file, dead_entries, "dead")?;
    Ok(())
}

fn lane_payload<T: Clone>(
    parsed_payload: Option<&T>,
    loaded: &LoadedShard<T>,
    planned: &FileCachePlan,
    lane: &str,
) -> Result<T> {
    if let Some(payload) = parsed_payload {
        return Ok(payload.clone());
    }
    if loaded.malformed {
        anyhow::bail!(
            "missing parsed Rust source-health {lane} cache replacement for {}",
            planned.file.path
        );
    }
    loaded
        .entries
        .get(&planned.entry_id)
        .map(|entry| entry.payload.clone())
        .with_context(|| {
            format!(
                "missing reusable Rust source-health {lane} cache entry for {}",
                planned.file.path
            )
        })
}

fn cache_entry<T>(
    planned: &FileCachePlan,
    producer_meta: &CompactCacheProducerMeta,
    payload: T,
) -> CompactCacheEntry<T> {
    CompactCacheEntry {
        schema_version: CACHE_STORE_SCHEMA_VERSION,
        key: planned.key.clone(),
        identity: cache_identity(&planned.file),
        producer_meta: producer_meta.clone(),
        payload,
    }
}

fn write_lane_shard<T: Serialize>(
    lane_dir: &Path,
    shard_file: &str,
    entries: BTreeMap<String, CompactCacheEntry<T>>,
    lane: &str,
) -> Result<()> {
    let path = lane_dir.join(shard_file);
    let shard = CompactCacheShard {
        schema_version: CACHE_SHARD_SCHEMA_VERSION,
        entries,
    };
    atomic_write_json(&path, &shard).with_context(|| {
        format!(
            "failed to write Rust source-health {lane} cache shard {}",
            path.display()
        )
    })
}

fn load_lane_metas(store: &CacheStore, shard_file: &str) -> Result<LoadedLaneMetas> {
    Ok(LoadedLaneMetas {
        summary: load_shard_meta(&store.summary_dir, shard_file)?,
        clone: load_shard_meta(&store.clone_dir, shard_file)?,
        dead: load_shard_meta(&store.dead_dir, shard_file)?,
    })
}

fn load_shard_meta(lane_dir: &Path, shard_file: &str) -> Result<LoadedShardMeta> {
    let path = lane_dir.join(shard_file);
    let Ok(file) = fs::File::open(&path) else {
        return Ok(LoadedShardMeta {
            malformed: false,
            entries: BTreeMap::new(),
        });
    };
    let Ok(shard) = serde_json::from_reader::<_, CompactCacheShardMeta>(BufReader::new(file))
    else {
        return Ok(LoadedShardMeta {
            malformed: true,
            entries: BTreeMap::new(),
        });
    };
    if shard.schema_version != CACHE_SHARD_SCHEMA_VERSION {
        return Ok(LoadedShardMeta {
            malformed: true,
            entries: BTreeMap::new(),
        });
    }
    Ok(LoadedShardMeta {
        malformed: false,
        entries: shard.entries,
    })
}

fn load_lane_shard<T: DeserializeOwned>(
    lane_dir: &Path,
    shard_file: &str,
) -> Result<LoadedShard<T>> {
    let path = lane_dir.join(shard_file);
    let Ok(file) = fs::File::open(&path) else {
        return Ok(LoadedShard {
            malformed: false,
            entries: BTreeMap::new(),
        });
    };
    let Ok(shard) = serde_json::from_reader::<_, CompactCacheShard<T>>(BufReader::new(file)) else {
        return Ok(LoadedShard {
            malformed: true,
            entries: BTreeMap::new(),
        });
    };
    if shard.schema_version != CACHE_SHARD_SCHEMA_VERSION {
        return Ok(LoadedShard {
            malformed: true,
            entries: BTreeMap::new(),
        });
    }
    Ok(LoadedShard {
        malformed: false,
        entries: shard.entries,
    })
}

fn load_required_lane_shard<T: DeserializeOwned>(
    lane_dir: &Path,
    shard_file: &str,
    lane: &str,
) -> Result<LoadedShard<T>> {
    let path = lane_dir.join(shard_file);
    let file = fs::File::open(&path).with_context(|| {
        format!(
            "failed to read Rust source-health {lane} cache shard {}",
            path.display()
        )
    })?;
    let shard = serde_json::from_reader::<_, CompactCacheShard<T>>(BufReader::new(file))
        .with_context(|| {
            format!(
                "failed to parse Rust source-health {lane} cache shard {}",
                path.display()
            )
        })?;
    if shard.schema_version != CACHE_SHARD_SCHEMA_VERSION {
        anyhow::bail!(
            "unsupported Rust source-health {lane} cache shard schema {} in {}",
            shard.schema_version,
            path.display()
        );
    }
    Ok(LoadedShard {
        malformed: false,
        entries: shard.entries,
    })
}

fn prior_entry_ids(metas: &LoadedLaneMetas) -> BTreeSet<String> {
    metas
        .summary
        .entries
        .keys()
        .chain(metas.clone.entries.keys())
        .chain(metas.dead.entries.keys())
        .cloned()
        .collect()
}

fn files_by_cache_shard(files: &[SourceFileEntry]) -> BTreeMap<String, Vec<FileCachePlan>> {
    let mut by_shard = BTreeMap::<String, Vec<FileCachePlan>>::new();
    for file in files {
        let key = strict_cache_key(file);
        let entry_id = cache_entry_id(&key);
        let shard_file = cache_shard_file_name(&entry_id);
        by_shard.entry(shard_file).or_default().push(FileCachePlan {
            file: file.clone(),
            key,
            entry_id,
        });
    }
    by_shard
}

fn cache_identity(file: &SourceFileEntry) -> CompactCacheIdentity {
    CompactCacheIdentity {
        rel_path: file.path.clone(),
        language: "rust".to_string(),
        content_hash: file.sha256.clone(),
    }
}

fn strict_cache_key(file: &SourceFileEntry) -> String {
    format!("{}|rust|compact", file.path)
}

fn cache_entry_id(key: &str) -> String {
    sha256_text(key).trim_start_matches("sha256:").to_string()
}

fn cache_shard_file_name(entry_id: &str) -> String {
    let shard = entry_id.chars().next().unwrap_or('0');
    format!("{shard}.json")
}

fn existing_cache_files(store: &CacheStore) -> Result<ExistingCacheFiles> {
    let mut shards = BTreeSet::new();
    let mut legacy_entries = BTreeSet::new();
    collect_lane_shard_names(&store.summary_dir, &mut shards)?;
    collect_lane_shard_names(&store.clone_dir, &mut shards)?;
    collect_lane_shard_names(&store.dead_dir, &mut shards)?;
    collect_legacy_root_json_files(store, &mut legacy_entries)?;
    Ok(ExistingCacheFiles {
        shards,
        legacy_entries,
    })
}

fn collect_lane_shard_names(lane_dir: &Path, shards: &mut BTreeSet<String>) -> Result<()> {
    let Ok(read_dir) = fs::read_dir(lane_dir) else {
        return Ok(());
    };
    for entry in read_dir {
        let entry = entry.with_context(|| {
            format!(
                "failed to read Rust source-health cache lane directory {}",
                lane_dir.display()
            )
        })?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if is_shard_file_name(&name) {
            shards.insert(name);
        }
    }
    Ok(())
}

fn collect_legacy_root_json_files(
    store: &CacheStore,
    legacy_entries: &mut BTreeSet<String>,
) -> Result<()> {
    let Ok(read_dir) = fs::read_dir(&store.producer_dir) else {
        return Ok(());
    };
    for entry in read_dir {
        let entry = entry.with_context(|| {
            format!(
                "failed to read Rust source-health cache directory {}",
                store.producer_dir.display()
            )
        })?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if name.ends_with(".json") {
            legacy_entries.insert(name);
        }
    }
    Ok(())
}

fn remove_legacy_entry_files(store: &CacheStore, legacy_entries: &BTreeSet<String>) -> usize {
    for entry in legacy_entries {
        fs::remove_file(store.producer_dir.join(entry)).ok();
    }
    legacy_entries.len()
}

fn remove_lane_shard_files(store: &CacheStore, shard_file: &str) {
    fs::remove_file(store.summary_dir.join(shard_file)).ok();
    fs::remove_file(store.clone_dir.join(shard_file)).ok();
    fs::remove_file(store.dead_dir.join(shard_file)).ok();
}

fn is_shard_file_name(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".json") else {
        return false;
    };
    stem.len() == 1 && stem.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn current_producer_meta(parser: &ParserRequest) -> CompactCacheProducerMeta {
    CompactCacheProducerMeta {
        producer_id: PRODUCER_ID.to_string(),
        producer_version: PRODUCER_VERSION,
        fact_schema_version: FACT_SCHEMA_VERSION,
        parser_identity: format!(
            "{:?}:{}:{:?}:{:?}:{:?}",
            PARSER_KIND,
            PARSER_VERSION,
            parser.edition_policy,
            parser.edition,
            parser.edition_source
        ),
        policy_identity: format!(
            "schema={SCHEMA_VERSION};policy={POLICY_VERSION};signal={SIGNAL_POLICY_ID}:{SIGNAL_POLICY_VERSION}"
        ),
        artifact_profile: "compact".to_string(),
    }
}

fn open_cache_store(root: &Path, cache_root: Option<&Path>) -> CacheStore {
    let cache_root = cache_root
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.join(".audit").join(".cache"));
    let repo_fingerprint = repo_fingerprint(root);
    let repo_hash = repo_fingerprint.trim_start_matches("sha256:");
    let repo_cache_dir = cache_root.join("incremental").join(repo_hash);
    let producer_dir = repo_cache_dir.join(PRODUCER_ID);
    let summary_dir = producer_dir.join("summary");
    let clone_dir = producer_dir.join("clone");
    let dead_dir = producer_dir.join("dead");
    CacheStore {
        cache_root,
        repo_cache_dir,
        producer_dir,
        summary_dir,
        clone_dir,
        dead_dir,
    }
}

fn repo_fingerprint(root: &Path) -> String {
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let marker = if canonical.join(".git").exists() {
        "git-worktree"
    } else if canonical.join("Cargo.toml").exists() {
        "cargo-root"
    } else {
        "directory-root"
    };
    sha256_text(&format!(
        "rust-source-health-cache-v1|{}|{}",
        canonical.display(),
        marker
    ))
}
