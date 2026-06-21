use crate::classify::Diagnostic;
use crate::metadata::CargoMetadata;
use crate::protocol::{
    BlockingTarget, CacheReuse, CacheReusePolicy, CacheReuseReason, CacheReuseSummary,
    CacheReuseSummaryStatus, ConfidenceTier, CoverageEntry, CoverageKind, CoverageStatus,
    CoverageUnavailableReason, CoverageUnavailableReasons, Disposition, Finding,
    SemanticCleanSummary, Summary,
};

pub(super) fn summary(
    findings: &[Finding],
    diagnostics: &[Diagnostic],
    coverage: &[CoverageEntry],
    cache_reuse: &CacheReuse,
) -> Summary {
    Summary {
        findings: findings.len(),
        diagnostics: diagnostics.len(),
        coverage: coverage.len(),
        verified_findings: findings
            .iter()
            .filter(|finding| finding.confidence.tier == ConfidenceTier::Verified)
            .count(),
        rule_backed_findings: findings
            .iter()
            .filter(|finding| finding.confidence.tier == ConfidenceTier::RuleBacked)
            .count(),
        candidate_findings: findings
            .iter()
            .filter(|finding| finding.confidence.tier == ConfidenceTier::Candidate)
            .count(),
        coverage_unavailable_diagnostics: diagnostics
            .iter()
            .filter(|diagnostic| {
                matches!(
                    diagnostic.classification.disposition,
                    Disposition::CoverageUnavailable
                )
            })
            .count(),
        semantic_clean: semantic_clean_summary(coverage),
        cache_reuse: cache_reuse_summary(cache_reuse),
    }
}

fn semantic_clean_summary(coverage: &[CoverageEntry]) -> SemanticCleanSummary {
    coverage
        .iter()
        .find(|entry| entry.coverage_kind == CoverageKind::AbsenceClean)
        .map(|entry| SemanticCleanSummary {
            status: entry.status,
            clean: entry.clean,
            clean_kind: entry.clean_kind,
            clean_scope: entry.clean_scope,
            reason: entry.reason.clone(),
        })
        .unwrap_or_else(|| SemanticCleanSummary {
            status: CoverageStatus::Unavailable,
            clean: None,
            clean_kind: None,
            clean_scope: None,
            reason: Some(CoverageUnavailableReasons::one(
                CoverageUnavailableReason::AbsenceCleanCoverageEntryMissing,
            )),
        })
}

fn cache_reuse_summary(cache_reuse: &CacheReuse) -> CacheReuseSummary {
    CacheReuseSummary {
        status: CacheReuseSummaryStatus::NotReusable,
        policy: cache_reuse.policy,
        reason: cache_reuse.reason,
        blocking_target_count: cache_reuse.blocking_targets.len(),
    }
}

pub(super) fn cache_reuse_metadata(metadata: Option<&CargoMetadata>) -> CacheReuse {
    let blocking_targets: Vec<BlockingTarget> = metadata
        .map(|metadata| {
            metadata
                .packages
                .iter()
                .flat_map(|pkg| {
                    pkg.targets
                        .iter()
                        .filter_map(|target| {
                            if target.blocks_cache_reuse() {
                                Some(BlockingTarget {
                                    package_id: pkg.id.clone(),
                                    package_name: pkg.name.clone(),
                                    target_name: target.name.clone(),
                                    target_kinds: target.kind.clone(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        })
        .unwrap_or_default();
    CacheReuse {
        policy: CacheReusePolicy::NoReuseUnlessCompleteInfluenceSetIsCaptured,
        reason: CacheReuseReason::AnalysisInputSetIncompleteForCacheReuse,
        blocking_targets,
    }
}
