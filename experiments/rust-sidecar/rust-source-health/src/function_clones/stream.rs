use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{
    AstFunctionCloneGroup, AstFunctionCloneGroupKind, AstFunctionCloneGroups,
    AstFunctionCloneInputError, AstFunctionOwner, AstFunctionSignatureGroup,
    AstNearFunctionCandidateGenerationSummary, AstNearFunctionCompatibilitySkippedPairEstimates,
    AstVisibility, RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC,
    RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS, RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES,
    RUST_FUNCTION_CLONE_NEAR_SKIPPED_PAIR_ESTIMATE_KIND,
    RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC, RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS,
};

use super::common::{
    FunctionBodyFactView, FunctionCloneFile, FunctionCloneFileView, FunctionSignatureFactView,
    GroupMember, SignatureMember,
};

pub(crate) struct FunctionCloneAccumulator {
    files: Vec<Box<str>>,
    files_with_parse_errors: Vec<AstFunctionCloneInputError>,
    generated_file_fact_count: usize,
    body_members: Vec<OwnedBodyMember>,
    exact_by_hash: BTreeMap<Box<str>, Vec<usize>>,
    structure_by_hash: BTreeMap<Box<str>, Vec<usize>>,
    signature_members: Vec<OwnedSignatureMember>,
    signatures_by_hash: BTreeMap<Box<str>, Vec<usize>>,
    signature_count: usize,
    signature_document_frequency: BTreeMap<String, usize>,
}

impl FunctionCloneAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            files: Vec::new(),
            files_with_parse_errors: Vec::new(),
            generated_file_fact_count: 0,
            body_members: Vec::new(),
            exact_by_hash: BTreeMap::new(),
            structure_by_hash: BTreeMap::new(),
            signature_members: Vec::new(),
            signatures_by_hash: BTreeMap::new(),
            signature_count: 0,
            signature_document_frequency: BTreeMap::new(),
        }
    }

    pub(crate) fn push_file(&mut self, path: String, file: FunctionCloneFile) {
        let file_index = self.files.len();
        self.files.push(path.into_boxed_str());
        if !file.parse_ok() {
            self.files_with_parse_errors
                .push(AstFunctionCloneInputError {
                    file: self.files[file_index].to_string(),
                    message: file
                        .parse_error_message()
                        .map(str::to_string)
                        .unwrap_or_else(|| "parse error".to_string()),
                });
        }
        if file.generated() {
            self.generated_file_fact_count += file.function_body_fingerprints().len();
        }
        self.push_body_members(file_index, &file);
        self.push_signature_members(file_index, &file);
    }

    pub(crate) fn finish(
        mut self,
        skipped_files: &[crate::protocol::SkippedFile],
    ) -> AstFunctionCloneGroups {
        let exact_body_groups = self.body_groups(
            &self.exact_by_hash,
            AstFunctionCloneGroupKind::ExactFunctionBodyGroup,
            "same normalized function body; verify domain ownership before merging",
        );
        let structure_groups = self.body_groups(
            &self.structure_by_hash,
            AstFunctionCloneGroupKind::FunctionBodyStructureGroup,
            "same anonymized function-body structure; review cue only, not proof of semantic equivalence",
        );
        let signature_document_frequency = std::mem::take(&mut self.signature_document_frequency);
        let signature_groups = self.signature_groups(signature_document_frequency);
        for member in &mut self.body_members {
            member.prune_grouping_hashes_for_near();
        }
        self.exact_by_hash.clear();
        self.structure_by_hash.clear();
        self.signature_members.clear();
        self.signatures_by_hash.clear();
        let near_members = self
            .body_members
            .iter()
            .map(|member| GroupMember {
                file: self.files[member.file_index].as_ref(),
                fact: member,
                generated: member.generated,
            })
            .collect::<Vec<_>>();
        let near_function_candidates = super::near::build_near_function_candidates_from_members(
            near_members,
            &exact_body_groups,
            &structure_groups,
        );
        let near_diagnostics = near_function_candidates.diagnostics;
        let files_with_read_errors = super::input::files_with_read_errors(skipped_files);
        let complete = self.files_with_parse_errors.is_empty() && files_with_read_errors.is_empty();

        AstFunctionCloneGroups {
            complete,
            files_with_parse_errors: self.files_with_parse_errors,
            files_with_read_errors,
            exact_body_group_count: super::body::review_visible_group_count(&exact_body_groups),
            structure_group_count: super::body::review_visible_group_count(&structure_groups),
            signature_group_count: super::signatures::review_visible_signature_group_count(
                &signature_groups,
            ),
            near_function_candidate_count: near_function_candidates.review_visible_count,
            near_function_candidate_projection_limit: RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES,
            candidate_generation_summary: AstNearFunctionCandidateGenerationSummary {
                eligible_function_count: near_diagnostics.eligible_function_count,
                retained_call_token_bucket_count: near_diagnostics.retained_call_token_bucket_count,
                retained_raw_pair_estimate: near_diagnostics.retained_raw_pair_estimate,
                generated_unique_pair_count: near_diagnostics.generated_unique_pair_count,
                scored_pair_count: near_diagnostics.scored_pair_count,
                compatibility_skipped_raw_pair_estimate_by_reason:
                    AstNearFunctionCompatibilitySkippedPairEstimates {
                        qualifier_mismatch: near_diagnostics
                            .compatibility_skipped_raw_pair_estimate_by_reason
                            .qualifier_mismatch,
                        parameter_count_delta: near_diagnostics
                            .compatibility_skipped_raw_pair_estimate_by_reason
                            .parameter_count_delta,
                        body_loc_band_mismatch: near_diagnostics
                            .compatibility_skipped_raw_pair_estimate_by_reason
                            .body_loc_band_mismatch,
                        statement_count_band_mismatch: near_diagnostics
                            .compatibility_skipped_raw_pair_estimate_by_reason
                            .statement_count_band_mismatch,
                    },
                debug_formatter_boilerplate_skipped_pair_count: near_diagnostics
                    .debug_formatter_boilerplate_skipped_pair_count,
                display_formatter_boilerplate_skipped_pair_count: near_diagnostics
                    .display_formatter_boilerplate_skipped_pair_count,
                compatibility_skipped_pair_estimate_kind: near_diagnostics
                    .compatibility_skipped_pair_estimate_kind(),
                near_function_candidate_count_scope: near_diagnostics
                    .near_function_candidate_count_scope(),
            },
            skipped_low_discrimination_buckets: near_diagnostics.skipped_low_discrimination_buckets,
            skipped_low_discrimination_bucket_count: near_diagnostics
                .skipped_low_discrimination_bucket_count,
            skipped_low_discrimination_raw_pair_estimate: near_diagnostics
                .skipped_low_discrimination_raw_pair_estimate,
            skipped_low_discrimination_pair_estimate_kind:
                RUST_FUNCTION_CLONE_NEAR_SKIPPED_PAIR_ESTIMATE_KIND,
            generated_file_fact_count: self.generated_file_fact_count,
            exact_body_groups,
            structure_groups,
            signature_groups,
            near_function_candidates: near_function_candidates.candidates,
            ..AstFunctionCloneGroups::default()
        }
    }

    fn push_body_members(&mut self, file_index: usize, file: &FunctionCloneFile) {
        let generated = file.generated();
        for fact in file.function_body_fingerprints() {
            let exact_eligible = fact.body_loc() >= RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC
                && fact.statement_count() >= RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS;
            let structure_eligible = fact.body_loc() >= RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC
                && fact.statement_count() >= RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS;
            let exact_hash = exact_eligible.then(|| fact.normalized_exact_hash().into());
            let structure_hash =
                structure_eligible.then(|| fact.normalized_structure_hash().into());
            let member_index = self.body_members.len();
            self.body_members
                .push(OwnedBodyMember::from_fact(file_index, generated, fact));
            if let Some(exact_hash) = exact_hash {
                self.exact_by_hash
                    .entry(exact_hash)
                    .or_default()
                    .push(member_index);
            }
            if let Some(structure_hash) = structure_hash {
                self.structure_by_hash
                    .entry(structure_hash)
                    .or_default()
                    .push(member_index);
            }
        }
    }

    fn push_signature_members(&mut self, file_index: usize, file: &FunctionCloneFile) {
        let generated = file.generated();
        for fact in file.function_signatures() {
            self.signature_count += 1;
            for token in super::signatures::signature_domain_type_tokens(fact) {
                *self.signature_document_frequency.entry(token).or_default() += 1;
            }
            let hash = fact.hash().into();
            let member_index = self.signature_members.len();
            self.signature_members
                .push(OwnedSignatureMember::from_fact(file_index, generated, fact));
            self.signatures_by_hash
                .entry(hash)
                .or_default()
                .push(member_index);
        }
    }

    fn body_groups(
        &self,
        by_hash: &BTreeMap<Box<str>, Vec<usize>>,
        kind: AstFunctionCloneGroupKind,
        reason: &'static str,
    ) -> Vec<AstFunctionCloneGroup> {
        let mut groups = by_hash
            .iter()
            .filter_map(|(hash, member_indexes)| {
                let members = member_indexes
                    .iter()
                    .map(|index| {
                        let member = &self.body_members[*index];
                        GroupMember {
                            file: self.files[member.file_index].as_ref(),
                            fact: member,
                            generated: member.generated,
                        }
                    })
                    .collect::<Vec<_>>();
                super::body::group_from_members(kind, hash.to_string(), members, reason)
            })
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

    fn signature_groups(
        &self,
        signature_document_frequency: BTreeMap<String, usize>,
    ) -> Vec<AstFunctionSignatureGroup> {
        let type_token_idfs = super::signatures::signature_type_token_idfs(
            self.signature_count,
            signature_document_frequency,
        );
        let mut groups = self
            .signatures_by_hash
            .iter()
            .filter_map(|(hash, member_indexes)| {
                let members = member_indexes
                    .iter()
                    .map(|index| {
                        let member = &self.signature_members[*index];
                        SignatureMember {
                            file: self.files[member.file_index].as_ref(),
                            fact: member,
                            generated: member.generated,
                        }
                    })
                    .collect::<Vec<_>>();
                super::signatures::signature_group_from_members(
                    hash.to_string(),
                    members,
                    &type_token_idfs,
                )
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
}

#[derive(Debug)]
struct OwnedBodyMember {
    file_index: usize,
    generated: bool,
    name: Box<str>,
    visibility: AstVisibility,
    owner: Option<AstFunctionOwner>,
    exact_body_hash: Box<str>,
    body_loc: usize,
    statement_count: usize,
    param_count: usize,
    is_async: bool,
    is_unsafe: bool,
    is_const: bool,
    call_tokens: Vec<Box<str>>,
    line: usize,
}

impl OwnedBodyMember {
    fn from_fact(
        file_index: usize,
        generated: bool,
        fact: &impl FunctionBodyFactView,
    ) -> OwnedBodyMember {
        Self {
            file_index,
            generated,
            name: fact.name().into(),
            visibility: fact.visibility(),
            owner: fact.owner().cloned(),
            exact_body_hash: fact.exact_body_hash().into(),
            body_loc: fact.body_loc(),
            statement_count: fact.statement_count(),
            param_count: fact.param_count(),
            is_async: fact.is_async(),
            is_unsafe: fact.is_unsafe(),
            is_const: fact.is_const(),
            call_tokens: fact
                .call_tokens()
                .iter()
                .map(|token| token.as_ref().into())
                .collect::<BTreeSet<Box<str>>>()
                .into_iter()
                .collect(),
            line: fact.line(),
        }
    }

    fn prune_grouping_hashes_for_near(&mut self) {
        self.exact_body_hash = "".into();
    }
}

impl FunctionBodyFactView for OwnedBodyMember {
    type CallToken = Box<str>;

    fn name(&self) -> &str {
        &self.name
    }

    fn visibility(&self) -> AstVisibility {
        self.visibility
    }

    fn owner(&self) -> Option<&AstFunctionOwner> {
        self.owner.as_ref()
    }

    fn exact_body_hash(&self) -> &str {
        &self.exact_body_hash
    }

    fn normalized_exact_hash(&self) -> &str {
        ""
    }

    fn normalized_structure_hash(&self) -> &str {
        ""
    }

    fn body_loc(&self) -> usize {
        self.body_loc
    }

    fn statement_count(&self) -> usize {
        self.statement_count
    }

    fn param_count(&self) -> usize {
        self.param_count
    }

    fn is_async(&self) -> bool {
        self.is_async
    }

    fn is_unsafe(&self) -> bool {
        self.is_unsafe
    }

    fn is_const(&self) -> bool {
        self.is_const
    }

    fn call_tokens(&self) -> &[Self::CallToken] {
        &self.call_tokens
    }

    fn line(&self) -> usize {
        self.line
    }
}

#[derive(Debug)]
struct OwnedSignatureMember {
    file_index: usize,
    generated: bool,
    hash: Box<str>,
    name: Box<str>,
    visibility: AstVisibility,
    owner: Option<AstFunctionOwner>,
    generics: Option<Box<str>>,
    receiver_text: Option<Box<str>>,
    params: Vec<Box<str>>,
    return_type: Option<Box<str>>,
    line: usize,
}

impl OwnedSignatureMember {
    fn from_fact(
        file_index: usize,
        generated: bool,
        fact: &impl FunctionSignatureFactView,
    ) -> OwnedSignatureMember {
        Self {
            file_index,
            generated,
            hash: fact.hash().into(),
            name: fact.name().into(),
            visibility: fact.visibility(),
            owner: fact.owner().cloned(),
            generics: fact.generics().map(Into::into),
            receiver_text: fact.receiver_text().map(Into::into),
            params: fact
                .param_type_texts()
                .into_iter()
                .map(Into::into)
                .collect(),
            return_type: fact.return_type().map(Into::into),
            line: fact.line(),
        }
    }
}

impl FunctionSignatureFactView for OwnedSignatureMember {
    fn hash(&self) -> &str {
        &self.hash
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn visibility(&self) -> AstVisibility {
        self.visibility
    }

    fn owner(&self) -> Option<&AstFunctionOwner> {
        self.owner.as_ref()
    }

    fn generics(&self) -> Option<&str> {
        self.generics.as_deref()
    }

    fn receiver_text(&self) -> Option<&str> {
        self.receiver_text.as_deref()
    }

    fn param_type_texts(&self) -> Vec<&str> {
        self.params.iter().map(AsRef::as_ref).collect()
    }

    fn return_type(&self) -> Option<&str> {
        self.return_type.as_deref()
    }

    fn line(&self) -> usize {
        self.line
    }
}
