use anyhow::{bail, Result};
use serde_json::{json, Value};

mod evidence;
mod findings;
mod policy;
mod projection;
mod protocol;
mod support;

use evidence::{
    evidence_for_finding, resolver_blindness_meta, resolver_summary, runtime_by_key,
    staleness_by_key,
};
use findings::{action_by_id, excluded_findings, merge_action_evidence, ordinary_findings};
use policy::{tier_for_finding, Tier};
use projection::{build_safe_fix_groups, by_tier, scored_entry, sort_tier_entries, summary};
pub use protocol::{
    PublicDeepImportRisk, RankFixesArtifact, RankFixesArtifacts, RankFixesRequest,
    RANK_FIXES_REQUEST_SCHEMA_VERSION,
};
use support::with_evidence_support;

const TOOL_NAME: &str = "rank-fixes.mjs";

pub fn build_rank_fixes_artifact(request: RankFixesRequest) -> Result<RankFixesArtifact> {
    if request.schema_version != RANK_FIXES_REQUEST_SCHEMA_VERSION {
        bail!(
            "rank-fixes-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if request.public_deep_import_risk_by_file.is_empty() {
        bail!("rank-fixes-artifact: missing publicDeepImportRiskByFile");
    }

    let runtime_by = runtime_by_key(request.artifacts.runtime_evidence.as_ref());
    let staleness_by = staleness_by_key(request.artifacts.staleness.as_ref());
    let action_by_id = action_by_id(request.artifacts.export_action_safety.as_ref());
    let resolver = resolver_summary(request.artifacts.symbols.as_ref());

    let mut findings = ordinary_findings(&request.artifacts.dead_classify);
    for finding in &mut findings {
        merge_action_evidence(finding, &action_by_id);
    }
    let muted_findings = excluded_findings(&request.artifacts.dead_classify);

    let mut scored = Vec::new();
    for finding in findings {
        let ranked_finding = with_evidence_support(finding.value.clone(), &request);
        let runtime = runtime_by.get(&finding.key);
        let staleness = staleness_by.get(&finding.key);
        let evidence =
            evidence_for_finding(&finding, runtime, staleness, resolver.as_ref(), &request);
        let result = tier_for_finding(&ranked_finding, &evidence);
        scored.push(scored_entry(ranked_finding, evidence, result));
    }

    for finding in muted_findings {
        let evidence = json!({
            "policy": {
                "excluded": true,
                "reason": finding.excluded_reason.as_deref().unwrap_or("unknown")
            }
        });
        let result = tier_for_finding(&finding.value, &evidence);
        scored.push(scored_entry(finding.value, evidence, result));
    }

    let mut by_tier = by_tier(scored);
    sort_tier_entries(&mut by_tier);
    let safe_fix_groups = build_safe_fix_groups(by_tier.get(&Tier::SafeFix));
    let summary = summary(&by_tier, safe_fix_groups.len());
    let inputs = input_summary(&request);

    Ok(RankFixesArtifact {
        meta: json!({
            "generated": request.generated,
            "root": request.root,
            "tool": TOOL_NAME,
            "executionOwner": "lumin-audit-core",
            "inputs": inputs,
            "resolverBlindness": resolver_blindness_meta(resolver.as_ref()),
            "topUnresolvedSpecifiers": request.artifacts.symbols
                .as_ref()
                .and_then(|symbols| symbols.get("topUnresolvedSpecifiers"))
                .cloned()
                .unwrap_or_else(|| json!([])),
        }),
        summary,
        safe_fixes: by_tier.remove(&Tier::SafeFix).unwrap_or_default(),
        safe_fix_groups,
        review_fixes: by_tier.remove(&Tier::ReviewFix).unwrap_or_default(),
        degraded: by_tier.remove(&Tier::Degraded).unwrap_or_default(),
        muted: by_tier.remove(&Tier::Muted).unwrap_or_default(),
    })
}

fn input_summary(request: &RankFixesRequest) -> Value {
    json!({
        "dead-classify.json": true,
        "runtime-evidence.json": request.artifacts.runtime_evidence.is_some(),
        "staleness.json": request.artifacts.staleness.is_some(),
        "symbols.json": request.artifacts.symbols.is_some(),
        "export-action-safety.json": request.artifacts.export_action_safety.is_some(),
        "call-graph.json": request.artifacts.call_graph.is_some(),
        "entry-surface.json": request.artifacts.entry_surface.is_some(),
        "module-reachability.json": request.artifacts.module_reachability.is_some()
    })
}
