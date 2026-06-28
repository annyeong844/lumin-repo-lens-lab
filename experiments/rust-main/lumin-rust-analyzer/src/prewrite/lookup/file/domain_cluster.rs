use lumin_rust_source_health::protocol::HealthResponse;

use candidates::{domain_entry_matches, domain_prefix_candidates};
pub(super) use model::DomainCluster;
use model::{DomainClusterEntry, DomainClusterExample, DomainClusterKind, DomainClusterMatchKind};
use path::posix_dirname;

mod candidates;
mod model;
mod path;
mod tokens;

// Mirrors _lib/pre-write-lookup-file.mjs domain cluster policy.
pub(in crate::prewrite) const DOMAIN_CLUSTER_MIN_MATCHES: usize = 2;
pub(in crate::prewrite) const DOMAIN_CLUSTER_MAX_EXAMPLES: usize = 8;
pub(in crate::prewrite) const DOMAIN_CLUSTER_MIN_PREFIX_LEN: usize = 4;

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
            .filter(|entry| candidates::domain_basename_key(entry.file).starts_with(&candidate.key))
            .count();
        let required_matches = if candidate.token_count >= 2 && prefix_match_count >= 1 {
            1
        } else {
            DOMAIN_CLUSTER_MIN_MATCHES
        };
        if matches.len() < required_matches {
            continue;
        }

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
            total_loc: total_loc(&matches),
            examples: examples(&matches),
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

fn total_loc(matches: &[DomainClusterEntry<'_>]) -> Option<usize> {
    matches
        .iter()
        .any(|entry| entry.loc.is_some())
        .then(|| matches.iter().filter_map(|entry| entry.loc).sum())
}

fn examples(matches: &[DomainClusterEntry<'_>]) -> Vec<DomainClusterExample> {
    matches
        .iter()
        .take(DOMAIN_CLUSTER_MAX_EXAMPLES)
        .map(|entry| DomainClusterExample {
            file: entry.file.to_string(),
            loc: entry.loc,
        })
        .collect()
}
