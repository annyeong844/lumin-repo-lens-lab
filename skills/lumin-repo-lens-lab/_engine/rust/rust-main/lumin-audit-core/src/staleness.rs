use anyhow::{bail, Result};
use lumin_rust_common::sha256_text;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub const STALENESS_REQUEST_SCHEMA_VERSION: &str = "lumin-staleness-producer-request.v1";
const STALENESS_CACHE_SCHEMA_VERSION: i64 = 1;
const TOOL_NAME: &str = "measure-staleness.mjs";
const MIN_PICKAXE_SYMBOL_LEN: usize = 4;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StalenessRequest {
    pub schema_version: String,
    pub root: String,
    #[serde(default)]
    pub generated: Option<String>,
    #[serde(default)]
    pub symbols_source: Option<String>,
    #[serde(default)]
    pub symbols: Value,
    #[serde(default = "default_max_age_days")]
    pub max_age_days: i64,
    #[serde(default = "default_stale_age_days")]
    pub stale_age_days: i64,
    #[serde(default = "default_since")]
    pub since: String,
    #[serde(default)]
    pub skip_pickaxe: bool,
    #[serde(default = "default_true")]
    pub incremental_enabled: bool,
    #[serde(default)]
    pub cache_root: Option<String>,
    #[serde(default)]
    pub clear_incremental_cache: bool,
}

#[derive(Debug, Clone)]
struct Mention {
    status: &'static str,
    ts: Option<i64>,
}

#[derive(Debug, Clone)]
struct Grounding {
    grounding: &'static str,
    confidence: &'static str,
    note: &'static str,
}

#[derive(Debug, Default, Clone)]
struct PerformanceCounters {
    dead_candidates_processed: i64,
    file_touch_git_calls: i64,
    line_blame_git_calls: i64,
    line_blame_cache_hits: i64,
    line_blame_cache_misses: i64,
    symbol_pickaxe_git_calls: i64,
}

impl PerformanceCounters {
    fn json(&self) -> Value {
        json!({
            "deadCandidatesProcessed": self.dead_candidates_processed,
            "fileTouchGitCalls": self.file_touch_git_calls,
            "lineBlameGitCalls": self.line_blame_git_calls,
            "lineBlameCacheHits": self.line_blame_cache_hits,
            "lineBlameCacheMisses": self.line_blame_cache_misses,
            "symbolPickaxeGitCalls": self.symbol_pickaxe_git_calls,
        })
    }
}

struct StalenessContext {
    root: PathBuf,
    max_age_days: i64,
    stale_age_days: i64,
    since: String,
    skip_pickaxe: bool,
    now: i64,
    performance: PerformanceCounters,
    dead_candidates_by_file: HashMap<String, usize>,
    file_touch_cache: HashMap<String, Option<i64>>,
    line_blame_cache: HashMap<String, HashMap<i64, i64>>,
    symbol_mention_cache: HashMap<String, Mention>,
}

pub fn build_staleness_artifact(request: StalenessRequest) -> Result<Value> {
    if request.schema_version != STALENESS_REQUEST_SCHEMA_VERSION {
        bail!(
            "staleness-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if !request.symbols.is_object() {
        bail!("staleness-artifact: symbols must be an object");
    }
    if request.max_age_days < 0 {
        bail!("staleness-artifact: maxAgeDays must be non-negative");
    }
    if request.stale_age_days < 0 {
        bail!("staleness-artifact: staleAgeDays must be non-negative");
    }

    let root = PathBuf::from(&request.root);
    ensure_git_repo(&root)?;
    let git_head = git(&root, &["rev-parse", "HEAD"]).trim().to_string();
    let git_head = if git_head.is_empty() {
        Value::Null
    } else {
        Value::String(git_head)
    };
    let symbols_source = request
        .symbols_source
        .clone()
        .unwrap_or_else(|| "symbols.json".to_string());
    let generated = request
        .generated
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let dead_list = request
        .symbols
        .get("deadProdList")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let cache = cache_context(
        &request,
        &root,
        git_head.as_str().unwrap_or("unknown"),
        &dead_list,
    );
    if request.clear_incremental_cache {
        let _ = fs::remove_dir_all(&cache.repo_cache_dir);
    }
    let cache_key = request
        .incremental_enabled
        .then(|| staleness_cache_key(&request, git_head.as_str().unwrap_or("unknown"), &dead_list));

    if request.incremental_enabled {
        if let Some(cache_key) = cache_key.as_deref() {
            if let Some(cached) = load_cached_artifact(&cache.cache_path, cache_key) {
                return Ok(refresh_cached_artifact(
                    cached,
                    &generated,
                    &request.root,
                    git_head,
                    &symbols_source,
                    incremental_meta(&request, &cache, true, "cache-hit", Some(cache_key)),
                ));
            }
        }
    }

    let mut dead_candidates_by_file = HashMap::new();
    for dead in &dead_list {
        if let Some(file) = string_field(dead, "file") {
            *dead_candidates_by_file.entry(file).or_insert(0) += 1;
        }
    }

    let mut ctx = StalenessContext {
        root,
        max_age_days: request.max_age_days,
        stale_age_days: request.stale_age_days,
        since: request.since.clone(),
        skip_pickaxe: request.skip_pickaxe,
        now: unix_now(),
        performance: PerformanceCounters::default(),
        dead_candidates_by_file,
        file_touch_cache: HashMap::new(),
        line_blame_cache: HashMap::new(),
        symbol_mention_cache: HashMap::new(),
    };

    let mut enriched = Vec::new();
    for dead in &dead_list {
        ctx.performance.dead_candidates_processed += 1;
        let file = string_field(dead, "file").unwrap_or_default();
        let symbol = string_field(dead, "symbol").unwrap_or_default();
        let line = number_field(dead, "line").unwrap_or(1);
        let file_ts = file_last_touched(&mut ctx, &file);
        let line_ts = line_last_touched(&mut ctx, &file, line);
        let mention = symbol_last_mention(&mut ctx, &symbol);
        let tier = staleness_tier(&ctx, line_ts, file_ts);
        let grounding = grounding_for(&ctx, tier, &mention);

        enriched.push(merge_dead_record(
            dead,
            EnrichmentFields {
                file_ts,
                file_days: days_since(ctx.now, file_ts),
                line_ts,
                line_days: days_since(ctx.now, line_ts),
                mention: &mention,
                mention_days: days_since(ctx.now, mention.ts),
                tier,
                grounding: &grounding,
            },
        ));
    }

    let mut by_tier = BTreeMap::from([
        ("fossil".to_string(), 0_i64),
        ("stale".to_string(), 0),
        ("recent".to_string(), 0),
        ("unknown".to_string(), 0),
    ]);
    let mut by_grounding = BTreeMap::from([
        ("grounded".to_string(), 0_i64),
        ("degraded".to_string(), 0),
        ("blind".to_string(), 0),
    ]);
    for entry in &enriched {
        if let Some(tier) = string_field(entry, "stalenessTier") {
            *by_tier.entry(tier).or_insert(0) += 1;
        }
        if let Some(grounding) = string_field(entry, "grounding") {
            *by_grounding.entry(grounding).or_insert(0) += 1;
        }
    }

    let artifact = json!({
        "meta": {
            "generated": generated,
            "root": request.root,
            "tool": TOOL_NAME,
            "gitHead": git_head,
            "symbolsSource": symbols_source,
            "thresholds": {
                "maxAgeDays": request.max_age_days,
                "staleAgeDays": request.stale_age_days,
                "since": request.since,
            },
            "skipPickaxe": request.skip_pickaxe,
            "incremental": incremental_meta(
                &request,
                &cache,
                false,
                if request.incremental_enabled { "cache-miss" } else { "disabled" },
                cache_key.as_deref(),
            ),
        },
        "summary": {
            "total": enriched.len(),
            "byTier": by_tier,
            "byGrounding": by_grounding,
            "pickaxeCacheSize": ctx.symbol_mention_cache.len(),
            "performance": ctx.performance.json(),
        },
        "enriched": enriched,
    });

    if request.incremental_enabled {
        if let Some(cache_key) = cache_key.as_deref() {
            save_cached_artifact(&cache.cache_path, cache_key, &artifact)?;
        }
    }

    Ok(artifact)
}

fn default_max_age_days() -> i64 {
    365
}

fn default_stale_age_days() -> i64 {
    90
}

fn default_since() -> String {
    "5 years ago".to_string()
}

fn default_true() -> bool {
    true
}

fn ensure_git_repo(root: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(root)
        .output();
    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => bail!("staleness-artifact: root is not a git working tree"),
    }
}

fn git(root: &Path, args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_default()
}

fn git_owned(root: &Path, args: &[String]) -> String {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_default()
}

fn file_last_touched(ctx: &mut StalenessContext, rel_file: &str) -> Option<i64> {
    if let Some(cached) = ctx.file_touch_cache.get(rel_file) {
        return *cached;
    }
    ctx.performance.file_touch_git_calls += 1;
    let out = git(
        &ctx.root,
        &["log", "-1", "--format=%at", "--follow", "--", rel_file],
    );
    let ts = parse_i64(out.trim());
    ctx.file_touch_cache.insert(rel_file.to_string(), ts);
    ts
}

fn line_last_touched(ctx: &mut StalenessContext, rel_file: &str, line: i64) -> Option<i64> {
    let safe_line = if line > 0 { line } else { 1 };
    if ctx
        .dead_candidates_by_file
        .get(rel_file)
        .copied()
        .unwrap_or(0)
        <= 1
    {
        ctx.performance.line_blame_git_calls += 1;
        let range = format!("{safe_line},{safe_line}");
        let out = git(
            &ctx.root,
            &["blame", "--porcelain", "-L", &range, "--", rel_file],
        );
        return first_author_time(&out);
    }
    line_blame_times(ctx, rel_file).get(&safe_line).copied()
}

fn line_blame_times(ctx: &mut StalenessContext, rel_file: &str) -> HashMap<i64, i64> {
    if let Some(cached) = ctx.line_blame_cache.get(rel_file) {
        ctx.performance.line_blame_cache_hits += 1;
        return cached.clone();
    }
    ctx.performance.line_blame_cache_misses += 1;
    ctx.performance.line_blame_git_calls += 1;
    let out = git(&ctx.root, &["blame", "--line-porcelain", "--", rel_file]);
    let times = parse_line_blame_times(&out);
    ctx.line_blame_cache
        .insert(rel_file.to_string(), times.clone());
    times
}

fn parse_line_blame_times(out: &str) -> HashMap<i64, i64> {
    let mut times = HashMap::new();
    let mut current_final_line = None;
    let mut current_author_time = None;
    for line in out.lines() {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if (parts.len() == 3 || parts.len() == 4)
            && is_blame_hash(parts[0])
            && parse_i64(parts[1]).is_some()
            && parse_i64(parts[2]).is_some()
        {
            current_final_line = parse_i64(parts[2]);
            current_author_time = None;
            continue;
        }
        if let Some(rest) = line.strip_prefix("author-time ") {
            current_author_time = parse_i64(rest);
            continue;
        }
        if line.starts_with('\t') {
            if let (Some(final_line), Some(author_time)) = (current_final_line, current_author_time)
            {
                times.insert(final_line, author_time);
            }
            current_final_line = None;
            current_author_time = None;
        }
    }
    times
}

fn first_author_time(out: &str) -> Option<i64> {
    out.lines()
        .find_map(|line| line.strip_prefix("author-time ").and_then(parse_i64))
}

fn is_blame_hash(text: &str) -> bool {
    let len = text.len();
    (7..=64).contains(&len) && text.chars().all(|ch| ch == '^' || ch.is_ascii_hexdigit())
}

fn symbol_last_mention(ctx: &mut StalenessContext, symbol: &str) -> Mention {
    if ctx.skip_pickaxe || symbol.len() < MIN_PICKAXE_SYMBOL_LEN || !is_safe_ident(symbol) {
        return Mention {
            status: "skipped",
            ts: None,
        };
    }
    if let Some(cached) = ctx.symbol_mention_cache.get(symbol) {
        return cached.clone();
    }
    let mut args = vec!["log".to_string()];
    if !ctx.since.is_empty() {
        args.push(format!("--since={}", ctx.since));
    }
    args.push(format!("-S{symbol}"));
    args.push("--format=%at".to_string());
    args.push("-1".to_string());
    ctx.performance.symbol_pickaxe_git_calls += 1;
    let out = git_owned(&ctx.root, &args);
    let result = parse_i64(out.trim())
        .map(|ts| Mention {
            status: "warm",
            ts: Some(ts),
        })
        .unwrap_or(Mention {
            status: "cold",
            ts: None,
        });
    ctx.symbol_mention_cache
        .insert(symbol.to_string(), result.clone());
    result
}

fn is_safe_ident(symbol: &str) -> bool {
    let mut chars = symbol.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}

fn staleness_tier(
    ctx: &StalenessContext,
    line_ts: Option<i64>,
    file_ts: Option<i64>,
) -> &'static str {
    let best = line_ts.or(file_ts);
    let age = days_since(ctx.now, best);
    match age {
        None => "unknown",
        Some(age) if age >= ctx.max_age_days => "fossil",
        Some(age) if age >= ctx.stale_age_days => "stale",
        Some(_) => "recent",
    }
}

fn grounding_for(ctx: &StalenessContext, tier: &str, mention: &Mention) -> Grounding {
    let mention_age = if mention.status == "warm" {
        days_since(ctx.now, mention.ts)
    } else {
        None
    };
    let mention_cold = mention.status == "cold";
    let mention_warm_old =
        mention.status == "warm" && mention_age.is_some_and(|age| age >= ctx.stale_age_days);
    let mention_warm_recent =
        mention.status == "warm" && mention_age.is_some_and(|age| age < ctx.stale_age_days);

    if tier == "fossil" && (mention_cold || mention_warm_old) {
        return Grounding {
            grounding: "grounded",
            confidence: "high",
            note: if mention_cold {
                "Definition is ancient AND the name has no pickaxe hits in the scanned window. Strongest temporal signal for safe removal."
            } else {
                "Definition is ancient AND the name has not been touched anywhere recently. Strong safe-to-remove signal."
            },
        };
    }
    if tier == "fossil" && mention_warm_recent {
        return Grounding {
            grounding: "grounded",
            confidence: "medium",
            note: "Definition is ancient but the name is still being edited somewhere in the repo — inspect before removal (possible rename/resurrection).",
        };
    }
    if tier == "fossil" {
        return Grounding {
            grounding: "grounded",
            confidence: "medium",
            note: "Definition is ancient. Pickaxe mention-check was skipped (symbol too short or unsafe) — temporal evidence partial.",
        };
    }
    if tier == "stale" {
        return Grounding {
            grounding: "degraded",
            confidence: "medium",
            note: "Definition has not been touched recently. Dead status plausible but not as defensible as fossil tier.",
        };
    }
    if tier == "recent" {
        return Grounding {
            grounding: "degraded",
            confidence: "low",
            note: "Definition was touched recently — removing may collide with active development.",
        };
    }
    Grounding {
        grounding: "blind",
        confidence: "low",
        note: "File not tracked by git or no commit history. No temporal evidence.",
    }
}

struct EnrichmentFields<'a> {
    file_ts: Option<i64>,
    file_days: Option<i64>,
    line_ts: Option<i64>,
    line_days: Option<i64>,
    mention: &'a Mention,
    mention_days: Option<i64>,
    tier: &'a str,
    grounding: &'a Grounding,
}

fn merge_dead_record(dead: &Value, fields: EnrichmentFields<'_>) -> Value {
    let mut obj = dead.as_object().cloned().unwrap_or_default();
    insert_option_i64(&mut obj, "fileLastTouchedAt", fields.file_ts);
    insert_option_i64(&mut obj, "fileLastTouchedDaysAgo", fields.file_days);
    insert_option_i64(&mut obj, "lineLastTouchedAt", fields.line_ts);
    insert_option_i64(&mut obj, "lineLastTouchedDaysAgo", fields.line_days);
    obj.insert(
        "symbolMentionStatus".to_string(),
        Value::String(fields.mention.status.to_string()),
    );
    insert_option_i64(
        &mut obj,
        "symbolLastMentionedAt",
        (fields.mention.status == "warm")
            .then_some(fields.mention.ts)
            .flatten(),
    );
    insert_option_i64(
        &mut obj,
        "symbolLastMentionedDaysAgo",
        (fields.mention.status == "warm")
            .then_some(fields.mention_days)
            .flatten(),
    );
    obj.insert(
        "stalenessTier".to_string(),
        Value::String(fields.tier.to_string()),
    );
    obj.insert(
        "grounding".to_string(),
        Value::String(fields.grounding.grounding.to_string()),
    );
    obj.insert(
        "confidence".to_string(),
        Value::String(fields.grounding.confidence.to_string()),
    );
    obj.insert(
        "note".to_string(),
        Value::String(fields.grounding.note.to_string()),
    );
    Value::Object(obj)
}

fn insert_option_i64(obj: &mut Map<String, Value>, key: &str, value: Option<i64>) {
    obj.insert(
        key.to_string(),
        value.map(Value::from).unwrap_or(Value::Null),
    );
}

fn days_since(now: i64, ts: Option<i64>) -> Option<i64> {
    ts.map(|ts| floor_div(now - ts, 86_400))
}

fn floor_div(numerator: i64, denominator: i64) -> i64 {
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    if remainder != 0 && ((remainder > 0) != (denominator > 0)) {
        quotient - 1
    } else {
        quotient
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn parse_i64(text: &str) -> Option<i64> {
    text.trim().parse::<i64>().ok()
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value.get(field)?.as_str().map(ToString::to_string)
}

fn number_field(value: &Value, field: &str) -> Option<i64> {
    value.get(field)?.as_i64().or_else(|| {
        value
            .get(field)?
            .as_f64()
            .filter(|value| value.is_finite())
            .map(|value| value as i64)
    })
}

#[derive(Debug)]
struct CacheContext {
    cache_root: PathBuf,
    repo_fingerprint: String,
    repo_cache_dir: PathBuf,
    cache_path: PathBuf,
}

fn cache_context(
    request: &StalenessRequest,
    root: &Path,
    _git_head: &str,
    _dead_list: &[Value],
) -> CacheContext {
    let cache_root = request
        .cache_root
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join(".audit").join(".cache"));
    let repo_fingerprint = repo_fingerprint_for_root(root);
    let repo_cache_dir = cache_root
        .join("incremental")
        .join(repo_fingerprint.trim_start_matches("sha256:"));
    let cache_path = repo_cache_dir
        .join("staleness")
        .join("staleness.cache.json");
    CacheContext {
        cache_root,
        repo_fingerprint,
        repo_cache_dir,
        cache_path,
    }
}

fn repo_fingerprint_for_root(root: &Path) -> String {
    let real_root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let marker = if real_root.join(".git").exists() {
        "git-worktree"
    } else if real_root.join("package.json").exists() {
        "package-root"
    } else {
        "directory-root"
    };
    sha256_text(&canonical_json(&json!({
        "schemaVersion": 1,
        "realRoot": slash_path(&real_root),
        "marker": marker,
        "platform": js_platform(),
    })))
}

fn staleness_cache_key(request: &StalenessRequest, git_head: &str, dead_list: &[Value]) -> String {
    let observed_date = request
        .generated
        .as_deref()
        .and_then(|generated| generated.get(0..10))
        .unwrap_or("unknown");
    let dead_list = dead_list
        .iter()
        .map(|entry| {
            json!({
                "file": entry.get("file").cloned().unwrap_or(Value::Null),
                "line": entry.get("line").cloned().unwrap_or(Value::Null),
                "symbol": entry.get("symbol").cloned().unwrap_or(Value::Null),
                "identity": entry.get("identity").cloned().unwrap_or(Value::Null),
            })
        })
        .collect::<Vec<_>>();
    sha256_text(&canonical_json(&json!({
        "schemaVersion": STALENESS_CACHE_SCHEMA_VERSION,
        "gitHead": git_head,
        "observedDate": observed_date,
        "thresholds": {
            "maxAgeDays": request.max_age_days,
            "staleAgeDays": request.stale_age_days,
            "since": request.since,
        },
        "skipPickaxe": request.skip_pickaxe,
        "deadList": dead_list,
    })))
}

fn incremental_meta(
    request: &StalenessRequest,
    cache: &CacheContext,
    reused_result: bool,
    reason: &str,
    cache_key: Option<&str>,
) -> Value {
    json!({
        "enabled": request.incremental_enabled,
        "reusedResult": reused_result,
        "reason": reason,
        "cacheRoot": cache.cache_root.to_string_lossy(),
        "repoFingerprint": cache.repo_fingerprint,
        "cacheSchemaVersion": STALENESS_CACHE_SCHEMA_VERSION,
        "cacheKey": cache_key,
    })
}

fn load_cached_artifact(cache_path: &Path, cache_key: &str) -> Option<Value> {
    let text = fs::read_to_string(cache_path).ok()?;
    let parsed = serde_json::from_str::<Value>(&text).ok()?;
    if parsed.get("schemaVersion")?.as_i64()? != STALENESS_CACHE_SCHEMA_VERSION {
        return None;
    }
    parsed
        .get("entries")?
        .get(cache_key)?
        .get("artifact")
        .cloned()
}

fn save_cached_artifact(cache_path: &Path, cache_key: &str, artifact: &Value) -> Result<()> {
    let mut cache = fs::read_to_string(cache_path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .filter(|value| {
            value.get("schemaVersion").and_then(Value::as_i64)
                == Some(STALENESS_CACHE_SCHEMA_VERSION)
                && value.get("entries").is_some_and(Value::is_object)
        })
        .unwrap_or_else(|| {
            json!({
                "schemaVersion": STALENESS_CACHE_SCHEMA_VERSION,
                "entries": {},
            })
        });
    if let Some(entries) = cache.get_mut("entries").and_then(Value::as_object_mut) {
        entries.insert(cache_key.to_string(), json!({ "artifact": artifact }));
    }
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        cache_path,
        format!("{}\n", serde_json::to_string_pretty(&cache)?),
    )?;
    Ok(())
}

fn refresh_cached_artifact(
    mut cached: Value,
    generated: &str,
    root: &str,
    git_head: Value,
    symbols_source: &str,
    incremental: Value,
) -> Value {
    if let Some(meta) = cached.get_mut("meta").and_then(Value::as_object_mut) {
        meta.insert(
            "generated".to_string(),
            Value::String(generated.to_string()),
        );
        meta.insert("root".to_string(), Value::String(root.to_string()));
        meta.insert("gitHead".to_string(), git_head);
        meta.insert(
            "symbolsSource".to_string(),
            Value::String(symbols_source.to_string()),
        );
        meta.insert("incremental".to_string(), incremental);
    }
    if let Some(summary) = cached.get_mut("summary").and_then(Value::as_object_mut) {
        summary.insert(
            "performance".to_string(),
            PerformanceCounters::default().json(),
        );
    }
    cached
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn js_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "win32"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        std::env::consts::OS
    }
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
        }
        Value::Array(items) => {
            let body = items
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{body}]")
        }
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();
            let body = keys
                .into_iter()
                .map(|key| {
                    let encoded_key =
                        serde_json::to_string(key).unwrap_or_else(|_| "\"\"".to_string());
                    format!("{encoded_key}:{}", canonical_json(&map[key]))
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{body}}}")
        }
    }
}
