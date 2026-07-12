use std::collections::BTreeMap;

use crate::protocol::{
    AstFunctionCloneGroup, AstFunctionCloneGroupKind, RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC,
    RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS, RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC,
    RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS,
};

mod group;

pub(super) use group::group_from_members;

use super::common::{FunctionBodyFactView, FunctionCloneFileView, GroupMember};

pub(super) fn group_exact_body_groups<F: FunctionCloneFileView>(
    files: &BTreeMap<String, F>,
) -> Vec<AstFunctionCloneGroup> {
    group_by_hash(
        files,
        AstFunctionCloneGroupKind::ExactFunctionBodyGroup,
        RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC,
        RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS,
        FunctionBodyFactView::normalized_exact_hash,
        "same normalized function body; verify domain ownership before merging",
    )
}

pub(super) fn group_structure_groups<F: FunctionCloneFileView>(
    files: &BTreeMap<String, F>,
) -> Vec<AstFunctionCloneGroup> {
    group_by_hash(
        files,
        AstFunctionCloneGroupKind::FunctionBodyStructureGroup,
        RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC,
        RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS,
        FunctionBodyFactView::normalized_structure_hash,
        "same anonymized function-body structure; review cue only, not proof of semantic equivalence",
    )
}

pub(super) fn review_visible_group_count(groups: &[AstFunctionCloneGroup]) -> usize {
    groups.iter().filter(|group| !group.generated_only).count()
}

fn group_by_hash<F: FunctionCloneFileView>(
    files: &BTreeMap<String, F>,
    kind: AstFunctionCloneGroupKind,
    min_body_loc: usize,
    min_statements: usize,
    hash_for: fn(&F::BodyFact) -> &str,
    reason: &'static str,
) -> Vec<AstFunctionCloneGroup> {
    let mut by_hash = BTreeMap::<&str, Vec<GroupMember<'_, F::BodyFact>>>::new();
    for (file, health) in files {
        let generated = health.generated();
        for fact in health.function_body_fingerprints() {
            if fact.body_loc() < min_body_loc || fact.statement_count() < min_statements {
                continue;
            }
            by_hash
                .entry(hash_for(fact))
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
        .filter_map(|(hash, members)| group_from_members(kind, hash.to_string(), members, reason))
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
