use crate::protocol::AstNearFunctionCandidate;

use crate::function_clones::common::GroupMember;

pub(in crate::function_clones) struct NearFunctionCandidateProjection {
    pub(in crate::function_clones) review_visible_count: usize,
    pub(in crate::function_clones) candidates: Vec<AstNearFunctionCandidate>,
}

pub(super) struct NearFact<'a> {
    pub(super) member: GroupMember<'a>,
    pub(super) identity: String,
    pub(super) significant_call_tokens: Vec<String>,
    pub(super) name_tokens: Vec<String>,
}
