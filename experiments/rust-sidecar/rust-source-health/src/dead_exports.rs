use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{
    AstDefinition, AstDefinitionAttributeKind, AstDefinitionKind, AstImplBlock, AstImplMethod,
    AstOpaqueSurface, AstOpaqueVisibility, AstVisibility, FileHealth, RustUnusedDefinitionAction,
    RustUnusedDefinitionAnalysis, RustUnusedDefinitionCandidate, RustUnusedDefinitionCandidateKind,
    RustUnusedDefinitionDefinition, RustUnusedDefinitionDegradedScope,
    RustUnusedDefinitionEvidence, RustUnusedDefinitionObservedReferences,
    RustUnusedDefinitionOwner, RustUnusedDefinitionPolicy, RustUnusedDefinitionSummary,
    RustUnusedDefinitionTier, RUST_UNUSED_DEFINITION_CANDIDATE_COUNT_SCOPE,
    RUST_UNUSED_DEFINITION_CFG_BLOCKER, RUST_UNUSED_DEFINITION_CFG_GATE,
    RUST_UNUSED_DEFINITION_DERIVE_BLOCKER, RUST_UNUSED_DEFINITION_DERIVE_GATE,
    RUST_UNUSED_DEFINITION_FFI_BLOCKER, RUST_UNUSED_DEFINITION_FFI_GATE,
    RUST_UNUSED_DEFINITION_FP_GATE_NAMESPACE, RUST_UNUSED_DEFINITION_OPAQUE_BLOCKER,
    RUST_UNUSED_DEFINITION_OPAQUE_GATE, RUST_UNUSED_DEFINITION_POLICY_ID,
    RUST_UNUSED_DEFINITION_PUBLIC_SURFACE_BLOCKER, RUST_UNUSED_DEFINITION_PUBLIC_SURFACE_GATE,
    RUST_UNUSED_DEFINITION_QUALIFIED_PATH_REF_SCOPE, RUST_UNUSED_DEFINITION_SAFE_ACTION_SCOPE,
    RUST_UNUSED_DEFINITION_TEST_ONLY_BLOCKER, RUST_UNUSED_DEFINITION_TEST_ONLY_GATE,
    RUST_UNUSED_DEFINITION_TRAIT_IMPL_BLOCKER, RUST_UNUSED_DEFINITION_TRAIT_IMPL_GATE,
    RUST_UNUSED_DEFINITION_TS_MODEL,
};

pub fn classify_unused_definitions(
    files: &BTreeMap<String, FileHealth>,
) -> RustUnusedDefinitionAnalysis {
    let observed_references = observed_qualified_path_ref_counts(files, false);
    let test_only_references = observed_qualified_path_ref_counts(files, true);
    let mut summary = RustUnusedDefinitionSummary::default();
    let mut excluded_candidates = Vec::new();
    let mut degraded_scopes = Vec::new();

    for (file, health) in files {
        if !health.parse.ok {
            summary.degraded_count += 1;
            degraded_scopes.push(RustUnusedDefinitionDegradedScope {
                kind: "parse-error-file".to_string(),
                file: file.clone(),
                message: "dead-export absence claims are not grounded for files with parse errors"
                    .to_string(),
            });
        }
        for definition in &health.ast.definitions {
            summary.definition_count += 1;
            let production_refs = observed_references
                .get(&definition.name)
                .copied()
                .unwrap_or_default();
            let test_only_refs = test_only_references
                .get(&definition.name)
                .copied()
                .unwrap_or_default();
            if definition.visibility == AstVisibility::Public && production_refs == 0 {
                push_public_definition_candidate(
                    file,
                    health,
                    definition,
                    test_only_refs,
                    &mut summary,
                    &mut excluded_candidates,
                );
            }
        }
        push_trait_impl_candidates(
            file,
            &health.ast.impls,
            &mut summary,
            &mut excluded_candidates,
        );
    }

    RustUnusedDefinitionAnalysis {
        policy: RustUnusedDefinitionPolicy {
            policy_id: RUST_UNUSED_DEFINITION_POLICY_ID.to_string(),
            ts_model: RUST_UNUSED_DEFINITION_TS_MODEL.to_string(),
            rust_fp_gate_namespace: RUST_UNUSED_DEFINITION_FP_GATE_NAMESPACE.to_string(),
            candidate_count_scope: RUST_UNUSED_DEFINITION_CANDIDATE_COUNT_SCOPE.to_string(),
            safe_action_scope: RUST_UNUSED_DEFINITION_SAFE_ACTION_SCOPE.to_string(),
        },
        summary,
        findings: Vec::new(),
        excluded_candidates,
        degraded_scopes,
    }
}

fn observed_qualified_path_ref_counts(
    files: &BTreeMap<String, FileHealth>,
    test_context: bool,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for health in files.values() {
        let mut names_in_file = BTreeSet::new();
        for path_ref in &health.ast.path_refs {
            if path_ref.test_context == test_context {
                names_in_file.insert(path_ref.name.clone());
            }
        }
        for name in names_in_file {
            *counts.entry(name).or_insert(0) += 1;
        }
    }
    counts
}

fn push_public_definition_candidate(
    file: &str,
    health: &FileHealth,
    definition: &AstDefinition,
    test_only_refs: usize,
    summary: &mut RustUnusedDefinitionSummary,
    excluded_candidates: &mut Vec<RustUnusedDefinitionCandidate>,
) {
    if has_attribute_kind(definition, AstDefinitionAttributeKind::FfiLinker) {
        summary.blocked_ffi_count += 1;
        excluded_candidates.push(definition_candidate(
            file,
            definition,
            CandidateGate::new(
                RustUnusedDefinitionTier::Review,
                RustUnusedDefinitionAction::Review,
                RUST_UNUSED_DEFINITION_FFI_GATE,
                RUST_UNUSED_DEFINITION_FFI_BLOCKER,
            ),
            test_only_refs,
            Vec::new(),
        ));
    } else if has_attribute_kind(definition, AstDefinitionAttributeKind::Cfg) {
        summary.blocked_cfg_count += 1;
        summary.degraded_count += 1;
        excluded_candidates.push(definition_candidate(
            file,
            definition,
            CandidateGate::new(
                RustUnusedDefinitionTier::Degraded,
                RustUnusedDefinitionAction::Degraded,
                RUST_UNUSED_DEFINITION_CFG_GATE,
                RUST_UNUSED_DEFINITION_CFG_BLOCKER,
            ),
            test_only_refs,
            Vec::new(),
        ));
    } else if has_attribute_kind(definition, AstDefinitionAttributeKind::Derive) {
        summary.blocked_derive_surface_count += 1;
        excluded_candidates.push(definition_candidate(
            file,
            definition,
            CandidateGate::new(
                RustUnusedDefinitionTier::Review,
                RustUnusedDefinitionAction::Review,
                RUST_UNUSED_DEFINITION_DERIVE_GATE,
                RUST_UNUSED_DEFINITION_DERIVE_BLOCKER,
            ),
            test_only_refs,
            definition_attribute_evidence(definition, AstDefinitionAttributeKind::Derive),
        ));
    } else if let Some(surface) = review_opaque_surface(&health.ast.opaque_surfaces) {
        summary.blocked_opaque_count += 1;
        excluded_candidates.push(definition_candidate(
            file,
            definition,
            CandidateGate::new(
                RustUnusedDefinitionTier::Review,
                RustUnusedDefinitionAction::Review,
                RUST_UNUSED_DEFINITION_OPAQUE_GATE,
                RUST_UNUSED_DEFINITION_OPAQUE_BLOCKER,
            ),
            test_only_refs,
            vec![RustUnusedDefinitionEvidence {
                kind: "review-opaque-surface".to_string(),
                message: format!(
                    "review-visible opaque syntax '{}' prevents grounded dead-export absence claims",
                    surface.detail
                ),
            }],
        ));
    } else if test_only_refs > 0 {
        summary.test_only_support_count += 1;
        excluded_candidates.push(definition_candidate(
            file,
            definition,
            CandidateGate::new(
                RustUnusedDefinitionTier::Review,
                RustUnusedDefinitionAction::Review,
                RUST_UNUSED_DEFINITION_TEST_ONLY_GATE,
                RUST_UNUSED_DEFINITION_TEST_ONLY_BLOCKER,
            ),
            test_only_refs,
            Vec::new(),
        ));
    } else {
        summary.blocked_public_surface_count += 1;
        excluded_candidates.push(definition_candidate(
            file,
            definition,
            CandidateGate::new(
                RustUnusedDefinitionTier::Review,
                RustUnusedDefinitionAction::DemoteToRestricted,
                RUST_UNUSED_DEFINITION_PUBLIC_SURFACE_GATE,
                RUST_UNUSED_DEFINITION_PUBLIC_SURFACE_BLOCKER,
            ),
            0,
            Vec::new(),
        ));
    }
}

fn push_trait_impl_candidates(
    file: &str,
    impls: &[AstImplBlock],
    summary: &mut RustUnusedDefinitionSummary,
    excluded_candidates: &mut Vec<RustUnusedDefinitionCandidate>,
) {
    for impl_block in impls {
        if impl_block.trait_path.is_none() {
            continue;
        }
        for method in &impl_block.methods {
            summary.blocked_trait_impl_count += 1;
            excluded_candidates.push(trait_impl_candidate(file, method));
        }
    }
}

fn definition_candidate(
    file: &str,
    definition: &AstDefinition,
    gate: CandidateGate<'_>,
    test_only_refs: usize,
    evidence: Vec<RustUnusedDefinitionEvidence>,
) -> RustUnusedDefinitionCandidate {
    RustUnusedDefinitionCandidate {
        kind: RustUnusedDefinitionCandidateKind::RustUnusedDefinition,
        tier: gate.tier,
        action: gate.action,
        definition: RustUnusedDefinitionDefinition {
            file: file.to_string(),
            name: definition.name.clone(),
            kind: definition.kind,
            visibility: definition.visibility,
            owner: RustUnusedDefinitionOwner::Module,
            location: definition.location.clone(),
        },
        observed_references: RustUnusedDefinitionObservedReferences {
            production: 0,
            test_only: test_only_refs,
            searched_scopes: vec![RUST_UNUSED_DEFINITION_QUALIFIED_PATH_REF_SCOPE.to_string()],
        },
        fp_gates: vec![gate.gate.to_string()],
        action_blockers: vec![gate.blocker.to_string()],
        safe_action: None,
        evidence,
    }
}

struct CandidateGate<'a> {
    tier: RustUnusedDefinitionTier,
    action: RustUnusedDefinitionAction,
    gate: &'a str,
    blocker: &'a str,
}

impl<'a> CandidateGate<'a> {
    fn new(
        tier: RustUnusedDefinitionTier,
        action: RustUnusedDefinitionAction,
        gate: &'a str,
        blocker: &'a str,
    ) -> Self {
        Self {
            tier,
            action,
            gate,
            blocker,
        }
    }
}

fn trait_impl_candidate(file: &str, method: &AstImplMethod) -> RustUnusedDefinitionCandidate {
    RustUnusedDefinitionCandidate {
        kind: RustUnusedDefinitionCandidateKind::RustUnusedDefinition,
        tier: RustUnusedDefinitionTier::Review,
        action: RustUnusedDefinitionAction::Review,
        definition: RustUnusedDefinitionDefinition {
            file: file.to_string(),
            name: method.name.clone(),
            kind: AstDefinitionKind::Function,
            visibility: method.visibility,
            owner: RustUnusedDefinitionOwner::TraitImpl,
            location: method.location.clone(),
        },
        observed_references: RustUnusedDefinitionObservedReferences {
            production: 0,
            test_only: 0,
            searched_scopes: vec![RUST_UNUSED_DEFINITION_QUALIFIED_PATH_REF_SCOPE.to_string()],
        },
        fp_gates: vec![RUST_UNUSED_DEFINITION_TRAIT_IMPL_GATE.to_string()],
        action_blockers: vec![RUST_UNUSED_DEFINITION_TRAIT_IMPL_BLOCKER.to_string()],
        safe_action: None,
        evidence: Vec::new(),
    }
}

fn has_attribute_kind(definition: &AstDefinition, kind: AstDefinitionAttributeKind) -> bool {
    definition
        .attributes
        .iter()
        .any(|attribute| attribute.kind == kind)
}

fn definition_attribute_evidence(
    definition: &AstDefinition,
    kind: AstDefinitionAttributeKind,
) -> Vec<RustUnusedDefinitionEvidence> {
    definition
        .attributes
        .iter()
        .filter(|attribute| attribute.kind == kind)
        .map(|attribute| RustUnusedDefinitionEvidence {
            kind: "definition-attribute".to_string(),
            message: format!(
                "definition attribute '{}' prevents grounded dead-export absence claims",
                attribute
                    .text
                    .trim_start_matches("#[")
                    .trim_end_matches(']')
            ),
        })
        .collect()
}

fn review_opaque_surface(surfaces: &[AstOpaqueSurface]) -> Option<&AstOpaqueSurface> {
    surfaces
        .iter()
        .find(|surface| surface.visibility.visibility() == AstOpaqueVisibility::Review)
}
