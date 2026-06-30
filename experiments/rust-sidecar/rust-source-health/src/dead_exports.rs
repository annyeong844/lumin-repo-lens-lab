use std::collections::BTreeMap;

use crate::analyzer::{
    CompactDeadDefinition, CompactDeadDefinitionAttribute, CompactDeadFile, CompactDeadImplBlock,
    CompactDeadImplMethod,
};
use crate::protocol::{
    AstDefinition, AstDefinitionAttributeKind, AstDefinitionKind, AstDefinitionOwner, AstImplBlock,
    AstImplMethod, AstOpaqueSurface, AstOpaqueVisibility, AstVisibility, FileHealth, Location,
    PathClassification, RustUnusedDefinitionAction, RustUnusedDefinitionAnalysis,
    RustUnusedDefinitionCandidate, RustUnusedDefinitionCandidateKind,
    RustUnusedDefinitionDefinition, RustUnusedDefinitionDegradedScope,
    RustUnusedDefinitionEvidence, RustUnusedDefinitionExcludedCandidateProjection,
    RustUnusedDefinitionObservedReferences, RustUnusedDefinitionOwner, RustUnusedDefinitionPolicy,
    RustUnusedDefinitionSummary, RustUnusedDefinitionTier,
    RUST_UNUSED_DEFINITION_CANDIDATE_COUNT_SCOPE, RUST_UNUSED_DEFINITION_CFG_BLOCKER,
    RUST_UNUSED_DEFINITION_CFG_GATE, RUST_UNUSED_DEFINITION_DERIVE_BLOCKER,
    RUST_UNUSED_DEFINITION_DERIVE_GATE, RUST_UNUSED_DEFINITION_ENTRYPOINT_BLOCKER,
    RUST_UNUSED_DEFINITION_ENTRYPOINT_GATE, RUST_UNUSED_DEFINITION_EXCLUDED_CANDIDATE_COUNT_SCOPE,
    RUST_UNUSED_DEFINITION_EXCLUDED_CANDIDATE_EXAMPLE_LIMIT, RUST_UNUSED_DEFINITION_FFI_BLOCKER,
    RUST_UNUSED_DEFINITION_FFI_GATE, RUST_UNUSED_DEFINITION_FP_GATE_NAMESPACE,
    RUST_UNUSED_DEFINITION_GENERATED_BLOCKER, RUST_UNUSED_DEFINITION_GENERATED_GATE,
    RUST_UNUSED_DEFINITION_LOCAL_REF_SCOPE, RUST_UNUSED_DEFINITION_OPAQUE_BLOCKER,
    RUST_UNUSED_DEFINITION_OPAQUE_GATE, RUST_UNUSED_DEFINITION_POLICY_ID,
    RUST_UNUSED_DEFINITION_PUBLIC_SURFACE_BLOCKER, RUST_UNUSED_DEFINITION_PUBLIC_SURFACE_GATE,
    RUST_UNUSED_DEFINITION_SAFE_ACTION_SCOPE, RUST_UNUSED_DEFINITION_TEST_ONLY_BLOCKER,
    RUST_UNUSED_DEFINITION_TEST_ONLY_GATE, RUST_UNUSED_DEFINITION_TRAIT_IMPL_BLOCKER,
    RUST_UNUSED_DEFINITION_TRAIT_IMPL_GATE, RUST_UNUSED_DEFINITION_TS_MODEL,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct UnusedDefinitionAnalysisOptions {
    excluded_candidate_example_limit: Option<usize>,
}

impl UnusedDefinitionAnalysisOptions {
    pub(crate) const fn full_artifact() -> Self {
        Self {
            excluded_candidate_example_limit: None,
        }
    }

    pub(crate) const fn compact_artifact() -> Self {
        Self {
            excluded_candidate_example_limit: Some(
                RUST_UNUSED_DEFINITION_EXCLUDED_CANDIDATE_EXAMPLE_LIMIT,
            ),
        }
    }
}

pub(crate) fn classify_unused_definitions_with_options(
    files: &BTreeMap<String, FileHealth>,
    options: UnusedDefinitionAnalysisOptions,
) -> RustUnusedDefinitionAnalysis {
    classify_unused_definition_files_with_options(files, options)
}

pub(crate) fn classify_compact_unused_definitions_with_options(
    files: &BTreeMap<String, CompactDeadFile>,
    options: UnusedDefinitionAnalysisOptions,
) -> RustUnusedDefinitionAnalysis {
    classify_unused_definition_files_with_options(files, options)
}

fn classify_unused_definition_files_with_options<F: UnusedDefinitionFile>(
    files: &BTreeMap<String, F>,
    options: UnusedDefinitionAnalysisOptions,
) -> RustUnusedDefinitionAnalysis {
    let observed_references = observed_qualified_path_ref_counts(files, false);
    let test_only_references = observed_qualified_path_ref_counts(files, true);
    let mut summary = RustUnusedDefinitionSummary::default();
    let mut findings = Vec::new();
    let mut excluded_candidates =
        ExcludedCandidateCollector::new(options.excluded_candidate_example_limit);
    let mut degraded_scopes = Vec::new();

    for (file, health) in files {
        if !health.parse_ok() {
            summary.degraded_count += 1;
            degraded_scopes.push(RustUnusedDefinitionDegradedScope {
                kind: "parse-error-file".to_string(),
                file: file.clone(),
                message: "dead-export absence claims are not grounded for files with parse errors"
                    .to_string(),
            });
        }
        for definition in health.definitions() {
            summary.definition_count += 1;
            let production_refs = observed_references
                .get(definition.name())
                .copied()
                .unwrap_or_default();
            let test_only_refs = test_only_references
                .get(definition.name())
                .copied()
                .unwrap_or_default();
            if is_public_surface_owner(definition.owner())
                && is_public_surface_visibility(definition.visibility())
                && production_refs == 0
            {
                push_public_definition_candidate(
                    file,
                    health,
                    definition,
                    test_only_refs,
                    &mut summary,
                    &mut excluded_candidates,
                );
            } else if definition.owner() == AstDefinitionOwner::Module
                && definition.visibility() == AstVisibility::Private
                && production_refs == 0
            {
                push_private_definition_candidate(
                    file,
                    health,
                    definition,
                    test_only_refs,
                    &mut summary,
                    &mut findings,
                    &mut excluded_candidates,
                );
            }
        }
        push_trait_impl_candidates(file, health.impls(), &mut summary, &mut excluded_candidates);
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
        findings,
        excluded_candidate_projection: excluded_candidates.projection(),
        excluded_candidates: excluded_candidates.into_candidates(),
        degraded_scopes,
    }
}

trait UnusedDefinitionFile {
    type Definition: UnusedDefinitionDefinitionView;
    type ImplBlock: UnusedDefinitionImplBlockView;

    fn parse_ok(&self) -> bool;
    fn classifications(&self) -> &[PathClassification];
    fn definitions(&self) -> &[Self::Definition];
    fn impls(&self) -> &[Self::ImplBlock];
    fn count_ref_names(&self, test_context: bool, counts: &mut BTreeMap<String, usize>);
    fn review_opaque_surface(&self) -> Option<&AstOpaqueSurface>;
}

trait UnusedDefinitionDefinitionView {
    type Attribute: UnusedDefinitionAttributeView;

    fn kind(&self) -> AstDefinitionKind;
    fn name(&self) -> &str;
    fn visibility(&self) -> AstVisibility;
    fn owner(&self) -> AstDefinitionOwner;
    fn test_context(&self) -> bool;
    fn attributes(&self) -> &[Self::Attribute];
    fn location(&self) -> &Location;
}

trait UnusedDefinitionAttributeView {
    fn kind(&self) -> AstDefinitionAttributeKind;
    fn text(&self) -> &str;
}

trait UnusedDefinitionImplBlockView {
    type Method: UnusedDefinitionImplMethodView;

    fn trait_path(&self) -> Option<&str>;
    fn methods(&self) -> &[Self::Method];
}

trait UnusedDefinitionImplMethodView {
    fn name(&self) -> &str;
    fn visibility(&self) -> AstVisibility;
    fn location(&self) -> &Location;
}

impl UnusedDefinitionFile for FileHealth {
    type Definition = AstDefinition;
    type ImplBlock = AstImplBlock;

    fn parse_ok(&self) -> bool {
        self.parse.ok
    }

    fn classifications(&self) -> &[PathClassification] {
        &self.path.classifications
    }

    fn definitions(&self) -> &[Self::Definition] {
        &self.ast.definitions
    }

    fn impls(&self) -> &[Self::ImplBlock] {
        &self.ast.impls
    }

    fn count_ref_names(&self, test_context: bool, counts: &mut BTreeMap<String, usize>) {
        let names = if test_context {
            &self.ast.test_local_ref_names
        } else {
            &self.ast.local_ref_names
        };
        for name in names {
            *counts.entry(name.clone()).or_insert(0) += 1;
        }
    }

    fn review_opaque_surface(&self) -> Option<&AstOpaqueSurface> {
        review_opaque_surface(&self.ast.opaque_surfaces)
    }
}

impl UnusedDefinitionFile for CompactDeadFile {
    type Definition = CompactDeadDefinition;
    type ImplBlock = CompactDeadImplBlock;

    fn parse_ok(&self) -> bool {
        self.parse_ok
    }

    fn classifications(&self) -> &[PathClassification] {
        &self.path_classifications
    }

    fn definitions(&self) -> &[Self::Definition] {
        &self.definitions
    }

    fn impls(&self) -> &[Self::ImplBlock] {
        &self.impls
    }

    fn count_ref_names(&self, test_context: bool, counts: &mut BTreeMap<String, usize>) {
        let names = if test_context {
            &self.test_local_ref_names
        } else {
            &self.local_ref_names
        };
        for name in names {
            *counts.entry(name.to_string()).or_insert(0) += 1;
        }
    }

    fn review_opaque_surface(&self) -> Option<&AstOpaqueSurface> {
        self.review_opaque_surface.as_ref()
    }
}

impl UnusedDefinitionDefinitionView for AstDefinition {
    type Attribute = crate::protocol::AstDefinitionAttribute;

    fn kind(&self) -> AstDefinitionKind {
        self.kind
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn visibility(&self) -> AstVisibility {
        self.visibility
    }

    fn owner(&self) -> AstDefinitionOwner {
        self.owner
    }

    fn test_context(&self) -> bool {
        self.test_context
    }

    fn attributes(&self) -> &[Self::Attribute] {
        &self.attributes
    }

    fn location(&self) -> &Location {
        &self.location
    }
}

impl UnusedDefinitionDefinitionView for CompactDeadDefinition {
    type Attribute = CompactDeadDefinitionAttribute;

    fn kind(&self) -> AstDefinitionKind {
        self.kind
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn visibility(&self) -> AstVisibility {
        self.visibility
    }

    fn owner(&self) -> AstDefinitionOwner {
        self.owner
    }

    fn test_context(&self) -> bool {
        self.test_context
    }

    fn attributes(&self) -> &[Self::Attribute] {
        &self.attributes
    }

    fn location(&self) -> &Location {
        &self.location
    }
}

impl UnusedDefinitionAttributeView for crate::protocol::AstDefinitionAttribute {
    fn kind(&self) -> AstDefinitionAttributeKind {
        self.kind
    }

    fn text(&self) -> &str {
        &self.text
    }
}

impl UnusedDefinitionAttributeView for CompactDeadDefinitionAttribute {
    fn kind(&self) -> AstDefinitionAttributeKind {
        self.kind
    }

    fn text(&self) -> &str {
        &self.text
    }
}

impl UnusedDefinitionImplBlockView for AstImplBlock {
    type Method = AstImplMethod;

    fn trait_path(&self) -> Option<&str> {
        self.trait_path.as_deref()
    }

    fn methods(&self) -> &[Self::Method] {
        &self.methods
    }
}

impl UnusedDefinitionImplBlockView for CompactDeadImplBlock {
    type Method = CompactDeadImplMethod;

    fn trait_path(&self) -> Option<&str> {
        self.trait_path.as_deref()
    }

    fn methods(&self) -> &[Self::Method] {
        &self.methods
    }
}

impl UnusedDefinitionImplMethodView for AstImplMethod {
    fn name(&self) -> &str {
        &self.name
    }

    fn visibility(&self) -> AstVisibility {
        self.visibility
    }

    fn location(&self) -> &Location {
        &self.location
    }
}

impl UnusedDefinitionImplMethodView for CompactDeadImplMethod {
    fn name(&self) -> &str {
        &self.name
    }

    fn visibility(&self) -> AstVisibility {
        self.visibility
    }

    fn location(&self) -> &Location {
        &self.location
    }
}

struct ExcludedCandidateCollector {
    candidates: Vec<RustUnusedDefinitionCandidate>,
    example_limit: Option<usize>,
    total_count: usize,
}

impl ExcludedCandidateCollector {
    fn new(example_limit: Option<usize>) -> Self {
        Self {
            candidates: Vec::new(),
            example_limit,
            total_count: 0,
        }
    }

    fn push_with(&mut self, candidate: impl FnOnce() -> RustUnusedDefinitionCandidate) {
        self.total_count += 1;
        if self
            .example_limit
            .is_none_or(|limit| self.candidates.len() < limit)
        {
            self.candidates.push(candidate());
        }
    }

    fn projection(&self) -> RustUnusedDefinitionExcludedCandidateProjection {
        RustUnusedDefinitionExcludedCandidateProjection {
            count_scope: RUST_UNUSED_DEFINITION_EXCLUDED_CANDIDATE_COUNT_SCOPE.to_string(),
            total_count: self.total_count,
            retained_count: self.candidates.len(),
            example_limit: self.example_limit,
        }
    }

    fn into_candidates(self) -> Vec<RustUnusedDefinitionCandidate> {
        self.candidates
    }
}

fn observed_qualified_path_ref_counts<F: UnusedDefinitionFile>(
    files: &BTreeMap<String, F>,
    test_context: bool,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for health in files.values() {
        health.count_ref_names(test_context, &mut counts);
    }
    counts
}

fn push_public_definition_candidate(
    file: &str,
    health: &impl UnusedDefinitionFile,
    definition: &impl UnusedDefinitionDefinitionView,
    test_only_refs: usize,
    summary: &mut RustUnusedDefinitionSummary,
    excluded_candidates: &mut ExcludedCandidateCollector,
) {
    if has_attribute_kind(definition, AstDefinitionAttributeKind::FfiLinker) {
        summary.blocked_ffi_count += 1;
        excluded_candidates.push_with(|| {
            definition_candidate(
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
            )
        });
    } else if has_attribute_kind(definition, AstDefinitionAttributeKind::Cfg) {
        summary.blocked_cfg_count += 1;
        summary.degraded_count += 1;
        excluded_candidates.push_with(|| {
            definition_candidate(
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
            )
        });
    } else if has_attribute_kind(definition, AstDefinitionAttributeKind::Derive) {
        summary.blocked_derive_surface_count += 1;
        excluded_candidates.push_with(|| {
            definition_candidate(
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
            )
        });
    } else if let Some(surface) = health.review_opaque_surface() {
        summary.blocked_opaque_count += 1;
        excluded_candidates.push_with(|| definition_candidate(
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
        excluded_candidates.push_with(|| {
            definition_candidate(
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
            )
        });
    } else {
        summary.blocked_public_surface_count += 1;
        excluded_candidates.push_with(|| {
            definition_candidate(
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
            )
        });
    }
}

fn push_private_definition_candidate(
    file: &str,
    health: &impl UnusedDefinitionFile,
    definition: &impl UnusedDefinitionDefinitionView,
    test_only_refs: usize,
    summary: &mut RustUnusedDefinitionSummary,
    findings: &mut Vec<RustUnusedDefinitionCandidate>,
    excluded_candidates: &mut ExcludedCandidateCollector,
) {
    if !health.parse_ok() {
        return;
    }
    if !is_supported_private_candidate_kind(definition.kind()) {
        return;
    }
    if has_path_classification(health, PathClassification::Test) {
        summary.test_only_support_count += 1;
        excluded_candidates.push_with(|| {
            definition_candidate(
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
            )
        });
    } else if has_path_classification(health, PathClassification::Generated) {
        summary.blocked_generated_count += 1;
        excluded_candidates.push_with(|| {
            definition_candidate(
                file,
                definition,
                CandidateGate::new(
                    RustUnusedDefinitionTier::Muted,
                    RustUnusedDefinitionAction::Muted,
                    RUST_UNUSED_DEFINITION_GENERATED_GATE,
                    RUST_UNUSED_DEFINITION_GENERATED_BLOCKER,
                ),
                test_only_refs,
                Vec::new(),
            )
        });
    } else if is_rust_entrypoint(file, definition) {
        summary.blocked_entrypoint_count += 1;
        excluded_candidates.push_with(|| {
            definition_candidate(
                file,
                definition,
                CandidateGate::new(
                    RustUnusedDefinitionTier::Review,
                    RustUnusedDefinitionAction::Review,
                    RUST_UNUSED_DEFINITION_ENTRYPOINT_GATE,
                    RUST_UNUSED_DEFINITION_ENTRYPOINT_BLOCKER,
                ),
                test_only_refs,
                Vec::new(),
            )
        });
    } else if has_attribute_kind(definition, AstDefinitionAttributeKind::FfiLinker) {
        summary.blocked_ffi_count += 1;
        excluded_candidates.push_with(|| {
            definition_candidate(
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
            )
        });
    } else if has_attribute_kind(definition, AstDefinitionAttributeKind::Cfg) {
        summary.blocked_cfg_count += 1;
        summary.degraded_count += 1;
        excluded_candidates.push_with(|| {
            definition_candidate(
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
            )
        });
    } else if has_attribute_kind(definition, AstDefinitionAttributeKind::Derive) {
        summary.blocked_derive_surface_count += 1;
        excluded_candidates.push_with(|| {
            definition_candidate(
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
            )
        });
    } else if let Some(surface) = health.review_opaque_surface() {
        summary.blocked_opaque_count += 1;
        excluded_candidates.push_with(|| definition_candidate(
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
    } else if definition.test_context()
        || has_attribute_kind(definition, AstDefinitionAttributeKind::Test)
        || test_only_refs > 0
    {
        summary.test_only_support_count += 1;
        excluded_candidates.push_with(|| {
            definition_candidate(
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
            )
        });
    } else {
        summary.candidate_count += 1;
        findings.push(remove_candidate(file, definition));
    }
}

fn push_trait_impl_candidates(
    file: &str,
    impls: &[impl UnusedDefinitionImplBlockView],
    summary: &mut RustUnusedDefinitionSummary,
    excluded_candidates: &mut ExcludedCandidateCollector,
) {
    for impl_block in impls {
        if impl_block.trait_path().is_none() {
            continue;
        }
        for method in impl_block.methods() {
            summary.blocked_trait_impl_count += 1;
            excluded_candidates.push_with(|| trait_impl_candidate(file, method));
        }
    }
}

fn remove_candidate(
    file: &str,
    definition: &impl UnusedDefinitionDefinitionView,
) -> RustUnusedDefinitionCandidate {
    RustUnusedDefinitionCandidate {
        kind: RustUnusedDefinitionCandidateKind::RustUnusedDefinition,
        tier: RustUnusedDefinitionTier::RemoveCandidate,
        action: RustUnusedDefinitionAction::RemoveCandidate,
        definition: RustUnusedDefinitionDefinition {
            file: file.to_string(),
            name: definition.name().to_string(),
            kind: definition.kind(),
            visibility: definition.visibility(),
            owner: unused_definition_owner(definition.owner()),
            location: definition.location().clone(),
        },
        observed_references: RustUnusedDefinitionObservedReferences {
            production: 0,
            test_only: 0,
            searched_scopes: vec![RUST_UNUSED_DEFINITION_LOCAL_REF_SCOPE.to_string()],
        },
        fp_gates: Vec::new(),
        action_blockers: Vec::new(),
        safe_action: None,
        evidence: Vec::new(),
    }
}

fn definition_candidate(
    file: &str,
    definition: &impl UnusedDefinitionDefinitionView,
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
            name: definition.name().to_string(),
            kind: definition.kind(),
            visibility: definition.visibility(),
            owner: unused_definition_owner(definition.owner()),
            location: definition.location().clone(),
        },
        observed_references: RustUnusedDefinitionObservedReferences {
            production: 0,
            test_only: test_only_refs,
            searched_scopes: vec![RUST_UNUSED_DEFINITION_LOCAL_REF_SCOPE.to_string()],
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

fn trait_impl_candidate(
    file: &str,
    method: &impl UnusedDefinitionImplMethodView,
) -> RustUnusedDefinitionCandidate {
    RustUnusedDefinitionCandidate {
        kind: RustUnusedDefinitionCandidateKind::RustUnusedDefinition,
        tier: RustUnusedDefinitionTier::Review,
        action: RustUnusedDefinitionAction::Review,
        definition: RustUnusedDefinitionDefinition {
            file: file.to_string(),
            name: method.name().to_string(),
            kind: AstDefinitionKind::Function,
            visibility: method.visibility(),
            owner: RustUnusedDefinitionOwner::TraitImpl,
            location: method.location().clone(),
        },
        observed_references: RustUnusedDefinitionObservedReferences {
            production: 0,
            test_only: 0,
            searched_scopes: vec![RUST_UNUSED_DEFINITION_LOCAL_REF_SCOPE.to_string()],
        },
        fp_gates: vec![RUST_UNUSED_DEFINITION_TRAIT_IMPL_GATE.to_string()],
        action_blockers: vec![RUST_UNUSED_DEFINITION_TRAIT_IMPL_BLOCKER.to_string()],
        safe_action: None,
        evidence: Vec::new(),
    }
}

fn is_public_surface_visibility(visibility: AstVisibility) -> bool {
    matches!(
        visibility,
        AstVisibility::Public | AstVisibility::Crate | AstVisibility::Restricted
    )
}

fn is_public_surface_owner(owner: AstDefinitionOwner) -> bool {
    matches!(
        owner,
        AstDefinitionOwner::Module | AstDefinitionOwner::InherentImpl
    )
}

fn unused_definition_owner(owner: AstDefinitionOwner) -> RustUnusedDefinitionOwner {
    match owner {
        AstDefinitionOwner::Module => RustUnusedDefinitionOwner::Module,
        AstDefinitionOwner::InherentImpl => RustUnusedDefinitionOwner::InherentImpl,
        AstDefinitionOwner::TraitImpl => RustUnusedDefinitionOwner::TraitImpl,
        AstDefinitionOwner::Trait => RustUnusedDefinitionOwner::Unknown,
    }
}

fn is_supported_private_candidate_kind(kind: AstDefinitionKind) -> bool {
    matches!(
        kind,
        AstDefinitionKind::Function | AstDefinitionKind::Const | AstDefinitionKind::Static
    )
}

fn has_path_classification(
    health: &impl UnusedDefinitionFile,
    classification: PathClassification,
) -> bool {
    health.classifications().contains(&classification)
}

fn is_rust_entrypoint(file: &str, definition: &impl UnusedDefinitionDefinitionView) -> bool {
    definition.kind() == AstDefinitionKind::Function
        && definition.name() == "main"
        && (file == "build.rs" || file.ends_with("/build.rs") || file.ends_with("/main.rs"))
}

fn has_attribute_kind(
    definition: &impl UnusedDefinitionDefinitionView,
    kind: AstDefinitionAttributeKind,
) -> bool {
    definition
        .attributes()
        .iter()
        .any(|attribute| attribute.kind() == kind)
}

fn definition_attribute_evidence(
    definition: &impl UnusedDefinitionDefinitionView,
    kind: AstDefinitionAttributeKind,
) -> Vec<RustUnusedDefinitionEvidence> {
    definition
        .attributes()
        .iter()
        .filter(|attribute| attribute.kind() == kind)
        .map(|attribute| RustUnusedDefinitionEvidence {
            kind: "definition-attribute".to_string(),
            message: format!(
                "definition attribute '{}' prevents grounded dead-export absence claims",
                attribute
                    .text()
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
