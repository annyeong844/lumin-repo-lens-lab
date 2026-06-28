use anyhow::{bail, Result};
use serde::Serialize;

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
        if let Some(error) = self.artifact.files.first_semantic_ref_contract_error(
            expected_refs,
            self.artifact.summary.semantic_unlinked_refs(),
        ) {
            bail!("blocked-artifact-contract: {error}");
        }
        Ok(())
    }

    pub(crate) fn to_pretty_string(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.artifact)
    }
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
