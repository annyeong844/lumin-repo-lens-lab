use std::collections::BTreeSet;

use super::model::DomainClusterCandidate;
use super::path::posix_basename;
use super::tokens::{
    display_prefix_from_tokens, is_usable_domain_key, normalize_domain_key, normalize_domain_token,
    split_name_tokens,
};

pub(super) fn domain_entry_matches(file: &str, candidate_key: &str) -> bool {
    domain_basename_key(file).starts_with(candidate_key)
        || domain_token_keys(posix_basename(file)).contains(candidate_key)
}

pub(super) fn domain_prefix_candidates(intent_file: &str) -> Vec<DomainClusterCandidate> {
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

pub(super) fn domain_basename_key(file: &str) -> String {
    normalize_domain_key(strip_rust_extension(posix_basename(file)))
}

fn domain_token_keys(file_name: &str) -> BTreeSet<String> {
    split_name_tokens(strip_rust_extension(file_name))
        .into_iter()
        .map(|token| normalize_domain_token(&token))
        .filter(|key| is_usable_domain_key(key))
        .collect()
}

fn strip_rust_extension(file_name: &str) -> &str {
    file_name.strip_suffix(".rs").unwrap_or(file_name)
}
