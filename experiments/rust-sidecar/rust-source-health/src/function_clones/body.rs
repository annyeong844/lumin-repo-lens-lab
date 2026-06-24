use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{
    AstFunctionBodyFingerprint, AstFunctionCloneGroup, AstFunctionCloneGroupKind,
    AstFunctionCloneLine, FileHealth, FunctionCloneRisk, PathClassification,
    RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC, RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS,
    RUST_FUNCTION_CLONE_MIN_GROUP_SIZE, RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC,
    RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS,
};

use super::common::{member_identity, GroupMember};

pub(super) fn group_exact_body_groups(
    files: &BTreeMap<String, FileHealth>,
) -> Vec<AstFunctionCloneGroup> {
    group_by_hash(
        files,
        AstFunctionCloneGroupKind::ExactFunctionBodyGroup,
        RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC,
        RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS,
        |fact| &fact.normalized_exact_hash,
        "same normalized function body; verify domain ownership before merging",
    )
}

pub(super) fn group_structure_groups(
    files: &BTreeMap<String, FileHealth>,
) -> Vec<AstFunctionCloneGroup> {
    group_by_hash(
        files,
        AstFunctionCloneGroupKind::FunctionBodyStructureGroup,
        RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC,
        RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS,
        |fact| &fact.normalized_structure_hash,
        "same anonymized function-body structure; review cue only, not proof of semantic equivalence",
    )
}

pub(super) fn review_visible_group_count(groups: &[AstFunctionCloneGroup]) -> usize {
    groups.iter().filter(|group| !group.generated_only).count()
}

fn group_by_hash(
    files: &BTreeMap<String, FileHealth>,
    kind: AstFunctionCloneGroupKind,
    min_body_loc: usize,
    min_statements: usize,
    hash_for: fn(&AstFunctionBodyFingerprint) -> &String,
    reason: &'static str,
) -> Vec<AstFunctionCloneGroup> {
    let mut by_hash = BTreeMap::<String, Vec<GroupMember<'_>>>::new();
    for (file, health) in files {
        let generated = health
            .path
            .classifications
            .contains(&PathClassification::Generated);
        for fact in &health.ast.function_body_fingerprints {
            if fact.body_loc < min_body_loc || fact.statement_count < min_statements {
                continue;
            }
            by_hash
                .entry(hash_for(fact).clone())
                .or_default()
                .push(GroupMember {
                    file: file.as_str(),
                    fact,
                    generated,
                });
        }
    }

    let mut groups = by_hash
        .into_iter()
        .filter_map(|(hash, members)| group_from_members(kind, hash, members, reason))
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        left.generated_only
            .cmp(&right.generated_only)
            .then_with(|| right.size.cmp(&left.size))
            .then_with(|| right.body_loc_range[1].cmp(&left.body_loc_range[1]))
            .then_with(|| left.identities.join("|").cmp(&right.identities.join("|")))
    });
    groups
}

fn group_from_members(
    kind: AstFunctionCloneGroupKind,
    hash: String,
    mut members: Vec<GroupMember<'_>>,
    reason: &'static str,
) -> Option<AstFunctionCloneGroup> {
    if members.len() < RUST_FUNCTION_CLONE_MIN_GROUP_SIZE {
        return None;
    }
    members.sort_by_key(member_identity);

    let generated_only = members.iter().all(|member| member.generated);
    let exact_hash_count = members
        .iter()
        .map(|member| member.fact.exact_body_hash.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let owner_files = members
        .iter()
        .map(|member| member.file.to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let names = members
        .iter()
        .map(|member| member.fact.name.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let visibilities = members
        .iter()
        .map(|member| member.fact.visibility)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let body_locs = members
        .iter()
        .map(|member| member.fact.body_loc)
        .collect::<Vec<_>>();
    let lines = members
        .iter()
        .map(|member| AstFunctionCloneLine {
            identity: member_identity(member),
            file: member.file.to_string(),
            line: member.fact.location.line,
        })
        .collect::<Vec<_>>();
    let shared_call_tokens = shared_call_tokens(&members);

    Some(AstFunctionCloneGroup {
        kind,
        hash,
        size: members.len(),
        risk: FunctionCloneRisk::ReviewOnly,
        generated_only,
        exact_hash_count,
        identities: lines.iter().map(|line| line.identity.clone()).collect(),
        owner_files,
        names,
        visibilities,
        lines,
        body_loc_range: [
            body_locs.iter().copied().min().unwrap_or(0),
            body_locs.iter().copied().max().unwrap_or(0),
        ],
        shared_call_tokens,
        reason,
    })
}

fn shared_call_tokens(members: &[GroupMember<'_>]) -> Vec<String> {
    let Some((first, rest)) = members.split_first() else {
        return Vec::new();
    };
    let mut shared = first
        .fact
        .call_tokens
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for member in rest {
        let tokens = member
            .fact
            .call_tokens
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        shared = shared.intersection(&tokens).cloned().collect();
    }
    shared.into_iter().collect()
}
