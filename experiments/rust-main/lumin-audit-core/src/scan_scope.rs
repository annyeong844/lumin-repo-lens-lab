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
const WALK_PRUNE_NAMES: &[&str] = &["node_modules", ".git", "coverage"];
const WALK_PRUNE_PREFIXES: &[&str] = &["dist", "build"];

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
    for segment in segments.iter().take(walk_end).skip(1) {
        if should_prune_walk_dir(segment) {
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

fn should_prune_walk_dir(name: &str) -> bool {
    if WALK_PRUNE_NAMES.contains(&name) {
        return true;
    }
    WALK_PRUNE_PREFIXES
        .iter()
        .any(|prefix| name == *prefix || name.starts_with(&format!("{prefix}-")))
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

fn is_test_like_path(path: &str) -> bool {
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
