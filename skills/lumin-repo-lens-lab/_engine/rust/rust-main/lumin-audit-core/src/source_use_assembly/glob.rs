use serde_json::{json, Map, Value};

use crate::relative_source_resolver::{
    dirname_text, join_relative_spec, normalize_path_text, RelativeSourceResolver,
};

use super::input::SourceUseAssemblyRecord;
use super::path::{basename_text, is_inside_or_same, relative_scope, root_relative};

pub(super) enum ImportMetaGlobExpansion {
    Resolved { targets: Vec<String> },
    Unsupported { evidence: Map<String, Value> },
}

#[derive(Debug)]
struct ParsedImportMetaGlobPattern {
    segments: Vec<String>,
    star_index: usize,
    prefix: String,
    suffix: String,
}

pub(super) fn expand_import_meta_glob(
    root: &str,
    resolver: &RelativeSourceResolver,
    record: &SourceUseAssemblyRecord,
    cap: usize,
) -> ImportMetaGlobExpansion {
    let pattern = record.from_spec.as_deref().unwrap_or_default();
    let parsed = match validate_pattern(pattern) {
        Ok(parsed) => parsed,
        Err(reason) => return unsupported(reason, None, None, None),
    };

    let consumer_dir = dirname_text(&record.consumer_file);
    let base_pattern = if parsed.star_index == 0 {
        ".".to_string()
    } else {
        parsed.segments[..parsed.star_index].join("/")
    };
    let base_dir = join_relative_spec(consumer_dir, &base_pattern);
    if !is_inside_or_same(root, &base_dir) {
        return unsupported(
            "import-meta-glob-outside-root-unsupported",
            None,
            None,
            None,
        );
    }

    let mut matches = resolver
        .source_files()
        .filter(|source_file| {
            normalize_path_text(dirname_text(source_file)) == normalize_path_text(&base_dir)
                && is_inside_or_same(root, source_file)
                && basename_text(source_file).is_some_and(|basename| {
                    basename.starts_with(&parsed.prefix) && basename.ends_with(&parsed.suffix)
                })
        })
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    matches.sort_by_key(|path| root_relative(root, path));

    if matches.is_empty() {
        return unsupported(
            "import-meta-glob-zero-matches",
            Some(0),
            None,
            Some(relative_scope(root, &base_dir)),
        );
    }
    if matches.len() > cap {
        return unsupported(
            "import-meta-glob-match-cap-exceeded",
            Some(matches.len()),
            Some(cap),
            Some(relative_scope(root, &base_dir)),
        );
    }

    ImportMetaGlobExpansion::Resolved { targets: matches }
}

fn validate_pattern(
    pattern: &str,
) -> std::result::Result<ParsedImportMetaGlobPattern, &'static str> {
    if pattern.is_empty() || pattern == "import.meta.glob(<nonliteral>)" {
        return Err("import-meta-glob-nonliteral-unsupported");
    }
    let normalized = pattern.replace('\\', "/");
    if !normalized.starts_with("./") && !normalized.starts_with("../") {
        return Err("import-meta-glob-nonrelative-unsupported");
    }
    if normalized.contains('?')
        || normalized.contains('[')
        || normalized.contains(']')
        || normalized.contains('{')
        || normalized.contains('}')
    {
        return Err("import-meta-glob-unsupported-pattern");
    }
    if normalized.chars().filter(|ch| *ch == '*').count() != 1 {
        return Err("import-meta-glob-unsupported-pattern");
    }

    let segments = normalized
        .split('/')
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let Some(star_index) = segments.iter().position(|segment| segment.contains('*')) else {
        return Err("import-meta-glob-unsupported-pattern");
    };
    let star_segment = &segments[star_index];
    if star_segment.is_empty() || star_segment.contains("**") {
        return Err("import-meta-glob-unsupported-pattern");
    }
    let Some((prefix, suffix)) = star_segment.split_once('*') else {
        return Err("import-meta-glob-unsupported-pattern");
    };
    if !is_source_suffix(suffix) {
        return Err("import-meta-glob-target-extension-unsupported");
    }
    let prefix = prefix.to_string();
    let suffix = suffix.to_string();

    Ok(ParsedImportMetaGlobPattern {
        segments,
        star_index,
        prefix,
        suffix,
    })
}

fn is_source_suffix(suffix: &str) -> bool {
    let lower = suffix.to_ascii_lowercase();
    if lower.ends_with(".d.ts") || lower.ends_with(".d.mts") || lower.ends_with(".d.cts") {
        return false;
    }
    [".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

fn unsupported(
    reason: &'static str,
    match_count: Option<usize>,
    cap: Option<usize>,
    affected_package_scope: Option<String>,
) -> ImportMetaGlobExpansion {
    let mut evidence = Map::new();
    evidence.insert("reason".to_string(), json!(reason));
    evidence.insert("resolverStage".to_string(), json!("import-meta-glob"));
    evidence.insert("outputLevel".to_string(), json!("unsupported"));
    evidence.insert("unsupportedFamily".to_string(), json!("dynamic-modules"));
    evidence.insert("hint".to_string(), json!("dynamic-module-surface"));
    evidence.insert("scanPolicy".to_string(), json!("scanned-source-files"));
    if let Some(match_count) = match_count {
        evidence.insert("matchCount".to_string(), json!(match_count));
    }
    if let Some(cap) = cap {
        evidence.insert("cap".to_string(), json!(cap));
    }
    if let Some(scope) = affected_package_scope {
        evidence.insert("affectedPackageScope".to_string(), json!(scope));
    }
    ImportMetaGlobExpansion::Unsupported { evidence }
}
