use std::collections::BTreeMap;

use crate::prewrite::cues::model::{
    Cue, CueCandidate, CueCardBuilder, CueClaim, CueConfidence, CueEvidence, CueMatchedField,
    CueTier, EvidenceLane, NotSafeFor,
};
use crate::prewrite::lookup::{
    InlinePatternGroup, InlinePatternLookup, INLINE_PATTERN_POLICY_ID,
    INLINE_PATTERN_POLICY_VERSION,
};

use super::add_cue_for_candidate;

pub(super) fn add_inline_pattern_cues(
    lookups: &[InlinePatternLookup],
    cards: &mut BTreeMap<String, CueCardBuilder>,
) {
    for lookup in lookups.iter().filter(|lookup| lookup.is_match()) {
        for group in lookup.groups() {
            let candidate = candidate(group);
            add_cue_for_candidate(cards, candidate, cue(group));
        }
    }
}

fn candidate(group: &InlinePatternGroup) -> CueCandidate {
    CueCandidate {
        identity: group.identity(),
        owner_file: group
            .owner_files
            .first()
            .cloned()
            .unwrap_or_else(|| "<unknown>".to_string()),
        name: "statement-sequence".to_string(),
    }
}

fn cue(group: &InlinePatternGroup) -> Cue {
    let identity = group.identity();
    Cue {
        cue_tier: CueTier::AgentReview,
        safe_meaning: None,
        not_safe_for: vec![
            NotSafeFor::SemanticEquivalence,
            NotSafeFor::AutoReuse,
            NotSafeFor::AutoFix,
        ],
        evidence_lane: EvidenceLane::InlineExtraction,
        claim: CueClaim::RepeatedInlineStatementPattern,
        confidence: CueConfidence::Grounded,
        evidence: vec![CueEvidence {
            artifact: "rust-source-health",
            matched_field: CueMatchedField::RustSourceHealthInlinePatternHash,
            matched_field_source: None,
            algorithm_version: Some(group.normalizer_version),
            hash: Some(group.pattern_hash.clone()),
            visibility: None,
            local_name: None,
            candidate_identity: identity,
            file: group
                .occurrences
                .first()
                .map(|occurrence| occurrence.file.clone()),
            file_lookup_result: None,
            dependency_lookup_result: None,
            observed_import_count: None,
            consumer_threshold: None,
            distance: None,
            tokens: Vec::new(),
            policy_id: Some(INLINE_PATTERN_POLICY_ID),
            policy_version: Some(INLINE_PATTERN_POLICY_VERSION),
            operation_family: None,
            shared_domain_tokens: Vec::new(),
            locality: None,
            supporting_reasons: Vec::new(),
            surface_kind: Some("statement-sequence"),
            container_name: group
                .occurrences
                .first()
                .map(|occurrence| occurrence.enclosing_function.clone()),
            container_kind: Some("function"),
        }],
    }
}
