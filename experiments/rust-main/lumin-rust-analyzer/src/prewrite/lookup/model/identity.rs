use lumin_rust_source_health::protocol::{
    AstDefinitionKind, AstFunctionSignature, AstVisibility, PathClassification,
};
use serde::Serialize;

use crate::prewrite::index::{Candidate, MatchedField};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(in crate::prewrite) enum LookupResult {
    #[serde(rename = "NOT_OBSERVED")]
    NotObserved,
    #[serde(rename = "EXISTS")]
    Exists,
    #[serde(rename = "EXISTS_MULTIPLE")]
    ExistsMultiple,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct CandidateRecord {
    pub(in crate::prewrite) identity: String,
    pub(in crate::prewrite) owner_file: String,
    pub(in crate::prewrite) name: String,
    pub(in crate::prewrite) matched_field: MatchedField,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) definition_kind: Option<AstDefinitionKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) impl_target: Option<String>,
    #[serde(rename = "trait", skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) trait_path: Option<String>,
    pub(in crate::prewrite) visibility: AstVisibility,
    pub(in crate::prewrite) line: usize,
    pub(in crate::prewrite) column: usize,
    #[serde(skip)]
    pub(in crate::prewrite) function_signature: Option<FunctionSignatureEvidence>,
    pub(in crate::prewrite) policy_excluded: bool,
    pub(in crate::prewrite) path_classifications: Vec<PathClassification>,
}

#[derive(Debug, Clone)]
pub(in crate::prewrite) struct FunctionSignatureEvidence {
    pub(in crate::prewrite) hash: String,
    pub(in crate::prewrite) normalized_version: String,
}

impl FunctionSignatureEvidence {
    fn from_fact(fact: &AstFunctionSignature) -> Self {
        Self {
            hash: fact.hash.clone(),
            normalized_version: fact.normalized_version.clone(),
        }
    }
}

impl CandidateRecord {
    pub(in crate::prewrite) fn from_candidate(candidate: Candidate<'_>) -> Self {
        let path_classifications = candidate.classifications();
        let policy_excluded = candidate.path.suppressed
            || path_classifications.iter().any(|classification| {
                matches!(
                    classification,
                    PathClassification::Test | PathClassification::Generated
                )
            });
        Self {
            identity: candidate.identity(),
            owner_file: candidate.file.to_string(),
            name: candidate.name.to_string(),
            matched_field: candidate.lane.matched_field(),
            definition_kind: candidate.definition_kind,
            impl_target: candidate.owner.map(|owner| owner.target.to_string()),
            trait_path: candidate
                .owner
                .and_then(|owner| owner.trait_path)
                .map(str::to_string),
            visibility: candidate.visibility,
            line: candidate.location.line,
            column: candidate.location.column,
            function_signature: candidate
                .function_signature
                .map(FunctionSignatureEvidence::from_fact),
            policy_excluded,
            path_classifications,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct Locality {
    pub(in crate::prewrite) same_dir: bool,
    pub(in crate::prewrite) same_file: bool,
}

impl Locality {
    pub(in crate::prewrite) fn rank(self) -> usize {
        if self.same_file {
            2
        } else if self.same_dir {
            1
        } else {
            0
        }
    }
}
