use std::collections::BTreeSet;
use std::path::Path;

use lumin_rust_common::posix_path_text;

mod mute;
mod order;

use super::model::{
    LocalOperationPolicyEntry, LocalOperationPolicyStatus, LocalOperationSiblingPolicy, Locality,
    PolicySupportingReason,
};
use crate::prewrite::index::LocalOperationCandidate;
use crate::prewrite::operation::service_operation_info;

use mute::local_mute_reason;
use order::sort_local_entries;

pub(in crate::prewrite) const LOCAL_OPERATION_POLICY_ID: &str = "prewrite-local-operation-sibling";
pub(in crate::prewrite) const LOCAL_OPERATION_POLICY_VERSION: &str =
    "prewrite-local-operation-sibling-v1";
pub(in crate::prewrite) const LOCAL_OPERATION_POLICY_MAX_RESULTS: usize = 5;
const INTENT_OWNER_FILE_MISSING: &str = "intent-owner-file-missing";

pub(super) fn local_operation_sibling_policy(
    intent_name: &str,
    owner_file: Option<&str>,
    local_operations: &[LocalOperationCandidate<'_>],
) -> LocalOperationSiblingPolicy {
    let Some(owner_file) = owner_file else {
        return empty_policy(Some(INTENT_OWNER_FILE_MISSING));
    };
    let owner_file = posix_path_text(owner_file).into_owned();

    let intent_operation = service_operation_info(intent_name);
    let intent_domains = intent_operation
        .domain_tokens
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut promoted = Vec::new();
    let mut muted = Vec::new();

    for candidate in local_operations
        .iter()
        .filter(|candidate| candidate.file == owner_file.as_str())
    {
        let candidate_domain_tokens = service_operation_info(candidate.name).domain_tokens;
        let candidate_domains = candidate_domain_tokens
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let shared_domain_tokens = intent_operation
            .domain_tokens
            .iter()
            .filter(|token| candidate_domains.contains(*token))
            .cloned()
            .collect::<Vec<_>>();
        let locality = locality_for(&owner_file, candidate.file);
        let reason = local_mute_reason(
            candidate,
            intent_operation.operation_family,
            &intent_domains,
            &shared_domain_tokens,
            locality,
        );
        let supporting_reasons = if reason.is_none() {
            vec![PolicySupportingReason::LocalOperationSameFileDomainOverlap]
        } else {
            Vec::new()
        };
        let entry = LocalOperationPolicyEntry::from_candidate(
            candidate,
            candidate_domain_tokens,
            shared_domain_tokens,
            supporting_reasons,
            reason,
            locality,
        );
        if reason.is_some() {
            muted.push(entry);
        } else {
            promoted.push(entry);
        }
    }

    sort_local_entries(&mut promoted);
    sort_local_entries(&mut muted);
    let promoted_candidate_count = promoted.len();
    let muted_candidate_count = muted.len();
    promoted.truncate(LOCAL_OPERATION_POLICY_MAX_RESULTS);
    muted.truncate(LOCAL_OPERATION_POLICY_MAX_RESULTS);

    LocalOperationSiblingPolicy {
        policy_id: LOCAL_OPERATION_POLICY_ID,
        policy_version: LOCAL_OPERATION_POLICY_VERSION,
        status: LocalOperationPolicyStatus::Complete,
        reason: None,
        evaluated_candidate_count: promoted_candidate_count + muted_candidate_count,
        promoted_candidate_count,
        muted_candidate_count,
        promoted,
        muted,
    }
}

fn empty_policy(reason: Option<&'static str>) -> LocalOperationSiblingPolicy {
    LocalOperationSiblingPolicy {
        policy_id: LOCAL_OPERATION_POLICY_ID,
        policy_version: LOCAL_OPERATION_POLICY_VERSION,
        status: LocalOperationPolicyStatus::Complete,
        reason,
        evaluated_candidate_count: 0,
        promoted_candidate_count: 0,
        muted_candidate_count: 0,
        promoted: Vec::new(),
        muted: Vec::new(),
    }
}

fn locality_for(owner_file: &str, candidate_file: &str) -> Locality {
    Locality {
        same_file: owner_file == candidate_file,
        same_dir: parent_path(owner_file) == parent_path(candidate_file),
    }
}

fn parent_path(path: &str) -> Option<&Path> {
    Path::new(path).parent()
}
