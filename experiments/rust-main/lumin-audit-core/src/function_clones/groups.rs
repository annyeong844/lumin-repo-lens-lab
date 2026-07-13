use super::facts::FunctionFact;
use super::near::{MIN_BODY_LOC_FOR_GROUPING, MIN_GROUP_SIZE, MIN_STATEMENTS_FOR_GROUPING};
use super::projection::{line_json, shared_call_tokens, sort_clone_groups, sorted_unique};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy)]
enum GroupKey {
    NormalizedExact,
    NormalizedStructure,
}

pub(super) fn exact_body_groups(facts: &[FunctionFact]) -> Vec<Value> {
    group_facts(facts, GroupKey::NormalizedExact, 1, 1, MIN_GROUP_SIZE)
}

pub(super) fn structure_groups(facts: &[FunctionFact]) -> Vec<Value> {
    group_facts(
        facts,
        GroupKey::NormalizedStructure,
        MIN_BODY_LOC_FOR_GROUPING,
        MIN_STATEMENTS_FOR_GROUPING,
        MIN_GROUP_SIZE,
    )
}

pub(super) fn signature_groups(facts: &[FunctionFact]) -> Vec<Value> {
    group_signature_facts(facts, MIN_GROUP_SIZE)
}

fn group_facts(
    facts: &[FunctionFact],
    key: GroupKey,
    min_body_loc: usize,
    min_statements: usize,
    min_size: usize,
) -> Vec<Value> {
    let mut by_hash = BTreeMap::<String, Vec<&FunctionFact>>::new();
    for fact in facts {
        let hash = match key {
            GroupKey::NormalizedExact => &fact.normalized_exact_hash,
            GroupKey::NormalizedStructure => &fact.normalized_structure_hash,
        };
        if hash.is_empty() {
            continue;
        }
        if fact.body_loc < min_body_loc as i64 || fact.statement_count < min_statements as i64 {
            continue;
        }
        by_hash.entry(hash.to_string()).or_default().push(fact);
    }

    let mut groups = Vec::<Value>::new();
    for (group_hash, members) in by_hash {
        if members.len() < min_size {
            continue;
        }
        let mut sorted = members;
        sorted.sort_by(|left, right| left.identity.cmp(&right.identity));
        let generated_only = sorted.iter().all(|fact| fact.generated_file);
        let exact_hash_count = sorted
            .iter()
            .map(|fact| fact.normalized_exact_hash.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let shared_call_tokens = shared_call_tokens(&sorted);
        let body_loc_values = sorted.iter().map(|fact| fact.body_loc).collect::<Vec<_>>();

        groups.push(json!({
            "hash": group_hash,
            "size": sorted.len(),
            "generatedOnly": generated_only,
            "exactHashCount": exact_hash_count,
            "identities": sorted.iter().map(|fact| fact.identity.clone()).collect::<Vec<_>>(),
            "ownerFiles": sorted_unique(sorted.iter().map(|fact| fact.owner_file.clone())),
            "exportedNames": sorted_unique(sorted.iter().map(|fact| fact.exported_name.clone())),
            "visibilities": sorted_unique(sorted.iter().map(|fact| fact.visibility.clone())),
            "lines": sorted.iter().copied().map(line_json).collect::<Vec<_>>(),
            "bodyLocRange": [
                body_loc_values.iter().min().copied().unwrap_or(0),
                body_loc_values.iter().max().copied().unwrap_or(0),
            ],
            "sharedCallTokens": shared_call_tokens,
            "reason": match key {
                GroupKey::NormalizedExact => "same normalized function body; verify domain ownership before merging",
                GroupKey::NormalizedStructure => "same anonymized function-body structure; review cue only, not proof of semantic equivalence",
            },
        }));
    }

    sort_clone_groups(&mut groups, true);
    groups
}

fn group_signature_facts(facts: &[FunctionFact], min_size: usize) -> Vec<Value> {
    let mut by_hash = BTreeMap::<String, Vec<&FunctionFact>>::new();
    for fact in facts {
        let Some(hash) = &fact.normalized_signature_hash else {
            continue;
        };
        if hash.is_empty() {
            continue;
        }
        by_hash.entry(hash.clone()).or_default().push(fact);
    }

    let mut groups = Vec::<Value>::new();
    for (signature_hash, members) in by_hash {
        if members.len() < min_size {
            continue;
        }
        let mut sorted = members;
        sorted.sort_by(|left, right| left.identity.cmp(&right.identity));
        let generated_only = sorted.iter().all(|fact| fact.generated_file);
        let visibilities = sorted_unique(sorted.iter().map(|fact| fact.visibility.clone()));
        let has_file_local = visibilities
            .iter()
            .any(|visibility| visibility == "file-local");
        groups.push(json!({
            "kind": "function-signature-group",
            "hash": signature_hash,
            "size": sorted.len(),
            "generatedOnly": generated_only,
            "risk": "review-only",
            "signature": sorted.first().and_then(|fact| fact.signature.clone()).unwrap_or(Value::Null),
            "identities": sorted.iter().map(|fact| fact.identity.clone()).collect::<Vec<_>>(),
            "ownerFiles": sorted_unique(sorted.iter().map(|fact| fact.owner_file.clone())),
            "exportedNames": sorted_unique(sorted.iter().map(|fact| fact.exported_name.clone())),
            "visibilities": visibilities,
            "lines": sorted.iter().copied().map(line_json).collect::<Vec<_>>(),
            "reason": if has_file_local {
                "same normalized function type signature; file-local helpers are review cues only; not import/reuse proof or a merge recommendation"
            } else {
                "same normalized exported function type signature; review cue only; not proof of semantic equivalence or a merge recommendation"
            },
        }));
    }

    sort_clone_groups(&mut groups, false);
    groups
}
