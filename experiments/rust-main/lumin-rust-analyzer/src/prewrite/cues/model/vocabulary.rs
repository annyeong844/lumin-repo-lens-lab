use serde::Serialize;

use crate::prewrite::index::MatchedField;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub(in crate::prewrite) enum CueTier {
    #[serde(rename = "SAFE_CUE")]
    Safe,
    #[serde(rename = "AGENT_REVIEW_CUE")]
    AgentReview,
    #[serde(rename = "MUTED_CUE")]
    Muted,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum EvidenceLane {
    ExactSymbol,
    ExactFile,
    ImplMethodName,
    NearName,
    IntentToken,
    FileDomainCluster,
    DependencyHub,
    ServiceOperationSibling,
    LocalOperationSibling,
    ShapeHash,
    FunctionSignature,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite::cues) enum CueConfidence {
    Grounded,
    HeuristicReview,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite::cues) enum SafeMeaning {
    ClaimOnly,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite::cues) enum NotSafeFor {
    SemanticEquivalence,
    AutoReuse,
    AutoFix,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite::cues) enum CueClaim {
    ExactRustDefinitionExists,
    #[serde(rename = "exact Rust use-tree name exists")]
    ExactRustUseTreeNameExists,
    #[serde(rename = "exact file exists")]
    ExactFileExists,
    NearRustDefinitionName,
    NearRustImplMethodName,
    SupportedIntentTokenOverlap,
    RustImplMethodIntentTokenOverlap,
    #[serde(rename = "related Rust file domain cluster")]
    RelatedRustFileDomainCluster,
    #[serde(rename = "Rust dependency hub")]
    RustDependencyHub,
    #[serde(rename = "related service operation sibling")]
    RelatedServiceOperationSibling,
    #[serde(rename = "related local service operation")]
    RelatedLocalServiceOperation,
    #[serde(rename = "same normalized type shape")]
    SameNormalizedTypeShape,
    #[serde(rename = "same normalized function signature")]
    SameNormalizedFunctionSignature,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(in crate::prewrite) enum CueMatchedField {
    #[serde(rename = "defIndex")]
    DefIndex,
    #[serde(rename = "files[].ast.useTrees[]")]
    RustSourceHealthUseTrees,
    #[serde(rename = "implMethodIndex")]
    ImplMethodIndex,
    #[serde(rename = "files")]
    RustSourceHealthFiles,
    #[serde(rename = "fileLookups[].domainCluster")]
    FileDomainCluster,
    #[serde(rename = "dependencyLookups[].existingImports")]
    DependencyExistingImports,
    #[serde(rename = "preWriteLocalOperationIndex")]
    PreWriteLocalOperationIndex,
    #[serde(rename = "lookups[].serviceOperationSiblingPolicy.promoted")]
    ServiceOperationSiblingPolicyPromoted,
    #[serde(rename = "lookups[].localOperationSiblingPolicy.promoted")]
    LocalOperationSiblingPolicyPromoted,
    #[serde(rename = "files[].ast.shapeHashes[].hash")]
    RustSourceHealthShapeHash,
    #[serde(rename = "files[].ast.functionSignatures[].hash")]
    RustSourceHealthFunctionSignatureHash,
}

impl From<MatchedField> for CueMatchedField {
    fn from(field: MatchedField) -> Self {
        match field {
            MatchedField::Def => Self::DefIndex,
            MatchedField::UseTree => Self::RustSourceHealthUseTrees,
            MatchedField::ImplMethod => Self::ImplMethodIndex,
            MatchedField::PreWriteLocalOperation => Self::PreWriteLocalOperationIndex,
        }
    }
}
