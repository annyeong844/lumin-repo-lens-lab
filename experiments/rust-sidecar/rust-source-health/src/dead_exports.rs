use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{
    AstDefinition, AstVisibility, FileHealth, RustUnusedDefinitionAction,
    RustUnusedDefinitionAnalysis, RustUnusedDefinitionCandidate, RustUnusedDefinitionCandidateKind,
    RustUnusedDefinitionDefinition, RustUnusedDefinitionDegradedScope,
    RustUnusedDefinitionObservedReferences, RustUnusedDefinitionOwner, RustUnusedDefinitionPolicy,
    RustUnusedDefinitionSummary, RustUnusedDefinitionTier,
    RUST_UNUSED_DEFINITION_CANDIDATE_COUNT_SCOPE, RUST_UNUSED_DEFINITION_FP_GATE_NAMESPACE,
    RUST_UNUSED_DEFINITION_POLICY_ID, RUST_UNUSED_DEFINITION_PUBLIC_SURFACE_BLOCKER,
    RUST_UNUSED_DEFINITION_PUBLIC_SURFACE_GATE, RUST_UNUSED_DEFINITION_QUALIFIED_PATH_REF_SCOPE,
    RUST_UNUSED_DEFINITION_SAFE_ACTION_SCOPE, RUST_UNUSED_DEFINITION_TS_MODEL,
};

pub fn classify_unused_definitions(
    files: &BTreeMap<String, FileHealth>,
) -> RustUnusedDefinitionAnalysis {
    let observed_references = observed_qualified_path_ref_counts(files);
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
            if definition.visibility == AstVisibility::Public
                && !observed_references.contains_key(&definition.name)
            {
                summary.blocked_public_surface_count += 1;
                excluded_candidates.push(public_surface_candidate(file, definition));
            }
        }
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
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for health in files.values() {
        let mut names_in_file = BTreeSet::new();
        for path_ref in &health.ast.path_refs {
            names_in_file.insert(path_ref.name.clone());
        }
        for name in names_in_file {
            *counts.entry(name).or_insert(0) += 1;
        }
    }
    counts
}

fn public_surface_candidate(
    file: &str,
    definition: &AstDefinition,
) -> RustUnusedDefinitionCandidate {
    RustUnusedDefinitionCandidate {
        kind: RustUnusedDefinitionCandidateKind::RustUnusedDefinition,
        tier: RustUnusedDefinitionTier::Review,
        action: RustUnusedDefinitionAction::DemoteToRestricted,
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
            test_only: 0,
            searched_scopes: vec![RUST_UNUSED_DEFINITION_QUALIFIED_PATH_REF_SCOPE.to_string()],
        },
        fp_gates: vec![RUST_UNUSED_DEFINITION_PUBLIC_SURFACE_GATE.to_string()],
        action_blockers: vec![RUST_UNUSED_DEFINITION_PUBLIC_SURFACE_BLOCKER.to_string()],
        safe_action: None,
        evidence: Vec::new(),
    }
}
