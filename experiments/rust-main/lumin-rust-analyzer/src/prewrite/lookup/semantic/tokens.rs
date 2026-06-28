use crate::prewrite::index::Candidate;
use crate::prewrite::intent::NameDeclaration;
use crate::prewrite::tokens::{is_weak_common_token, unique_tokens};

pub(super) struct SemanticTokenMatch {
    pub(super) matched_tokens: Vec<String>,
    pub(super) matched_name_tokens: Vec<String>,
    pub(super) matched_support_tokens: Vec<String>,
    pub(super) score: usize,
}

impl SemanticTokenMatch {
    pub(super) fn all_matches_are_weak(&self) -> bool {
        self.matched_tokens
            .iter()
            .all(|token| is_weak_common_token(token))
    }

    pub(super) fn has_sufficient_non_weak_support(&self) -> bool {
        let strong_name_match_count = self
            .matched_name_tokens
            .iter()
            .filter(|token| !is_weak_common_token(token))
            .count();
        strong_name_match_count >= 2
            || (strong_name_match_count == 1 && !self.matched_support_tokens.is_empty())
    }
}

pub(super) fn semantic_token_match(
    candidate: Candidate<'_>,
    intent_tokens: &[String],
) -> Option<SemanticTokenMatch> {
    let candidate_name_tokens = unique_tokens(&[candidate.name]);
    let candidate_support_tokens = candidate_support_tokens(candidate);
    let mut candidate_tokens = candidate_name_tokens.clone();
    extend_unique(&mut candidate_tokens, &candidate_support_tokens);
    let matched_tokens = candidate_tokens
        .iter()
        .filter(|token| intent_tokens.contains(token))
        .cloned()
        .collect::<Vec<_>>();
    if matched_tokens.is_empty() {
        return None;
    }

    let matched_name_tokens = candidate_name_tokens
        .iter()
        .filter(|token| intent_tokens.contains(token))
        .cloned()
        .collect::<Vec<_>>();
    let strong_name_matches = matched_name_tokens
        .iter()
        .filter(|token| !is_weak_common_token(token))
        .cloned()
        .collect::<Vec<_>>();
    let matched_support_tokens = candidate_support_tokens
        .iter()
        .filter(|token| {
            intent_tokens.contains(token)
                && !is_weak_common_token(token)
                && !strong_name_matches.contains(token)
        })
        .cloned()
        .collect::<Vec<_>>();

    Some(SemanticTokenMatch {
        score: matched_tokens.len(),
        matched_tokens,
        matched_name_tokens,
        matched_support_tokens,
    })
}

pub(in crate::prewrite::lookup) fn query_tokens(
    intent_name: &str,
    declaration: Option<&NameDeclaration>,
) -> Vec<String> {
    unique_tokens(&[
        intent_name,
        declaration
            .and_then(|value| value.kind.as_deref())
            .unwrap_or(""),
        declaration
            .and_then(|value| value.why.as_deref())
            .unwrap_or(""),
    ])
}

fn candidate_support_tokens(candidate: Candidate<'_>) -> Vec<String> {
    let file_stem = candidate
        .file
        .rsplit('/')
        .next()
        .unwrap_or(candidate.file)
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(candidate.file);
    let owner_dir = candidate
        .file
        .rsplit_once('/')
        .map(|(directory, _)| directory)
        .unwrap_or("");
    unique_tokens(&[file_stem, owner_dir, candidate.owner_name().unwrap_or("")])
}

fn extend_unique(target: &mut Vec<String>, values: &[String]) {
    for value in values {
        if !target.contains(value) {
            target.push(value.clone());
        }
    }
}
