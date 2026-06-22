use lumin_rust_source_health::protocol::{
    AstDefinitionKind, AstVisibility, HealthResponse, Location, PathClassification, PathMeta,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum CandidateLane {
    Definition,
    ImplMethod,
}

impl CandidateLane {
    pub(super) fn matched_field(self) -> MatchedField {
        match self {
            Self::Definition => MatchedField::DefIndex,
            Self::ImplMethod => MatchedField::ImplMethodIndex,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(super) enum MatchedField {
    #[serde(rename = "defIndex")]
    DefIndex,
    #[serde(rename = "implMethodIndex")]
    ImplMethodIndex,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ImplOwner<'a> {
    pub(super) target: &'a str,
    pub(super) trait_path: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Candidate<'a> {
    pub(super) lane: CandidateLane,
    pub(super) file: &'a str,
    pub(super) name: &'a str,
    pub(super) owner: Option<ImplOwner<'a>>,
    pub(super) definition_kind: Option<AstDefinitionKind>,
    pub(super) visibility: AstVisibility,
    pub(super) location: &'a Location,
    pub(super) path: &'a PathMeta,
}

impl<'a> Candidate<'a> {
    pub(super) fn identity(self) -> String {
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

    pub(super) fn owner_name(self) -> Option<&'a str> {
        self.owner.map(|owner| owner.target)
    }

    pub(super) fn classifications(self) -> Vec<PathClassification> {
        self.path.classifications.clone()
    }
}

pub(super) struct CandidateIndex<'a> {
    pub(super) candidates: Vec<Candidate<'a>>,
}

impl<'a> CandidateIndex<'a> {
    pub(super) fn from_health(response: &'a HealthResponse) -> Self {
        let mut candidates = Vec::new();
        for (file, health) in &response.files {
            let impl_method_ranges = health
                .ast
                .impls
                .iter()
                .flat_map(|impl_block| impl_block.methods.iter())
                .map(|method| (method.location.byte_start, method.location.byte_end))
                .collect::<Vec<_>>();

            candidates.extend(
                health
                    .ast
                    .definitions
                    .iter()
                    .filter(|definition| {
                        definition.kind != AstDefinitionKind::Function
                            || !impl_method_ranges.contains(&(
                                definition.location.byte_start,
                                definition.location.byte_end,
                            ))
                    })
                    .map(|definition| Candidate {
                        lane: CandidateLane::Definition,
                        file,
                        name: &definition.name,
                        owner: None,
                        definition_kind: Some(definition.kind),
                        visibility: definition.visibility,
                        location: &definition.location,
                        path: &health.path,
                    }),
            );

            for impl_block in &health.ast.impls {
                let owner = ImplOwner {
                    target: &impl_block.target,
                    trait_path: impl_block.trait_path.as_deref(),
                };
                candidates.extend(impl_block.methods.iter().map(|method| Candidate {
                    lane: CandidateLane::ImplMethod,
                    file,
                    name: &method.name,
                    owner: Some(owner),
                    definition_kind: None,
                    visibility: method.visibility,
                    location: &method.location,
                    path: &health.path,
                }));
            }
        }

        candidates.sort_by(|left, right| {
            left.file
                .cmp(right.file)
                .then(left.owner_name().cmp(&right.owner_name()))
                .then(left.name.cmp(right.name))
                .then(left.location.byte_start.cmp(&right.location.byte_start))
                .then(left.lane.cmp(&right.lane))
        });
        Self { candidates }
    }
}
