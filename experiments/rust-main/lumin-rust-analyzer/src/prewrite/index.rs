use std::collections::BTreeSet;

use lumin_rust_source_health::protocol::{
    AstDefinition, AstDefinitionKind, AstVisibility, FileHealth, HealthResponse, Location,
    PathClassification, PathMeta,
};
use serde::Serialize;

use super::operation::{
    is_local_operation_container_name, local_operation_info, ServiceOperationFamily,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum CandidateLane {
    Definition,
    ImplMethod,
}

impl CandidateLane {
    pub(super) fn matched_field(self) -> MatchedField {
        match self {
            Self::Definition => MatchedField::Def,
            Self::ImplMethod => MatchedField::ImplMethod,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(super) enum MatchedField {
    #[serde(rename = "defIndex")]
    Def,
    #[serde(rename = "implMethodIndex")]
    ImplMethod,
    #[serde(rename = "preWriteLocalOperationIndex")]
    PreWriteLocalOperation,
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

#[derive(Debug, Clone)]
pub(super) struct LocalOperationCandidate<'a> {
    pub(super) file: &'a str,
    pub(super) name: &'a str,
    pub(super) container_name: &'a str,
    pub(super) container_kind: &'static str,
    pub(super) operation_family: ServiceOperationFamily,
    pub(super) location: &'a Location,
    pub(super) container_location: &'a Location,
    pub(super) path: &'a PathMeta,
}

impl<'a> LocalOperationCandidate<'a> {
    pub(super) fn identity(&self) -> String {
        format!("{}::{}#{}", self.file, self.container_name, self.name)
    }

    pub(super) fn classifications(&self) -> Vec<PathClassification> {
        self.path.classifications.clone()
    }
}

pub(super) struct CandidateIndex<'a> {
    pub(super) candidates: Vec<Candidate<'a>>,
    pub(super) local_operations: Vec<LocalOperationCandidate<'a>>,
}

impl<'a> CandidateIndex<'a> {
    pub(super) fn from_health(response: &'a HealthResponse) -> Self {
        let mut candidates = Vec::new();
        let mut local_operations = Vec::new();
        for (file, health) in &response.files {
            let impl_method_ranges = health
                .ast
                .impls
                .iter()
                .flat_map(|impl_block| impl_block.methods.iter())
                .map(|method| (method.location.byte_start, method.location.byte_end))
                .collect::<BTreeSet<_>>();
            let functions = function_definitions(health);
            let nested_function_ranges = functions
                .iter()
                .filter(|definition| nearest_function_container(definition, &functions).is_some())
                .map(|definition| (definition.location.byte_start, definition.location.byte_end))
                .collect::<BTreeSet<_>>();

            for definition in &functions {
                let Some(container) = nearest_function_container(definition, &functions) else {
                    continue;
                };
                if !is_local_operation_container_name(&container.name) {
                    continue;
                }
                let Some(info) = local_operation_info(&definition.name) else {
                    continue;
                };
                let Some(operation_family) = info.operation_family else {
                    continue;
                };
                local_operations.push(LocalOperationCandidate {
                    file,
                    name: &definition.name,
                    container_name: &container.name,
                    container_kind: "function-declaration",
                    operation_family,
                    location: &definition.location,
                    container_location: &container.location,
                    path: &health.path,
                });
            }

            candidates.extend(
                health
                    .ast
                    .definitions
                    .iter()
                    .filter(|definition| {
                        if definition.kind != AstDefinitionKind::Function {
                            return true;
                        }
                        let range = (definition.location.byte_start, definition.location.byte_end);
                        !impl_method_ranges.contains(&range)
                            && !nested_function_ranges.contains(&range)
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
        local_operations.sort_by(|left, right| {
            left.file
                .cmp(right.file)
                .then(left.container_name.cmp(right.container_name))
                .then(left.name.cmp(right.name))
                .then(left.location.byte_start.cmp(&right.location.byte_start))
        });
        Self {
            candidates,
            local_operations,
        }
    }
}

fn function_definitions(health: &FileHealth) -> Vec<&AstDefinition> {
    health
        .ast
        .definitions
        .iter()
        .filter(|definition| definition.kind == AstDefinitionKind::Function)
        .collect()
}

fn nearest_function_container<'a>(
    child: &AstDefinition,
    functions: &[&'a AstDefinition],
) -> Option<&'a AstDefinition> {
    functions
        .iter()
        .copied()
        .filter(|candidate| location_contains(&candidate.location, &child.location))
        .min_by_key(|candidate| {
            candidate
                .location
                .byte_end
                .saturating_sub(candidate.location.byte_start)
        })
}

fn location_contains(container: &Location, child: &Location) -> bool {
    container.byte_start < child.byte_start && child.byte_end <= container.byte_end
}
