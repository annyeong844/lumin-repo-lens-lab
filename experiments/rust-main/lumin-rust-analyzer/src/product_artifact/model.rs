use anyhow::{bail, Result};
use serde::Serialize;
use std::fmt::Debug;

use super::meta::ProductArtifactMeta;
use super::phases::PhaseBriefs;
use super::refs::ArtifactRefs;
use super::semantic::{ProductCoverageProjection, ProductOraclePlanProjection};
use crate::policy::{ActionPolicyProjection, OracleBridgeProjection, PolicyMetadata};
use crate::product_files::{
    ProductFilesProjection, ProductSemanticDiagnosticsProjection,
    ProductSemanticFindingsProjection, SemanticRefCounts,
};
use crate::product_summary::ProductSummary;

#[derive(Debug, Copy, Clone)]
pub(crate) struct PhaseTimings {
    pub(crate) syntax_ms: u128,
    pub(crate) semantic_ms: u128,
    pub(crate) analyzer_ms: u128,
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub(crate) struct ProductArtifact<'a> {
    pub(super) artifact: UnifiedArtifact<'a>,
}

impl ProductArtifact<'_> {
    pub(crate) fn validate_contract(&self) -> Result<()> {
        let expected_refs = SemanticRefCounts::new(
            self.artifact.semantic_findings.len(),
            self.artifact.semantic_diagnostics.len(),
        );
        if let Some(message) = self
            .artifact
            .files
            .first_invalid_semantic_ref(expected_refs)
        {
            bail!("blocked-artifact-contract: {message}");
        }
        let linked_refs = self.artifact.files.semantic_ref_counts();
        let unlinked_refs = self.artifact.summary.semantic_unlinked_refs();
        require_equal_contract(
            "files.semantic.findings.length + summary.semanticUnlinkedFindings",
            linked_refs.findings() + unlinked_refs.findings(),
            "semanticFindings.length",
            expected_refs.findings(),
        )?;
        require_equal_contract(
            "files.semantic.diagnostics.length + summary.semanticUnlinkedDiagnostics",
            linked_refs.diagnostics() + unlinked_refs.diagnostics(),
            "semanticDiagnostics.length",
            expected_refs.diagnostics(),
        )?;
        Ok(())
    }

    pub(crate) fn to_pretty_string(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.artifact)
    }
}

fn require_equal_contract<T>(
    left_label: &'static str,
    left: T,
    right_label: &'static str,
    right: T,
) -> Result<()>
where
    T: Eq + Debug,
{
    if left != right {
        bail!(
            "blocked-artifact-contract: {left_label} must match {right_label}: left={left:?} right={right:?}"
        );
    }
    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UnifiedArtifact<'a> {
    pub(super) schema_version: &'static str,
    pub(super) policy_version: &'static str,
    pub(super) policy: PolicyMetadata,
    pub(super) meta: ProductArtifactMeta,
    pub(super) summary: ProductSummary<'a>,
    pub(super) action_policy: ActionPolicyProjection<'a>,
    pub(super) oracle_bridge: OracleBridgeProjection<'a>,
    pub(super) files: ProductFilesProjection<'a>,
    pub(super) coverage: ProductCoverageProjection<'a>,
    pub(super) oracle_plan: ProductOraclePlanProjection<'a>,
    pub(super) semantic_findings: ProductSemanticFindingsProjection<'a>,
    pub(super) semantic_diagnostics: ProductSemanticDiagnosticsProjection<'a>,
    pub(super) artifact_refs: ArtifactRefs,
    pub(super) phases: PhaseBriefs<'a>,
}
