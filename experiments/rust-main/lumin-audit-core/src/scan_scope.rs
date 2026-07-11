use anyhow::{Context, Result};
use rayon::{prelude::*, ThreadPoolBuilder};
use std::fs;
use std::path::{Path, PathBuf};

const JS_FAMILY_EXTENSIONS: &[&str] = &["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"];
const CANONICAL_MARKERS: &[&str] = &[
    "src",
    "lib",
    "bin",
    "types",
    "apps",
    "packages",
    "tests",
    "test",
    "__tests__",
    "e2e",
    "integration",
    "public",
    "app",
    "pages",
    "scripts",
];
const ROOT_PRUNE_NAMES: &[&str] = &[
    "node_modules",
    ".git",
    "coverage",
    ".next",
    ".svelte-kit",
    ".astro",
    ".turbo",
    ".cache",
    ".nuxt",
    ".output",
    "out",
    "target",
    ".venv",
    "venv",
    "__pycache__",
];
const WALK_PRUNE_NAMES: &[&str] = &["node_modules", ".git"];
const WALK_PRUNE_PREFIXES: &[&str] = &["dist", "build"];
const SOURCE_DISCOVERY_WORKER_STACK_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ScanScopeOptions {
    pub include_tests: bool,
    pub exclude: Vec<String>,
    pub languages: Vec<String>,
    pub directory: bool,
}

impl Default for ScanScopeOptions {
    fn default() -> Self {
        Self {
            include_tests: true,
            exclude: Vec::new(),
            languages: JS_FAMILY_EXTENSIONS
                .iter()
                .map(|extension| (*extension).to_string())
                .collect(),
            directory: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanScopeStatus {
    pub included: bool,
    pub reason: Option<&'static str>,
}

pub fn scan_scope_status_for_path(
    root: &Path,
    full: &Path,
    options: &ScanScopeOptions,
) -> ScanScopeStatus {
    let root = absolute_existing_or_lexical(root);
    let full = absolute_existing_or_lexical(full);
    let Some(rel) = relative_posix(&root, &full) else {
        return excluded("outside-root");
    };

    if rel.is_empty() {
        return if options.directory {
            included()
        } else {
            excluded("outside-root")
        };
    }

    if !options.directory && !extension_is_in_scope(&full, &options.languages) {
        return excluded("language-excluded");
    }
    if !options.directory && !options.include_tests && is_test_like_path(&rel) {
        return excluded("test-excluded");
    }

    let segments = rel
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if let Some(root_segment) = segments.first() {
        if should_prune_root_dir(root_segment) {
            return excluded("root-pruned");
        }
    }

    let walk_end = if options.directory {
        segments.len()
    } else {
        segments.len().saturating_sub(1)
    };
    let mut walk_cursor = root.join(segments.first().copied().unwrap_or_default());
    for segment in segments.iter().take(walk_end).skip(1) {
        walk_cursor.push(segment);
        if should_prune_walk_dir(segment, &walk_cursor) {
            return excluded("walk-pruned");
        }
    }

    let exclude_rules = build_exclude_rules(&options.exclude);
    let directory_end = if options.directory {
        segments.len()
    } else {
        segments.len().saturating_sub(1)
    };
    let mut cursor = root.clone();
    for segment in segments.iter().take(directory_end) {
        cursor.push(segment);
        if is_excluded_path(&root, &cursor, &exclude_rules, true) {
            return excluded("excluded");
        }
    }
    if !options.directory && is_excluded_path(&root, &full, &exclude_rules, false) {
        return excluded("excluded");
    }

    included()
}

pub fn collect_source_files(root: &Path, options: &ScanScopeOptions) -> Result<Vec<PathBuf>> {
    let root = absolute_existing_or_lexical(root);
    if !root.is_dir() {
        anyhow::bail!(
            "source discovery root is not a directory: {}",
            root.display()
        );
    }

    let exclude_rules = build_exclude_rules(&options.exclude);
    let root_entries = read_directory(&root)?;
    let mut search_dirs = Vec::new();
    let mut files = Vec::new();

    for entry in &root_entries {
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?;
        if file_type.is_symlink() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let full = entry.path();
        if file_type.is_dir() {
            if should_prune_root_dir(&name) || is_excluded_path(&root, &full, &exclude_rules, true)
            {
                continue;
            }
            search_dirs.push(full);
        } else if file_type.is_file()
            && extension_is_in_scope(&full, &options.languages)
            && !is_excluded_path(&root, &full, &exclude_rules, false)
        {
            files.push(full);
        }
    }

    if search_dirs.is_empty() {
        walk_source_files(&root, &root, true, options, &exclude_rules, &mut files)?;
    } else {
        files.extend(walk_search_directories(
            &root,
            search_dirs,
            options,
            &exclude_rules,
        )?);
    }

    files.sort();
    files.dedup();
    if !options.include_tests {
        files.retain(|file| {
            relative_posix(&root, file).is_some_and(|relative| !is_test_like_path(&relative))
        });
    }
    Ok(files)
}

fn walk_search_directories(
    root: &Path,
    search_dirs: Vec<PathBuf>,
    options: &ScanScopeOptions,
    exclude_rules: &[ExcludeRule],
) -> Result<Vec<PathBuf>> {
    let available_threads = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1);
    if available_threads == 1 {
        let mut files = Vec::new();
        for directory in search_dirs {
            walk_source_files(root, &directory, false, options, exclude_rules, &mut files)?;
        }
        return Ok(files);
    }

    let pool = ThreadPoolBuilder::new()
        .num_threads(available_threads)
        .stack_size(SOURCE_DISCOVERY_WORKER_STACK_BYTES)
        .build()
        .context("failed to build source discovery worker pool")?;
    let directory_results = pool.install(|| {
        search_dirs
            .into_par_iter()
            .map(|directory| walk_source_files_parallel(root, &directory, options, exclude_rules))
            .collect::<Vec<Result<Vec<PathBuf>>>>()
    });

    let mut files = Vec::new();
    for result in directory_results {
        files.extend(result?);
    }
    Ok(files)
}

fn walk_source_files_parallel(
    root: &Path,
    directory: &Path,
    options: &ScanScopeOptions,
    exclude_rules: &[ExcludeRule],
) -> Result<Vec<PathBuf>> {
    let entry_results = read_directory(directory)?
        .into_par_iter()
        .map(|entry| {
            let file_type = entry
                .file_type()
                .with_context(|| format!("failed to inspect {}", entry.path().display()))?;
            if file_type.is_symlink() {
                return Ok(Vec::new());
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let full = entry.path();
            if file_type.is_dir() {
                if should_prune_walk_dir(&name, &full)
                    || is_excluded_path(root, &full, exclude_rules, true)
                {
                    return Ok(Vec::new());
                }
                return walk_source_files_parallel(root, &full, options, exclude_rules);
            }
            if file_type.is_file()
                && extension_is_in_scope(&full, &options.languages)
                && !is_excluded_path(root, &full, exclude_rules, false)
            {
                return Ok(vec![full]);
            }
            Ok(Vec::new())
        })
        .collect::<Vec<Result<Vec<PathBuf>>>>();

    let mut files = Vec::new();
    for result in entry_results {
        files.extend(result?);
    }
    Ok(files)
}

fn walk_source_files(
    root: &Path,
    directory: &Path,
    walking_root: bool,
    options: &ScanScopeOptions,
    exclude_rules: &[ExcludeRule],
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in read_directory(directory)? {
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?;
        if file_type.is_symlink() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let full = entry.path();
        if file_type.is_dir() {
            if (walking_root && should_prune_root_dir(&name))
                || should_prune_walk_dir(&name, &full)
                || is_excluded_path(root, &full, exclude_rules, true)
            {
                continue;
            }
            walk_source_files(root, &full, false, options, exclude_rules, files)?;
        } else if file_type.is_file()
            && extension_is_in_scope(&full, &options.languages)
            && !is_excluded_path(root, &full, exclude_rules, false)
        {
            files.push(full);
        }
    }
    Ok(())
}

fn read_directory(directory: &Path) -> Result<Vec<fs::DirEntry>> {
    let mut entries = fs::read_dir(directory)
        .with_context(|| format!("failed to read directory {}", directory.display()))?
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("failed to enumerate directory {}", directory.display()))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    Ok(entries)
}

pub fn to_repo_relative(root: &Path, candidate: &str) -> Option<String> {
    let candidate = Path::new(candidate);
    let abs = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    };
    relative_posix(
        &absolute_existing_or_lexical(root),
        &absolute_existing_or_lexical(&abs),
    )
    .filter(|relative| !relative.is_empty())
}

fn included() -> ScanScopeStatus {
    ScanScopeStatus {
        included: true,
        reason: None,
    }
}

fn excluded(reason: &'static str) -> ScanScopeStatus {
    ScanScopeStatus {
        included: false,
        reason: Some(reason),
    }
}

fn absolute_existing_or_lexical(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

fn relative_posix(root: &Path, full: &Path) -> Option<String> {
    let rel = full.strip_prefix(root).ok()?;
    Some(
        rel.components()
            .map(|component| component.as_os_str().to_string_lossy())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("/"),
    )
}

fn extension_is_in_scope(path: &Path, languages: &[String]) -> bool {
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_ascii_lowercase());
    let Some(extension) = extension else {
        return false;
    };
    languages
        .iter()
        .any(|language| language.trim_start_matches('.') == extension)
}

fn should_prune_root_dir(name: &str) -> bool {
    if ROOT_PRUNE_NAMES.contains(&name) {
        return true;
    }
    name == "dist"
        || name.starts_with("dist-")
        || name == "build"
        || name.starts_with("build-")
        || (name.starts_with('.') && !CANONICAL_MARKERS.contains(&name))
}

fn should_prune_walk_dir(name: &str, full: &Path) -> bool {
    if WALK_PRUNE_NAMES.contains(&name) {
        return true;
    }
    if name == "target"
        && full
            .parent()
            .is_some_and(|parent| parent.join("Cargo.toml").is_file())
    {
        return true;
    }
    WALK_PRUNE_PREFIXES
        .iter()
        .any(|prefix| name == *prefix || name.starts_with(&format!("{prefix}-")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prunes_only_target_directories_owned_by_cargo() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        let cargo_root = root.join("packages/cargo-app");
        let cargo_target_dir = cargo_root.join("target");
        let authored_target_dir = root.join("src/target");
        let cargo_target_file = cargo_target_dir.join("generated.ts");
        let authored_target_file = authored_target_dir.join("index.ts");
        std::fs::create_dir_all(&cargo_target_dir)?;
        std::fs::create_dir_all(&authored_target_dir)?;
        std::fs::write(
            cargo_root.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
        )?;
        std::fs::write(&cargo_target_file, "export const generated = true;\n")?;
        std::fs::write(&authored_target_file, "export const authored = true;\n")?;

        let options = ScanScopeOptions::default();
        assert_eq!(
            scan_scope_status_for_path(root, &cargo_target_file, &options),
            excluded("walk-pruned")
        );
        assert_eq!(
            scan_scope_status_for_path(root, &authored_target_file, &options),
            included()
        );
        Ok(())
    }

    #[test]
    fn preserves_authored_nested_coverage_modules() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        let authored = root.join("src/coverage/absence.rs");
        let parent = authored
            .parent()
            .ok_or_else(|| anyhow::anyhow!("fixture path has no parent"))?;
        std::fs::create_dir_all(parent)?;
        std::fs::write(&authored, "pub const COVERED: bool = true;\n")?;

        let options = ScanScopeOptions {
            languages: vec!["rs".to_string()],
            ..ScanScopeOptions::default()
        };
        assert_eq!(
            scan_scope_status_for_path(root, &authored, &options),
            included()
        );
        Ok(())
    }

    #[test]
    fn collects_files_with_checked_pruning_and_analysis_scope() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        let paths = [
            "index.ts",
            "coverage/generated.ts",
            "src/coverage/authored.ts",
            "src/target/index.ts",
            "src/unit.test.ts",
            "src/excluded/skip.ts",
            "packages/pkg/target/generated.ts",
        ];
        for path in paths {
            let file = root.join(path);
            fs::create_dir_all(file.parent().context("fixture file has no parent")?)?;
            fs::write(file, "export const value = true;\n")?;
        }
        fs::write(
            root.join("packages/pkg/Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
        )?;

        let files = collect_source_files(
            root,
            &ScanScopeOptions {
                include_tests: false,
                exclude: vec!["src/excluded".to_string()],
                ..ScanScopeOptions::default()
            },
        )?;
        let relative = files
            .iter()
            .filter_map(|file| to_repo_relative(root, &file.to_string_lossy()))
            .collect::<Vec<_>>();
        assert_eq!(
            relative,
            vec![
                "index.ts".to_string(),
                "src/coverage/authored.ts".to_string(),
                "src/target/index.ts".to_string(),
            ]
        );
        Ok(())
    }

    #[test]
    fn flat_repo_fallback_keeps_root_entries_and_prunes_root_coverage() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        fs::create_dir_all(root.join("coverage"))?;
        fs::write(root.join("index.ts"), "export const entry = true;\n")?;
        fs::write(
            root.join("coverage/generated.ts"),
            "export const generated = true;\n",
        )?;

        let files = collect_source_files(root, &ScanScopeOptions::default())?;
        let relative = files
            .iter()
            .filter_map(|file| to_repo_relative(root, &file.to_string_lossy()))
            .collect::<Vec<_>>();
        assert_eq!(relative, vec!["index.ts"]);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExcludeRule {
    File { pattern: String },
    Directory { needle: String },
}

fn build_exclude_rules(exclude: &[String]) -> Vec<ExcludeRule> {
    exclude
        .iter()
        .filter_map(|pattern| {
            let pattern = normalize_exclude_pattern(pattern);
            if pattern.is_empty() {
                return None;
            }
            let last_segment = pattern.rsplit('/').next().unwrap_or(pattern.as_str());
            if last_segment.contains('.') {
                Some(ExcludeRule::File { pattern })
            } else {
                Some(ExcludeRule::Directory {
                    needle: format!("/{pattern}/"),
                })
            }
        })
        .collect()
}

fn normalize_exclude_pattern(pattern: &str) -> String {
    let mut pattern = pattern.trim().replace('\\', "/");
    if let Some(stripped) = pattern.strip_prefix("*/") {
        pattern = stripped.to_string();
    }
    if let Some(stripped) = pattern.strip_suffix("/*") {
        pattern = stripped.to_string();
    }
    if let Some(stripped) = pattern.strip_prefix("./") {
        pattern = stripped.to_string();
    }
    while pattern.starts_with('/') {
        pattern.remove(0);
    }
    while pattern.ends_with('/') {
        pattern.pop();
    }
    pattern
}

fn is_excluded_path(root: &Path, full: &Path, rules: &[ExcludeRule], directory: bool) -> bool {
    let normalized = bounded_relative_path(root, full, directory);
    rules.iter().any(|rule| match rule {
        ExcludeRule::Directory { needle } => normalized.contains(needle),
        ExcludeRule::File { pattern } => !directory && normalized.ends_with(&format!("/{pattern}")),
    })
}

fn bounded_relative_path(root: &Path, full: &Path, directory: bool) -> String {
    let mut normalized = match relative_posix(root, full) {
        Some(rel) if !rel.is_empty() => format!("/{rel}"),
        _ => format!(
            "/{}",
            full.to_string_lossy()
                .replace('\\', "/")
                .trim_start_matches('/')
        ),
    };
    if directory {
        normalized.push('/');
    }
    normalized
}

pub(crate) fn is_test_like_path(path: &str) -> bool {
    let base = path.rsplit('/').next().unwrap_or(path);
    if is_js_test_file(base) || is_test_support_file(base) {
        return true;
    }
    if base.starts_with("test_") && base.ends_with(".py") {
        return true;
    }
    if base.ends_with("_test.py") || base.ends_with("_test.go") {
        return true;
    }

    path.split('/').any(|segment| {
        matches!(
            segment,
            "test"
                | "tests"
                | "e2e"
                | "integration"
                | "fixtures"
                | "fixture"
                | "mocks"
                | "mock"
                | "test-support"
                | "test-utils"
                | "runtime-tests"
                | "playground"
                | "playgrounds"
        ) || (segment.len() >= 4 && segment.starts_with("__") && segment.ends_with("__"))
            || segment.ends_with("-fixture")
            || segment.ends_with("-fixtures")
    })
}

fn is_js_test_file(base: &str) -> bool {
    let Some((stem, extension)) = base.rsplit_once('.') else {
        return false;
    };
    if !JS_FAMILY_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()) {
        return false;
    }
    stem.ends_with(".test") || stem.ends_with(".spec")
}

fn is_test_support_file(base: &str) -> bool {
    let Some((stem, extension)) = base.rsplit_once('.') else {
        return false;
    };
    if !JS_FAMILY_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()) {
        return false;
    }
    stem == "test-support"
        || stem.ends_with("-test-support")
        || stem.ends_with("_test-support")
        || stem.ends_with(".test-support")
}
