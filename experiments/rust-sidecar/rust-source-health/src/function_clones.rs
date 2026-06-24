use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{
    AstFunctionBodyFingerprint, AstFunctionCloneGroup, AstFunctionCloneGroupKind,
    AstFunctionCloneGroups, AstFunctionCloneInputError, AstFunctionCloneLine, AstFunctionParam,
    AstFunctionReceiver, AstFunctionSignature, AstFunctionSignatureGroup,
    AstFunctionSignatureGroupKind, AstNearFunctionCandidate, AstNearFunctionCandidateKind,
    AstVisibility, FileHealth, FunctionCloneRisk, PathClassification, SkippedFile,
    SkippedFileReason, RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC,
    RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS, RUST_FUNCTION_CLONE_MIN_GROUP_SIZE,
    RUST_FUNCTION_CLONE_NEAR_BODY_LOC_WEIGHT, RUST_FUNCTION_CLONE_NEAR_CALL_TOKEN_WEIGHT,
    RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES, RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA,
    RUST_FUNCTION_CLONE_NEAR_MIN_BODY_LOC_SIMILARITY,
    RUST_FUNCTION_CLONE_NEAR_MIN_CALL_TOKEN_JACCARD,
    RUST_FUNCTION_CLONE_NEAR_MIN_NAME_TOKEN_JACCARD_FALLBACK, RUST_FUNCTION_CLONE_NEAR_MIN_SCORE,
    RUST_FUNCTION_CLONE_NEAR_MIN_SIGNIFICANT_CALL_TOKEN_LEN,
    RUST_FUNCTION_CLONE_NEAR_MIN_STATEMENT_COUNT_SIMILARITY,
    RUST_FUNCTION_CLONE_NEAR_NAME_TOKEN_WEIGHT, RUST_FUNCTION_CLONE_NEAR_STATEMENT_COUNT_WEIGHT,
    RUST_FUNCTION_CLONE_NEAR_SUPPRESSED_GENERIC_CALL_TOKENS,
    RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC, RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS,
};

pub(crate) fn group_function_body_fingerprints(
    files: &BTreeMap<String, FileHealth>,
    skipped_files: &[SkippedFile],
) -> AstFunctionCloneGroups {
    let exact_body_groups = group_by_hash(
        files,
        AstFunctionCloneGroupKind::ExactFunctionBodyGroup,
        RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC,
        RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS,
        |fact| &fact.normalized_exact_hash,
        "same normalized function body; verify domain ownership before merging",
    );
    let structure_groups = group_by_hash(
        files,
        AstFunctionCloneGroupKind::FunctionBodyStructureGroup,
        RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC,
        RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS,
        |fact| &fact.normalized_structure_hash,
        "same anonymized function-body structure; review cue only, not proof of semantic equivalence",
    );
    let signature_groups = group_signature_facts(files);
    let near_function_candidates =
        build_near_function_candidates(files, &exact_body_groups, &structure_groups);
    let files_with_parse_errors = files_with_parse_errors(files);
    let files_with_read_errors = files_with_read_errors(skipped_files);
    let complete = files_with_parse_errors.is_empty() && files_with_read_errors.is_empty();

    AstFunctionCloneGroups {
        complete,
        files_with_parse_errors,
        files_with_read_errors,
        exact_body_group_count: review_visible_group_count(&exact_body_groups),
        structure_group_count: review_visible_group_count(&structure_groups),
        signature_group_count: review_visible_signature_group_count(&signature_groups),
        near_function_candidate_count: near_function_candidates.review_visible_count,
        near_function_candidate_projection_limit: RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES,
        generated_file_fact_count: generated_file_fact_count(files),
        exact_body_groups,
        structure_groups,
        signature_groups,
        near_function_candidates: near_function_candidates.candidates,
        ..AstFunctionCloneGroups::default()
    }
}

fn files_with_parse_errors(
    files: &BTreeMap<String, FileHealth>,
) -> Vec<AstFunctionCloneInputError> {
    files
        .iter()
        .filter_map(|(file, health)| {
            if health.parse.ok {
                return None;
            }
            Some(AstFunctionCloneInputError {
                file: file.clone(),
                message: health
                    .parse
                    .errors
                    .first()
                    .map(|error| error.message.clone())
                    .unwrap_or_else(|| "parse error".to_string()),
            })
        })
        .collect()
}

fn files_with_read_errors(skipped_files: &[SkippedFile]) -> Vec<AstFunctionCloneInputError> {
    skipped_files
        .iter()
        .map(|file| AstFunctionCloneInputError {
            file: file.path.clone(),
            message: skipped_file_reason_message(file.reason).to_string(),
        })
        .collect()
}

fn skipped_file_reason_message(reason: SkippedFileReason) -> &'static str {
    match reason {
        SkippedFileReason::InvalidUtf8 => "invalid-utf8",
    }
}

fn review_visible_group_count(groups: &[AstFunctionCloneGroup]) -> usize {
    groups.iter().filter(|group| !group.generated_only).count()
}

fn review_visible_signature_group_count(groups: &[AstFunctionSignatureGroup]) -> usize {
    groups.iter().filter(|group| !group.generated_only).count()
}

fn generated_file_fact_count(files: &BTreeMap<String, FileHealth>) -> usize {
    files
        .values()
        .filter(|health| {
            health
                .path
                .classifications
                .contains(&PathClassification::Generated)
        })
        .map(|health| health.ast.function_body_fingerprints.len())
        .sum()
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

fn group_signature_facts(files: &BTreeMap<String, FileHealth>) -> Vec<AstFunctionSignatureGroup> {
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

fn signature_group_from_members(
    hash: String,
    mut members: Vec<SignatureMember<'_>>,
) -> Option<AstFunctionSignatureGroup> {
    if members.len() < RUST_FUNCTION_CLONE_MIN_GROUP_SIZE {
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

fn build_near_function_candidates(
    files: &BTreeMap<String, FileHealth>,
    exact_body_groups: &[AstFunctionCloneGroup],
    structure_groups: &[AstFunctionCloneGroup],
) -> NearFunctionCandidateProjection {
    let grouped = grouped_identity_set(exact_body_groups, structure_groups);
    let mut eligible = function_members(files)
        .into_iter()
        .filter_map(|member| {
            let identity = member_identity(&member);
            if grouped.contains(&identity) {
                return None;
            }
            let significant_call_tokens = significant_call_tokens(member.fact);
            (!significant_call_tokens.is_empty()).then(|| NearFact {
                member,
                identity,
                significant_call_tokens,
                name_tokens: name_tokens(&member.fact.name),
            })
        })
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| left.identity.cmp(&right.identity));

    let mut by_call_token = BTreeMap::<&str, Vec<usize>>::new();
    for (index, fact) in eligible.iter().enumerate() {
        for token in &fact.significant_call_tokens {
            by_call_token.entry(token.as_str()).or_default().push(index);
        }
    }

    let mut pair_keys = BTreeSet::<(usize, usize)>::new();
    let mut candidates = Vec::new();
    for bucket in by_call_token.values() {
        for (left_offset, left_index) in bucket.iter().enumerate() {
            for right_index in bucket.iter().skip(left_offset + 1) {
                let pair_key = (*left_index, *right_index);
                if !pair_keys.insert(pair_key) {
                    continue;
                }
                if let Some(candidate) =
                    near_candidate_from_pair(&eligible[*left_index], &eligible[*right_index])
                {
                    candidates.push(candidate);
                }
            }
        }
    }

    let review_visible_count = candidates
        .iter()
        .filter(|candidate| !candidate.generated_only)
        .count();

    candidates.sort_by(|left, right| {
        left.generated_only
            .cmp(&right.generated_only)
            .then_with(|| right.score.total_cmp(&left.score))
            .then_with(|| left.identities.join("|").cmp(&right.identities.join("|")))
    });
    candidates.truncate(RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES);
    NearFunctionCandidateProjection {
        review_visible_count,
        candidates,
    }
}

fn function_members(files: &BTreeMap<String, FileHealth>) -> Vec<GroupMember<'_>> {
    files
        .iter()
        .flat_map(|(file, health)| {
            let generated = health
                .path
                .classifications
                .contains(&PathClassification::Generated);
            health
                .ast
                .function_body_fingerprints
                .iter()
                .map(move |fact| GroupMember {
                    file: file.as_str(),
                    fact,
                    generated,
                })
        })
        .collect()
}

fn near_candidate_from_pair(
    left: &NearFact<'_>,
    right: &NearFact<'_>,
) -> Option<AstNearFunctionCandidate> {
    if left.member.fact.is_async != right.member.fact.is_async
        || left.member.fact.is_unsafe != right.member.fact.is_unsafe
        || left.member.fact.is_const != right.member.fact.is_const
    {
        return None;
    }
    if left
        .member
        .fact
        .param_count
        .abs_diff(right.member.fact.param_count)
        > RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA
    {
        return None;
    }

    let shared_call_tokens = sorted_intersection(
        &left.significant_call_tokens,
        &right.significant_call_tokens,
    );
    if shared_call_tokens.is_empty() {
        return None;
    }

    let call_token_jaccard = jaccard(
        &left.significant_call_tokens,
        &right.significant_call_tokens,
    );
    let shared_name_tokens = sorted_intersection(&left.name_tokens, &right.name_tokens);
    let name_token_jaccard = jaccard(&left.name_tokens, &right.name_tokens);
    let body_loc_similarity =
        range_similarity(left.member.fact.body_loc, right.member.fact.body_loc);
    let statement_count_similarity = range_similarity(
        left.member.fact.statement_count,
        right.member.fact.statement_count,
    );
    if body_loc_similarity < RUST_FUNCTION_CLONE_NEAR_MIN_BODY_LOC_SIMILARITY
        || statement_count_similarity < RUST_FUNCTION_CLONE_NEAR_MIN_STATEMENT_COUNT_SIMILARITY
    {
        return None;
    }
    if call_token_jaccard < RUST_FUNCTION_CLONE_NEAR_MIN_CALL_TOKEN_JACCARD
        && name_token_jaccard < RUST_FUNCTION_CLONE_NEAR_MIN_NAME_TOKEN_JACCARD_FALLBACK
    {
        return None;
    }

    let score = round_score(
        (call_token_jaccard * RUST_FUNCTION_CLONE_NEAR_CALL_TOKEN_WEIGHT)
            + (name_token_jaccard * RUST_FUNCTION_CLONE_NEAR_NAME_TOKEN_WEIGHT)
            + (body_loc_similarity * RUST_FUNCTION_CLONE_NEAR_BODY_LOC_WEIGHT)
            + (statement_count_similarity * RUST_FUNCTION_CLONE_NEAR_STATEMENT_COUNT_WEIGHT),
    );
    if score < RUST_FUNCTION_CLONE_NEAR_MIN_SCORE {
        return None;
    }

    let mut pair = [left, right];
    pair.sort_by(|a, b| a.identity.cmp(&b.identity));
    let lines = pair
        .iter()
        .map(|fact| AstFunctionCloneLine {
            identity: fact.identity.clone(),
            file: fact.member.file.to_string(),
            line: fact.member.fact.location.line,
        })
        .collect::<Vec<_>>();
    let body_locs = pair
        .iter()
        .map(|fact| fact.member.fact.body_loc)
        .collect::<Vec<_>>();
    let statement_counts = pair
        .iter()
        .map(|fact| fact.member.fact.statement_count)
        .collect::<Vec<_>>();
    let mut reasons = vec![
        format!(
            "shared significant call tokens: {}",
            shared_call_tokens.join(", ")
        ),
        format!(
            "body size similarity: {}",
            format_score(body_loc_similarity)
        ),
        format!(
            "statement-count similarity: {}",
            format_score(statement_count_similarity)
        ),
    ];
    if !shared_name_tokens.is_empty() {
        reasons.push(format!(
            "shared exported-name tokens: {}",
            shared_name_tokens.join(", ")
        ));
    }

    Some(AstNearFunctionCandidate {
        kind: AstNearFunctionCandidateKind::NearFunctionCandidate,
        identities: pair.iter().map(|fact| fact.identity.clone()).collect(),
        owner_files: pair
            .iter()
            .map(|fact| fact.member.file.to_string())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        names: pair
            .iter()
            .map(|fact| fact.member.fact.name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        lines,
        score,
        risk: FunctionCloneRisk::ReviewOnly,
        generated_only: pair.iter().all(|fact| fact.member.generated),
        shared_call_tokens,
        shared_name_tokens,
        call_token_jaccard: round_score(call_token_jaccard),
        name_token_jaccard: round_score(name_token_jaccard),
        body_loc_range: [
            body_locs.iter().copied().min().unwrap_or(0),
            body_locs.iter().copied().max().unwrap_or(0),
        ],
        statement_count_range: [
            statement_counts.iter().copied().min().unwrap_or(0),
            statement_counts.iter().copied().max().unwrap_or(0),
        ],
        reasons,
        reason: "near function cue only; source review required; not proof of semantic equivalence or an automatic merge",
    })
}

fn grouped_identity_set(
    exact_body_groups: &[AstFunctionCloneGroup],
    structure_groups: &[AstFunctionCloneGroup],
) -> BTreeSet<String> {
    exact_body_groups
        .iter()
        .chain(structure_groups)
        .flat_map(|group| group.identities.iter().cloned())
        .collect()
}

fn significant_call_tokens(fact: &AstFunctionBodyFingerprint) -> Vec<String> {
    fact.call_tokens
        .iter()
        .filter(|token| {
            token.len() >= RUST_FUNCTION_CLONE_NEAR_MIN_SIGNIFICANT_CALL_TOKEN_LEN
                && !RUST_FUNCTION_CLONE_NEAR_SUPPRESSED_GENERIC_CALL_TOKENS
                    .contains(&token.as_str())
        })
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn name_tokens(name: &str) -> Vec<String> {
    let mut expanded = String::new();
    let mut previous_lower_or_digit = false;
    for ch in name.chars() {
        if ch.is_ascii_uppercase() && previous_lower_or_digit {
            expanded.push(' ');
        }
        expanded.push(ch);
        previous_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
    }
    expanded
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(str::trim)
        .filter(|token| token.len() >= 2)
        .map(str::to_ascii_lowercase)
        .collect()
}

fn sorted_intersection(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().map(String::as_str).collect::<BTreeSet<_>>();
    left.iter()
        .filter(|entry| right.contains(entry.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn jaccard(left: &[String], right: &[String]) -> f64 {
    let left = left.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let right = right.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let union = left.union(&right).count();
    if union == 0 {
        return 0.0;
    }
    left.intersection(&right).count() as f64 / union as f64
}

fn range_similarity(left: usize, right: usize) -> f64 {
    let max = left.max(right);
    if max == 0 {
        return 0.0;
    }
    1.0 - (left.abs_diff(right) as f64 / max as f64)
}

fn round_score(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn format_score(value: f64) -> String {
    let rounded = round_score(value);
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        rounded.to_string()
    }
}

fn member_identity(member: &GroupMember<'_>) -> String {
    match &member.fact.owner {
        None => format!("{}::{}", member.file, member.fact.name),
        Some(owner) => match &owner.trait_path {
            None => format!("{}::{}#{}", member.file, owner.target, member.fact.name),
            Some(trait_path) => format!(
                "{}::{} as {}#{}",
                member.file, owner.target, trait_path, member.fact.name
            ),
        },
    }
}

fn signature_member_identity(member: &SignatureMember<'_>) -> String {
    match &member.fact.owner {
        None => format!("{}::{}", member.file, member.fact.name),
        Some(owner) => match &owner.trait_path {
            None => format!("{}::{}#{}", member.file, owner.target, member.fact.name),
            Some(trait_path) => format!(
                "{}::{} as {}#{}",
                member.file, owner.target, trait_path, member.fact.name
            ),
        },
    }
}

fn signature_text(signature: &AstFunctionSignature) -> String {
    let mut params = Vec::new();
    if let Some(receiver) = &signature.receiver {
        params.push(receiver_text(receiver));
    }
    params.extend(signature.params.iter().map(param_text));

    let mut text = String::from("fn");
    if let Some(generics) = &signature.generics {
        text.push_str(generics);
    }
    text.push('(');
    text.push_str(&params.join(", "));
    text.push(')');
    if let Some(return_type) = &signature.return_type {
        text.push_str(" -> ");
        text.push_str(return_type);
    }
    text
}

fn receiver_text(receiver: &AstFunctionReceiver) -> String {
    receiver.text.clone()
}

fn param_text(param: &AstFunctionParam) -> String {
    param.type_text.clone()
}

#[derive(Clone, Copy)]
struct GroupMember<'a> {
    file: &'a str,
    fact: &'a AstFunctionBodyFingerprint,
    generated: bool,
}

#[derive(Clone, Copy)]
struct SignatureMember<'a> {
    file: &'a str,
    fact: &'a AstFunctionSignature,
    generated: bool,
}

struct NearFact<'a> {
    member: GroupMember<'a>,
    identity: String,
    significant_call_tokens: Vec<String>,
    name_tokens: Vec<String>,
}

struct NearFunctionCandidateProjection {
    review_visible_count: usize,
    candidates: Vec<AstNearFunctionCandidate>,
}
