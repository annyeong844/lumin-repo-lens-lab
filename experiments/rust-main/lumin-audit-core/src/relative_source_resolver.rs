use std::collections::{BTreeMap, BTreeSet};

const RESOLVE_FILE_EXTENSIONS: &[&str] = &[
    "", ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".mts", ".cts", ".d.ts", ".d.mts", ".d.cts",
];

const RESOLVE_INDEX_EXTENSIONS: &[&str] = &[
    "/index.ts",
    "/index.tsx",
    "/index.js",
    "/index.jsx",
    "/index.mjs",
    "/index.cjs",
    "/index.mts",
    "/index.cts",
    "/index.d.ts",
    "/index.d.mts",
    "/index.d.cts",
];

#[derive(Debug)]
pub(crate) struct RelativeSourceResolver {
    source_files: BTreeMap<String, String>,
    listed_source_files: Vec<String>,
}

impl RelativeSourceResolver {
    pub(crate) fn from_paths(source_files: Vec<String>) -> Self {
        let mut paths = BTreeMap::new();
        let mut listed = BTreeSet::new();
        for source_file in source_files {
            paths
                .entry(normalize_path_text(&source_file))
                .or_insert_with(|| source_file.clone());
            listed.insert(source_file);
        }
        Self {
            source_files: paths,
            listed_source_files: listed.into_iter().collect(),
        }
    }

    pub(crate) fn from_rooted_paths(root: &str, source_files: Vec<String>) -> Self {
        let mut paths = BTreeMap::new();
        let mut listed = BTreeSet::new();
        let root = normalize_path_text(root);
        for source_file in source_files {
            let normalized = normalize_path_text(&source_file);
            let resolved = if is_absolute_path_text(&normalized) {
                normalized.clone()
            } else {
                normalize_path_text(&format!("{}/{}", root.trim_end_matches('/'), normalized))
            };
            paths.entry(normalized).or_insert(resolved.clone());
            paths.entry(resolved.clone()).or_insert(resolved.clone());
            paths
                .entry(root_relative(&root, &resolved))
                .or_insert(resolved.clone());
            listed.insert(resolved);
        }
        Self {
            source_files: paths,
            listed_source_files: listed.into_iter().collect(),
        }
    }

    pub(crate) fn resolve(&self, from_file: &str, spec: &str) -> Option<String> {
        if !is_relative_spec(spec) {
            return None;
        }
        let base = join_relative_spec(dirname_text(from_file), spec);
        for extension in RESOLVE_FILE_EXTENSIONS {
            if let Some(resolved) = self.source_file(&format!("{base}{extension}")) {
                return Some(resolved);
            }
        }
        for extension in RESOLVE_INDEX_EXTENSIONS {
            if let Some(resolved) = self.source_file(&format!("{base}{extension}")) {
                return Some(resolved);
            }
        }
        if js_output_extension(spec) {
            for source_extension in js_output_source_extensions(spec) {
                if let Some(swapped) = replace_js_output_extension(spec, source_extension) {
                    let candidate = join_relative_spec(dirname_text(from_file), &swapped);
                    if let Some(resolved) = self.source_file(&candidate) {
                        return Some(resolved);
                    }
                }
            }
            if let Some(stripped) = strip_js_output_extension(&base) {
                for extension in RESOLVE_INDEX_EXTENSIONS {
                    if let Some(resolved) = self.source_file(&format!("{stripped}{extension}")) {
                        return Some(resolved);
                    }
                }
            }
        }
        None
    }

    pub(crate) fn source_files(&self) -> impl Iterator<Item = &str> {
        self.listed_source_files.iter().map(String::as_str)
    }

    fn source_file(&self, candidate: &str) -> Option<String> {
        self.source_files
            .get(&normalize_path_text(candidate))
            .cloned()
    }
}

pub(crate) fn dirname_text(path: &str) -> &str {
    path.rfind(['/', '\\']).map_or("", |index| &path[..index])
}

pub(crate) fn join_relative_spec(base: &str, spec: &str) -> String {
    let joined = if base.is_empty() {
        spec.to_string()
    } else {
        format!("{base}/{spec}")
    };
    normalize_path_text(&joined)
}

pub(crate) fn normalize_path_text(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let (prefix, rest) = split_path_prefix(&normalized);
    let absolute = rest.starts_with('/');
    let mut parts = Vec::new();
    for part in rest.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if let Some(last) = parts.last() {
                if last != &".." {
                    parts.pop();
                    continue;
                }
            }
            if !absolute {
                parts.push(part);
            }
            continue;
        }
        parts.push(part);
    }

    let body = parts.join("/");
    match (prefix.is_empty(), absolute, body.is_empty()) {
        (false, _, false) => format!("{prefix}/{body}"),
        (false, _, true) => prefix.to_string(),
        (true, true, false) => format!("/{body}"),
        (true, true, true) => "/".to_string(),
        (true, false, false) => body,
        (true, false, true) => ".".to_string(),
    }
}

fn root_relative(root: &str, path: &str) -> String {
    let root = root.trim_end_matches('/');
    path.strip_prefix(&format!("{root}/"))
        .map_or_else(|| path.to_string(), ToString::to_string)
}

fn split_path_prefix(path: &str) -> (&str, &str) {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return (&path[..2], &path[2..]);
    }
    ("", path)
}

fn is_absolute_path_text(path: &str) -> bool {
    let (prefix, rest) = split_path_prefix(path);
    !prefix.is_empty() || rest.starts_with('/')
}

fn is_relative_spec(spec: &str) -> bool {
    spec.starts_with("./") || spec.starts_with("../")
}

fn js_output_extension(spec: &str) -> bool {
    [".mjs", ".cjs", ".js", ".jsx"]
        .iter()
        .any(|extension| spec.ends_with(extension))
}

fn js_output_source_extensions(spec: &str) -> &'static [&'static str] {
    if spec.ends_with(".jsx") {
        &[".tsx", ".ts"]
    } else {
        &[".ts", ".tsx", ".mts", ".cts"]
    }
}

fn replace_js_output_extension(spec: &str, replacement: &str) -> Option<String> {
    [".mjs", ".cjs", ".js", ".jsx"]
        .iter()
        .find_map(|extension| {
            spec.strip_suffix(extension)
                .map(|prefix| format!("{prefix}{replacement}"))
        })
}

fn strip_js_output_extension(spec: &str) -> Option<&str> {
    [".mjs", ".cjs", ".js", ".jsx"]
        .iter()
        .find_map(|extension| spec.strip_suffix(extension))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsx_resolution_prefers_tsx_and_keeps_ts_fallback() {
        let resolver = RelativeSourceResolver::from_paths(vec![
            "src/view.ts".to_string(),
            "src/view.tsx".to_string(),
        ]);

        assert_eq!(
            resolver.resolve("src/main.ts", "./view.jsx").as_deref(),
            Some("src/view.tsx")
        );

        let resolver = RelativeSourceResolver::from_paths(vec!["src/view.ts".to_string()]);
        assert_eq!(
            resolver.resolve("src/main.ts", "./view.jsx").as_deref(),
            Some("src/view.ts")
        );
    }

    #[test]
    fn rooted_paths_resolve_relative_and_absolute_consumers_to_the_same_file() {
        let resolver =
            RelativeSourceResolver::from_rooted_paths("C:/repo", vec!["src/dep.ts".to_string()]);

        assert_eq!(
            resolver.resolve("src/main.ts", "./dep").as_deref(),
            Some("C:/repo/src/dep.ts")
        );
        assert_eq!(
            resolver.resolve("C:/repo/src/main.ts", "./dep").as_deref(),
            Some("C:/repo/src/dep.ts")
        );
        assert_eq!(
            resolver.source_files().collect::<Vec<_>>(),
            ["C:/repo/src/dep.ts"]
        );
    }
}
