use std::collections::BTreeMap;

use lumin_rust_source_health::protocol::PathClassification;
use serde::Serialize;

use super::index::MatchedField;
use super::lookup::{
    CandidateRecord, NameLookup, SuppressedNearNameHint, SuppressedSemanticHint, SuppressionReason,
};
use super::tokens::TOKEN_POLICY_VERSION;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub(super) enum CueTier {
    #[serde(rename = "SAFE_CUE")]
    Safe,
    #[serde(rename = "AGENT_REVIEW_CUE")]
    AgentReview,
    #[serde(rename = "MUTED_CUE")]
    Muted,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum EvidenceLane {
    ExactSymbol,
    ImplMethodName,
    NearName,
    IntentToken,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum CueConfidence {
    Grounded,
    HeuristicReview,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SafeMeaning {
    ClaimOnly,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum NotSafeFor {
    SemanticEquivalence,
    AutoReuse,
    AutoFix,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum CueClaim {
    ExactRustDefinitionExists,
    NearRustDefinitionName,
    NearRustImplMethodName,
    SupportedIntentTokenOverlap,
    RustImplMethodIntentTokenOverlap,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CueEvidence {
    artifact: &'static str,
    pub(super) matched_field: MatchedField,
    algorithm_version: &'static str,
    candidate_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    distance: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Cue {
    pub(super) cue_tier: CueTier,
    #[serde(skip_serializing_if = "Option::is_none")]
    safe_meaning: Option<SafeMeaning>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    not_safe_for: Vec<NotSafeFor>,
    pub(super) evidence_lane: EvidenceLane,
    claim: CueClaim,
    confidence: CueConfidence,
    pub(super) evidence: Vec<CueEvidence>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CueCandidate {
    pub(super) identity: String,
    pub(super) owner_file: String,
    pub(super) name: String,
}

impl From<&CandidateRecord> for CueCandidate {
    fn from(candidate: &CandidateRecord) -> Self {
        Self {
            identity: candidate.identity.clone(),
            owner_file: candidate.owner_file.clone(),
            name: candidate.name.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CueCard {
    pub(super) candidate: CueCandidate,
    pub(super) render_tier: CueTier,
    pub(super) cues: Vec<Cue>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum MutedReason {
    PolicyExcluded,
    DomainTokenOverlap,
    NearLengthDeltaExceeded,
    NearPrefixMismatch,
    NearDistanceExceeded,
    SingleNonWeakTokenOnly,
    InsufficientNonWeakSupport,
}

impl From<SuppressionReason> for MutedReason {
    fn from(reason: SuppressionReason) -> Self {
        match reason {
            SuppressionReason::DomainTokenOverlap => Self::DomainTokenOverlap,
            SuppressionReason::NearLengthDeltaExceeded => Self::NearLengthDeltaExceeded,
            SuppressionReason::NearPrefixMismatch => Self::NearPrefixMismatch,
            SuppressionReason::NearDistanceExceeded => Self::NearDistanceExceeded,
            SuppressionReason::SingleNonWeakTokenOnly => Self::SingleNonWeakTokenOnly,
            SuppressionReason::InsufficientNonWeakSupport => Self::InsufficientNonWeakSupport,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SuppressedCue {
    pub(super) cue_tier: CueTier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) original_cue_tier: Option<CueTier>,
    pub(super) evidence_lane: EvidenceLane,
    pub(super) reason: MutedReason,
    pub(super) candidate: CueCandidate,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) path_classifications: Vec<PathClassification>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tokens: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    distance: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    score: Option<usize>,
    candidate_count: usize,
}

pub(super) struct CueProjection {
    pub(super) cue_cards: Vec<CueCard>,
    pub(super) suppressed_cues: Vec<SuppressedCue>,
}

struct CueCardBuilder {
    candidate: CueCandidate,
    render_tier: CueTier,
    cues: Vec<Cue>,
}

pub(super) fn project(lookups: &[NameLookup]) -> CueProjection {
    let mut cards = BTreeMap::<String, CueCardBuilder>::new();
    let mut suppressed = Vec::new();
    for lookup in lookups {
        for candidate in &lookup.identities {
            add_active_cue(&mut cards, &mut suppressed, candidate, safe_cue(candidate));
        }
        for hint in &lookup.near_names {
            add_active_cue(
                &mut cards,
                &mut suppressed,
                &hint.candidate,
                near_name_cue(&hint.candidate, hint.distance),
            );
        }
        for hint in &lookup.semantic_hints {
            add_active_cue(
                &mut cards,
                &mut suppressed,
                &hint.candidate,
                semantic_hint_cue(&hint.candidate, &hint.matched_tokens),
            );
        }
        suppressed.extend(lookup.suppressed_near_names.iter().map(suppressed_near_cue));
        suppressed.extend(
            lookup
                .suppressed_semantic_hints
                .iter()
                .map(suppressed_semantic_cue),
        );
    }

    let mut cue_cards = cards
        .into_values()
        .map(|builder| CueCard {
            candidate: builder.candidate,
            render_tier: builder.render_tier,
            cues: builder.cues,
        })
        .collect::<Vec<_>>();
    cue_cards.sort_by(|left, right| {
        tier_rank(left.render_tier)
            .cmp(&tier_rank(right.render_tier))
            .then(left.candidate.owner_file.cmp(&right.candidate.owner_file))
            .then(left.candidate.name.cmp(&right.candidate.name))
            .then(left.candidate.identity.cmp(&right.candidate.identity))
    });
    suppressed.sort_by(|left, right| {
        left.reason
            .cmp(&right.reason)
            .then(left.candidate.owner_file.cmp(&right.candidate.owner_file))
            .then(left.candidate.name.cmp(&right.candidate.name))
            .then(left.candidate.identity.cmp(&right.candidate.identity))
    });
    CueProjection {
        cue_cards,
        suppressed_cues: suppressed,
    }
}

fn add_active_cue(
    cards: &mut BTreeMap<String, CueCardBuilder>,
    suppressed: &mut Vec<SuppressedCue>,
    candidate: &CandidateRecord,
    cue: Cue,
) {
    if candidate.policy_excluded {
        suppressed.push(SuppressedCue {
            cue_tier: CueTier::Muted,
            original_cue_tier: Some(cue.cue_tier),
            evidence_lane: cue.evidence_lane,
            reason: MutedReason::PolicyExcluded,
            candidate: CueCandidate::from(candidate),
            path_classifications: candidate.path_classifications.clone(),
            tokens: Vec::new(),
            distance: cue.evidence.first().and_then(|evidence| evidence.distance),
            score: None,
            candidate_count: 1,
        });
        return;
    }

    let card = cards
        .entry(candidate.identity.clone())
        .or_insert_with(|| CueCardBuilder {
            candidate: CueCandidate::from(candidate),
            render_tier: CueTier::Safe,
            cues: Vec::new(),
        });
    if cue.cue_tier == CueTier::AgentReview {
        card.render_tier = CueTier::AgentReview;
    }
    card.cues.push(cue);
}

fn safe_cue(candidate: &CandidateRecord) -> Cue {
    Cue {
        cue_tier: CueTier::Safe,
        safe_meaning: Some(SafeMeaning::ClaimOnly),
        not_safe_for: vec![
            NotSafeFor::SemanticEquivalence,
            NotSafeFor::AutoReuse,
            NotSafeFor::AutoFix,
        ],
        evidence_lane: EvidenceLane::ExactSymbol,
        claim: CueClaim::ExactRustDefinitionExists,
        confidence: CueConfidence::Grounded,
        evidence: vec![CueEvidence {
            artifact: "rust-source-health",
            matched_field: candidate.matched_field,
            algorithm_version: "exact-symbol.v1",
            candidate_identity: candidate.identity.clone(),
            distance: None,
            tokens: Vec::new(),
        }],
    }
}

fn near_name_cue(candidate: &CandidateRecord, distance: usize) -> Cue {
    let impl_method = candidate.matched_field == MatchedField::ImplMethodIndex;
    Cue {
        cue_tier: CueTier::AgentReview,
        safe_meaning: None,
        not_safe_for: Vec::new(),
        evidence_lane: if impl_method {
            EvidenceLane::ImplMethodName
        } else {
            EvidenceLane::NearName
        },
        claim: if impl_method {
            CueClaim::NearRustImplMethodName
        } else {
            CueClaim::NearRustDefinitionName
        },
        confidence: CueConfidence::HeuristicReview,
        evidence: vec![CueEvidence {
            artifact: "rust-source-health",
            matched_field: candidate.matched_field,
            algorithm_version: "near-name.v1",
            candidate_identity: candidate.identity.clone(),
            distance: Some(distance),
            tokens: Vec::new(),
        }],
    }
}

fn semantic_hint_cue(candidate: &CandidateRecord, tokens: &[String]) -> Cue {
    let impl_method = candidate.matched_field == MatchedField::ImplMethodIndex;
    Cue {
        cue_tier: CueTier::AgentReview,
        safe_meaning: None,
        not_safe_for: Vec::new(),
        evidence_lane: if impl_method {
            EvidenceLane::ImplMethodName
        } else {
            EvidenceLane::IntentToken
        },
        claim: if impl_method {
            CueClaim::RustImplMethodIntentTokenOverlap
        } else {
            CueClaim::SupportedIntentTokenOverlap
        },
        confidence: CueConfidence::HeuristicReview,
        evidence: vec![CueEvidence {
            artifact: "rust-source-health",
            matched_field: candidate.matched_field,
            algorithm_version: TOKEN_POLICY_VERSION,
            candidate_identity: candidate.identity.clone(),
            distance: None,
            tokens: tokens.to_vec(),
        }],
    }
}

fn suppressed_near_cue(hint: &SuppressedNearNameHint) -> SuppressedCue {
    SuppressedCue {
        cue_tier: CueTier::Muted,
        original_cue_tier: None,
        evidence_lane: EvidenceLane::NearName,
        reason: hint.reason.into(),
        candidate: CueCandidate::from(&hint.candidate),
        path_classifications: hint.candidate.path_classifications.clone(),
        tokens: hint.matched_tokens.clone(),
        distance: hint.distance,
        score: None,
        candidate_count: hint.candidate_count,
    }
}

fn suppressed_semantic_cue(hint: &SuppressedSemanticHint) -> SuppressedCue {
    SuppressedCue {
        cue_tier: CueTier::Muted,
        original_cue_tier: None,
        evidence_lane: EvidenceLane::IntentToken,
        reason: hint.reason.into(),
        candidate: CueCandidate::from(&hint.candidate),
        path_classifications: hint.candidate.path_classifications.clone(),
        tokens: hint.matched_tokens.clone(),
        distance: None,
        score: Some(hint.score),
        candidate_count: hint.candidate_count,
    }
}

fn tier_rank(tier: CueTier) -> usize {
    match tier {
        CueTier::Safe => 0,
        CueTier::AgentReview => 1,
        CueTier::Muted => 2,
    }
}
