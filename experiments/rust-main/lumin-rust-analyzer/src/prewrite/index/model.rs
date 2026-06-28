use lumin_rust_source_health::protocol::{
    AstDefinitionKind, AstFunctionSignature, AstVisibility, Location, PathClassification, PathMeta,
};
use serde::Serialize;

use crate::prewrite::operation::ServiceOperationFamily;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(in crate::prewrite) enum CandidateLane {
    Definition,
    UseTree,
    ImplMethod,
}

impl CandidateLane {
    pub(in crate::prewrite) fn matched_field(self) -> MatchedField {
        match self {
            Self::Definition => MatchedField::Def,
            Self::UseTree => MatchedField::UseTree,
            Self::ImplMethod => MatchedField::ImplMethod,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(in crate::prewrite) enum MatchedField {
    #[serde(rename = "defIndex")]
    Def,
    #[serde(rename = "useTreeIndex")]
    UseTree,
    #[serde(rename = "implMethodIndex")]
    ImplMethod,
    #[serde(rename = "preWriteLocalOperationIndex")]
    PreWriteLocalOperation,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::prewrite) struct ImplOwner<'a> {
    pub(in crate::prewrite) target: &'a str,
    pub(in crate::prewrite) trait_path: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::prewrite) struct Candidate<'a> {
    pub(in crate::prewrite) lane: CandidateLane,
    pub(in crate::prewrite) file: &'a str,
    pub(in crate::prewrite) name: &'a str,
    pub(in crate::prewrite) owner: Option<ImplOwner<'a>>,
    pub(in crate::prewrite) definition_kind: Option<AstDefinitionKind>,
    pub(in crate::prewrite) visibility: AstVisibility,
    pub(in crate::prewrite) location: &'a Location,
    pub(in crate::prewrite) path: &'a PathMeta,
    pub(in crate::prewrite) function_signature: Option<&'a AstFunctionSignature>,
}

impl<'a> Candidate<'a> {
    pub(in crate::prewrite) fn identity(self) -> String {
        match self.owner {
            None => format!("{}::{}", self.file, self.name),
            Some(owner) => match owner.trait_path {
                None => format!("{}::{}#{}", self.file, owner.target, self.name),
                Some(trait_path) => format!(
                    "{}::{} as {}#{}",
                    self.file, owner.target, trait_path, self.name
                ),
            },
        }
    }

    pub(in crate::prewrite) fn owner_name(self) -> Option<&'a str> {
        self.owner.map(|owner| owner.target)
    }

    pub(in crate::prewrite) fn classifications(self) -> Vec<PathClassification> {
        self.path.classifications.clone()
    }
}

#[derive(Debug, Clone)]
pub(in crate::prewrite) struct LocalOperationCandidate<'a> {
    pub(in crate::prewrite) file: &'a str,
    pub(in crate::prewrite) name: &'a str,
    pub(in crate::prewrite) container_name: &'a str,
    pub(in crate::prewrite) container_kind: &'static str,
    pub(in crate::prewrite) operation_family: ServiceOperationFamily,
    pub(in crate::prewrite) location: &'a Location,
    pub(in crate::prewrite) container_location: &'a Location,
    pub(in crate::prewrite) path: &'a PathMeta,
}

impl<'a> LocalOperationCandidate<'a> {
    pub(in crate::prewrite) fn identity(&self) -> String {
        format!("{}::{}#{}", self.file, self.container_name, self.name)
    }

    pub(in crate::prewrite) fn classifications(&self) -> Vec<PathClassification> {
        self.path.classifications.clone()
    }
}

pub(in crate::prewrite) struct CandidateIndex<'a> {
    pub(in crate::prewrite) candidates: Vec<Candidate<'a>>,
    pub(in crate::prewrite) local_operations: Vec<LocalOperationCandidate<'a>>,
}
