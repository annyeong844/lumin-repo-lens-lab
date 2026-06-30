use std::collections::BTreeSet;

use crate::protocol::{
    AstFunctionCloneGroup, AstFunctionCloneGroupKind, AstFunctionCloneLine, FunctionCloneRisk,
    RUST_FUNCTION_CLONE_MIN_GROUP_SIZE,
};

use super::super::common::{member_identity, FunctionBodyFactView, GroupMember};

pub(in crate::function_clones) fn group_from_members<B: FunctionBodyFactView>(
    kind: AstFunctionCloneGroupKind,
    hash: String,
    mut members: Vec<GroupMember<'_, B>>,
    reason: &'static str,
) -> Option<AstFunctionCloneGroup> {
    if members.len() < RUST_FUNCTION_CLONE_MIN_GROUP_SIZE {
        return None;
    }
    members.sort_by_key(member_identity);

    let generated_only = members.iter().all(|member| member.generated);
    let exact_hash_count = members
        .iter()
        .map(|member| member.fact.exact_body_hash())
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
        .map(|member| member.fact.name().to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let visibilities = members
        .iter()
        .map(|member| member.fact.visibility())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let body_locs = members
        .iter()
        .map(|member| member.fact.body_loc())
        .collect::<Vec<_>>();
    let lines = members
        .iter()
        .map(|member| AstFunctionCloneLine {
            identity: member_identity(member),
            file: member.file.to_string(),
            line: member.fact.line(),
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

fn shared_call_tokens<B: FunctionBodyFactView>(members: &[GroupMember<'_, B>]) -> Vec<String> {
    let Some((first, rest)) = members.split_first() else {
        return Vec::new();
    };
    let mut shared = first
        .fact
        .call_tokens()
        .iter()
        .map(AsRef::as_ref)
        .collect::<BTreeSet<_>>();
    for member in rest {
        let tokens = member
            .fact
            .call_tokens()
            .iter()
            .map(AsRef::as_ref)
            .collect::<BTreeSet<_>>();
        shared = shared.intersection(&tokens).copied().collect();
    }
    shared.into_iter().map(str::to_string).collect()
}
