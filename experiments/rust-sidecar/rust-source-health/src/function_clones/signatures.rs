use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{
    AstFunctionCloneLine, AstFunctionSignatureGroup, AstFunctionSignatureGroupKind, AstVisibility,
    FileHealth, FunctionCloneRisk, PathClassification, RUST_FUNCTION_CLONE_MIN_GROUP_SIZE,
    RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
};

use super::common::{signature_member_identity, signature_text, SignatureMember};

pub(super) fn group_signature_facts(
    files: &BTreeMap<String, FileHealth>,
) -> Vec<AstFunctionSignatureGroup> {
    let mut by_hash = BTreeMap::<String, Vec<SignatureMember<'_>>>::new();
    for (file, health) in files {
        let generated = health
            .path
            .classifications
            .contains(&PathClassification::Generated);
        for fact in &health.ast.function_signatures {
            by_hash
                .entry(fact.hash.clone())
                .or_default()
                .push(SignatureMember {
                    file: file.as_str(),
                    fact,
                    generated,
                });
        }
    }

    let mut groups = by_hash
        .into_iter()
        .filter_map(|(hash, members)| signature_group_from_members(hash, members))
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        left.generated_only
            .cmp(&right.generated_only)
            .then_with(|| right.size.cmp(&left.size))
            .then_with(|| left.identities.join("|").cmp(&right.identities.join("|")))
    });
    groups
}

pub(super) fn review_visible_signature_group_count(groups: &[AstFunctionSignatureGroup]) -> usize {
    groups.iter().filter(|group| !group.generated_only).count()
}

fn signature_group_from_members(
    hash: String,
    mut members: Vec<SignatureMember<'_>>,
) -> Option<AstFunctionSignatureGroup> {
    if members.len() < RUST_FUNCTION_CLONE_MIN_GROUP_SIZE {
        return None;
    }
    if members
        .first()
        .is_some_and(|member| member.fact.return_type.is_none())
    {
        return None;
    }
    members.sort_by_key(signature_member_identity);

    let generated_only = members.iter().all(|member| member.generated);
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
    let lines = members
        .iter()
        .map(|member| AstFunctionCloneLine {
            identity: signature_member_identity(member),
            file: member.file.to_string(),
            line: member.fact.location.line,
        })
        .collect::<Vec<_>>();
    let has_non_public = visibilities
        .iter()
        .any(|visibility| *visibility != AstVisibility::Public);

    Some(AstFunctionSignatureGroup {
        kind: AstFunctionSignatureGroupKind::FunctionSignatureGroup,
        normalized_version: RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
        hash,
        size: members.len(),
        risk: FunctionCloneRisk::ReviewOnly,
        generated_only,
        signature: members.first().map(|member| signature_text(member.fact)),
        identities: lines.iter().map(|line| line.identity.clone()).collect(),
        owner_files,
        names,
        visibilities,
        lines,
        reason: if has_non_public {
            "same normalized function type signature; non-public helpers are review cues only; not import/reuse proof or a merge recommendation"
        } else {
            "same normalized public function type signature; review cue only; not proof of semantic equivalence or a merge recommendation"
        },
    })
}
