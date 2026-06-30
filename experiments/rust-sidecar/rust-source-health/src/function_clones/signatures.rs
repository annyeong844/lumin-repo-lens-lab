use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{
    AstFunctionCloneLine, AstFunctionSignatureGroup, AstFunctionSignatureGroupKind, AstVisibility,
    FunctionCloneRisk, RUST_FUNCTION_CLONE_MIN_GROUP_SIZE,
    RUST_FUNCTION_CLONE_SIGNATURE_MIN_DOMAIN_IDF, RUST_FUNCTION_SIGNATURE_GENERIC_TYPE_TOKENS,
    RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
};

use super::common::{
    signature_member_identity, signature_text, FunctionCloneFileView, FunctionSignatureFactView,
    SignatureMember,
};

pub(super) fn group_signature_facts<F: FunctionCloneFileView>(
    files: &BTreeMap<String, F>,
) -> Vec<AstFunctionSignatureGroup> {
    let mut by_hash = BTreeMap::<&str, Vec<SignatureMember<'_, F::SignatureFact>>>::new();
    let mut signature_count = 0usize;
    let mut document_frequency = BTreeMap::<String, usize>::new();
    for (file, health) in files {
        let generated = health.generated();
        for fact in health.function_signatures() {
            signature_count += 1;
            for token in signature_domain_type_tokens(fact) {
                *document_frequency.entry(token).or_default() += 1;
            }
            by_hash
                .entry(fact.hash())
                .or_default()
                .push(SignatureMember {
                    file: file.as_str(),
                    fact,
                    generated,
                });
        }
    }
    let type_token_idfs = signature_type_token_idfs(signature_count, document_frequency);

    let mut groups = by_hash
        .into_iter()
        .filter_map(|(hash, members)| {
            signature_group_from_members(hash.to_string(), members, &type_token_idfs)
        })
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        right
            .review_visible
            .cmp(&left.review_visible)
            .then_with(|| left.generated_only.cmp(&right.generated_only))
            .then_with(|| right.size.cmp(&left.size))
            .then_with(|| left.identities.join("|").cmp(&right.identities.join("|")))
    });
    groups
}

pub(super) fn review_visible_signature_group_count(groups: &[AstFunctionSignatureGroup]) -> usize {
    groups.iter().filter(|group| group.review_visible).count()
}

pub(super) fn signature_group_from_members<S: FunctionSignatureFactView>(
    hash: String,
    mut members: Vec<SignatureMember<'_, S>>,
    type_token_idfs: &BTreeMap<String, f64>,
) -> Option<AstFunctionSignatureGroup> {
    if members.len() < RUST_FUNCTION_CLONE_MIN_GROUP_SIZE {
        return None;
    }
    if members
        .first()
        .is_some_and(|member| member.fact.return_type().is_none())
    {
        return None;
    }
    members.sort_by_key(signature_member_identity);

    let generated_only = members.iter().all(|member| member.generated);
    let signature_domain_idf_sum = members
        .first()
        .map(|member| signature_domain_idf_sum(member.fact, type_token_idfs))
        .unwrap_or(0.0);
    let has_domain_signature =
        signature_domain_idf_sum >= RUST_FUNCTION_CLONE_SIGNATURE_MIN_DOMAIN_IDF;
    let review_visible = !generated_only && has_domain_signature;
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
    let lines = members
        .iter()
        .map(|member| AstFunctionCloneLine {
            identity: signature_member_identity(member),
            file: member.file.to_string(),
            line: member.fact.line(),
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
        risk: if has_domain_signature {
            FunctionCloneRisk::ReviewOnly
        } else {
            FunctionCloneRisk::Muted
        },
        generated_only,
        review_visible,
        signature_domain_idf_sum: round_idf_sum(signature_domain_idf_sum),
        signature: members.first().map(|member| signature_text(member.fact)),
        identities: lines.iter().map(|line| line.identity.clone()).collect(),
        owner_files,
        names,
        visibilities,
        lines,
        reason: if generated_only {
            "same normalized function type signature in generated-only files; raw evidence only"
        } else if !has_domain_signature {
            "same normalized function type signature but domain type-token IDF is below the review threshold; raw evidence only"
        } else if has_non_public {
            "same normalized function type signature; non-public helpers are review cues only; not import/reuse proof or a merge recommendation"
        } else {
            "same normalized public function type signature; review cue only; not proof of semantic equivalence or a merge recommendation"
        },
    })
}

pub(super) fn signature_type_token_idfs(
    signature_count: usize,
    document_frequency: BTreeMap<String, usize>,
) -> BTreeMap<String, f64> {
    let total = signature_count as f64;
    document_frequency
        .into_iter()
        .map(|(token, count)| {
            let idf = ((total + 1.0) / (count as f64 + 1.0)).ln();
            (token, idf)
        })
        .collect()
}

fn signature_domain_idf_sum(
    signature: &impl FunctionSignatureFactView,
    type_token_idfs: &BTreeMap<String, f64>,
) -> f64 {
    signature_domain_type_tokens(signature)
        .iter()
        .map(|token| type_token_idfs.get(token).copied().unwrap_or(0.0))
        .sum()
}

pub(super) fn signature_domain_type_tokens(
    signature: &impl FunctionSignatureFactView,
) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    if let Some(generics) = signature.generics() {
        push_domain_type_tokens(generics, &mut tokens);
    }
    if let Some(receiver) = signature.receiver_text() {
        push_domain_type_tokens(receiver, &mut tokens);
    }
    for param in signature.param_type_texts() {
        push_domain_type_tokens(param, &mut tokens);
    }
    if let Some(return_type) = signature.return_type() {
        push_domain_type_tokens(return_type, &mut tokens);
    }
    tokens
}

fn push_domain_type_tokens(text: &str, tokens: &mut BTreeSet<String>) {
    let mut token = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            token.push(ch);
        } else {
            push_domain_type_token(&mut token, tokens);
        }
    }
    push_domain_type_token(&mut token, tokens);
}

fn push_domain_type_token(token: &mut String, tokens: &mut BTreeSet<String>) {
    if token.len() >= 3 && !RUST_FUNCTION_SIGNATURE_GENERIC_TYPE_TOKENS.contains(&token.as_str()) {
        tokens.insert(std::mem::take(token));
    } else {
        token.clear();
    }
}

fn round_idf_sum(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}
