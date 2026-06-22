use std::collections::BTreeSet;

use lumin_rust_source_health::protocol::HealthResponse;
use serde::Serialize;

// Mirrors _lib/pre-write-lookup-file.mjs domain cluster policy.
pub(in crate::prewrite) const DOMAIN_CLUSTER_MIN_MATCHES: usize = 2;
pub(in crate::prewrite) const DOMAIN_CLUSTER_MAX_EXAMPLES: usize = 8;
pub(in crate::prewrite) const DOMAIN_CLUSTER_MIN_PREFIX_LEN: usize = 4;
const GENERIC_DOMAIN_PREFIXES: &[&str] = &[
    "index", "main", "test", "tests", "spec", "helper", "helpers", "utils", "util", "types", "type",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DomainCluster {
    kind: DomainClusterKind,
    directory: String,
    basename_prefix: String,
    match_kind: DomainClusterMatchKind,
    prefix_path: String,
    match_count: usize,
    total_loc: Option<usize>,
    examples: Vec<DomainClusterExample>,
    omitted_count: usize,
    citations: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
enum DomainClusterKind {
    #[serde(rename = "DOMAIN_CLUSTER_DETECTED")]
    Detected,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum DomainClusterMatchKind {
    Prefix,
    DomainToken,
}

#[derive(Debug, Serialize)]
struct DomainClusterExample {
    file: String,
    loc: Option<usize>,
}

struct DomainClusterCandidate {
    display: String,
    key: String,
    token_count: usize,
}

struct DomainClusterEntry<'a> {
    file: &'a str,
    loc: Option<usize>,
}

pub(super) fn find_domain_cluster(
    intent_file: &str,
    syntax: &HealthResponse,
) -> Option<DomainCluster> {
    let dir = posix_dirname(intent_file);
    let same_dir = syntax
        .files
        .keys()
        .map(String::as_str)
        .filter(|file| *file != intent_file && posix_dirname(file) == dir)
        .map(|file| DomainClusterEntry { file, loc: None })
        .collect::<Vec<_>>();
    if same_dir.is_empty() {
        return None;
    }

    for candidate in domain_prefix_candidates(intent_file) {
        let mut matches = same_dir
            .iter()
            .filter(|entry| domain_entry_matches(entry.file, &candidate.key))
            .map(|entry| DomainClusterEntry {
                file: entry.file,
                loc: entry.loc,
            })
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| left.file.cmp(right.file));

        let prefix_match_count = matches
            .iter()
            .filter(|entry| domain_basename_key(entry.file).starts_with(&candidate.key))
            .count();
        let required_matches = if candidate.token_count >= 2 && prefix_match_count >= 1 {
            1
        } else {
            DOMAIN_CLUSTER_MIN_MATCHES
        };
        if matches.len() < required_matches {
            continue;
        }

        let loc_known = matches.iter().any(|entry| entry.loc.is_some());
        let total_loc = loc_known.then(|| matches.iter().filter_map(|entry| entry.loc).sum());
        let examples = matches
            .iter()
            .take(DOMAIN_CLUSTER_MAX_EXAMPLES)
            .map(|entry| DomainClusterExample {
                file: entry.file.to_string(),
                loc: entry.loc,
            })
            .collect::<Vec<_>>();
        let prefix_path = if dir == "." {
            candidate.display.clone()
        } else {
            format!("{dir}/{}", candidate.display)
        };

        return Some(DomainCluster {
            kind: DomainClusterKind::Detected,
            directory: dir.to_string(),
            basename_prefix: candidate.display,
            match_kind: if prefix_match_count == matches.len() {
                DomainClusterMatchKind::Prefix
            } else {
                DomainClusterMatchKind::DomainToken
            },
            prefix_path,
            match_count: matches.len(),
            total_loc,
            examples,
            omitted_count: matches.len().saturating_sub(DOMAIN_CLUSTER_MAX_EXAMPLES),
            citations: vec![format!(
                "[grounded, rust-source-health.files matched {} files with domain key '{}' in '{}']",
                matches.len(),
                candidate.key,
                dir
            )],
        });
    }

    None
}

fn domain_entry_matches(file: &str, candidate_key: &str) -> bool {
    domain_basename_key(file).starts_with(candidate_key)
        || domain_token_keys(posix_basename(file)).contains(candidate_key)
}

fn domain_prefix_candidates(intent_file: &str) -> Vec<DomainClusterCandidate> {
    let base = strip_rust_extension(posix_basename(intent_file));
    let tokens = split_name_tokens(base);
    let mut candidates = Vec::new();

    for count in (1..tokens.len()).rev() {
        let prefix_tokens = &tokens[..count];
        let display = display_prefix_from_tokens(prefix_tokens);
        let key = normalize_domain_key(&display);
        if is_usable_domain_key(&key) {
            candidates.push(DomainClusterCandidate {
                display,
                key,
                token_count: count,
            });
        }
    }

    let whole_key = normalize_domain_key(base);
    if is_usable_domain_key(&whole_key)
        && !candidates
            .iter()
            .any(|candidate| candidate.key == whole_key)
    {
        candidates.push(DomainClusterCandidate {
            display: base.to_string(),
            key: whole_key,
            token_count: tokens.len(),
        });
    }

    candidates
}

fn domain_token_keys(file_name: &str) -> BTreeSet<String> {
    split_name_tokens(strip_rust_extension(file_name))
        .into_iter()
        .map(|token| normalize_domain_token(&token))
        .filter(|key| is_usable_domain_key(key))
        .collect()
}

fn domain_basename_key(file: &str) -> String {
    normalize_domain_key(strip_rust_extension(posix_basename(file)))
}

fn is_usable_domain_key(key: &str) -> bool {
    key.len() >= DOMAIN_CLUSTER_MIN_PREFIX_LEN && !GENERIC_DOMAIN_PREFIXES.contains(&key)
}

fn strip_rust_extension(file_name: &str) -> &str {
    file_name.strip_suffix(".rs").unwrap_or(file_name)
}

fn split_name_tokens(base_name: &str) -> Vec<String> {
    let chars = base_name.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut current = String::new();

    for (index, ch) in chars.iter().copied().enumerate() {
        if matches!(ch, '-' | '_' | '.' | ' ' | '\t' | '\n' | '\r') {
            push_token(&mut tokens, &mut current);
            continue;
        }

        if let Some(previous) = current.chars().last() {
            let next = chars.get(index + 1).copied();
            let lower_to_upper = (previous.is_ascii_lowercase() || previous.is_ascii_digit())
                && ch.is_ascii_uppercase();
            let acronym_boundary = previous.is_ascii_uppercase()
                && ch.is_ascii_uppercase()
                && next.is_some_and(|next| next.is_ascii_lowercase());
            if lower_to_upper || acronym_boundary {
                push_token(&mut tokens, &mut current);
            }
        }
        current.push(ch);
    }
    push_token(&mut tokens, &mut current);

    tokens
}

fn push_token(tokens: &mut Vec<String>, current: &mut String) {
    if current.is_empty() {
        return;
    }
    tokens.push(std::mem::take(current));
}

fn normalize_domain_key(value: &str) -> String {
    normalize_domain_token(value)
}

fn normalize_domain_token(value: &str) -> String {
    let raw = value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect::<String>();
    if raw.len() > 4 && raw.ends_with("ies") {
        return format!("{}y", &raw[..raw.len() - 3]);
    }
    if raw.len() > 4 && raw.ends_with('s') {
        return raw[..raw.len() - 1].to_string();
    }
    raw
}

fn display_prefix_from_tokens(tokens: &[String]) -> String {
    let Some((first, rest)) = tokens.split_first() else {
        return String::new();
    };
    let mut display = first.clone();
    for token in rest {
        display.push_str(&capitalize_first(token));
    }
    display
}

fn capitalize_first(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut capitalized = first.to_uppercase().collect::<String>();
    capitalized.push_str(chars.as_str());
    capitalized
}

fn posix_basename(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(_, basename)| basename)
        .unwrap_or(path)
}

fn posix_dirname(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(dir, _)| if dir.is_empty() { "." } else { dir })
        .unwrap_or(".")
}
