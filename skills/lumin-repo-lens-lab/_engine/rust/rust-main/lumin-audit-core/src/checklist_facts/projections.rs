use super::{
    as_usize, generated_only_count, non_generated_array, parse_percent, round3, unavailable,
    value_at, SilentCatchFacts,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub(super) fn dead_code(fix_plan: Option<&Value>) -> Value {
    let Some(summary) = fix_plan.and_then(|artifact| artifact.get("summary")) else {
        return unavailable(
            "fix-plan.json missing — run rank-fixes.mjs after classify-dead-exports.mjs",
        );
    };
    let safe_fix = summary.get("SAFE_FIX").and_then(as_usize).unwrap_or(0);
    json!({
        "gate": if safe_fix >= 10 { "fix" } else if safe_fix > 0 { "watch" } else { "ok" },
        "safeFix": safe_fix,
        "reviewFix": summary.get("REVIEW_FIX").and_then(as_usize).unwrap_or(0),
        "degraded": summary.get("DEGRADED").and_then(as_usize).unwrap_or(0),
        "muted": summary.get("MUTED").and_then(as_usize).unwrap_or(0),
        "total": summary.get("total").and_then(as_usize).unwrap_or(0),
    })
}

pub(super) fn duplicate_implementation(function_clones: Option<&Value>) -> Value {
    let Some(function_clones) = function_clones else {
        return unavailable("function-clones.json missing — run full profile or build-function-clone-index.mjs first");
    };
    let exact_groups = non_generated_array(function_clones, "exactBodyGroups");
    let structure_groups = non_generated_array(function_clones, "structureGroups");
    let signature_groups = non_generated_array(function_clones, "signatureGroups");
    let near_function_candidates = non_generated_array(function_clones, "nearFunctionCandidates");
    let mut candidate_identities = BTreeSet::<String>::new();
    for group in structure_groups
        .iter()
        .chain(signature_groups.iter())
        .chain(near_function_candidates.iter())
    {
        if let Some(identities) = group.get("identities").and_then(Value::as_array) {
            for identity in identities.iter().filter_map(Value::as_str) {
                candidate_identities.insert(identity.to_string());
            }
        }
    }
    let has_candidates = !exact_groups.is_empty()
        || !structure_groups.is_empty()
        || !signature_groups.is_empty()
        || !near_function_candidates.is_empty();

    json!({
        "gate": if has_candidates { "watch" } else { "ok" },
        "available": true,
        "exactBodyGroups": exact_groups.len(),
        "structureGroupCandidates": structure_groups.len(),
        "signatureGroupCandidates": signature_groups.len(),
        "nearFunctionCandidates": near_function_candidates.len(),
        "generatedOnlyExactGroups": generated_only_count(function_clones, "exactBodyGroups"),
        "generatedOnlyStructureGroups": generated_only_count(function_clones, "structureGroups"),
        "generatedOnlySignatureGroups": generated_only_count(function_clones, "signatureGroups"),
        "generatedOnlyNearFunctionCandidates": generated_only_count(function_clones, "nearFunctionCandidates"),
        "candidateIdentityCount": candidate_identities.len(),
        "totalFunctionFacts": function_clones.get("facts").and_then(Value::as_array).map_or(0, Vec::len),
        "functionCloneIndexComplete": value_at(function_clones, &["meta", "complete"]).and_then(Value::as_bool).unwrap_or(true),
        "topExactGroups": exact_groups.into_iter().take(10).collect::<Vec<_>>(),
        "topStructureGroups": structure_groups.into_iter().take(10).collect::<Vec<_>>(),
        "topSignatureGroups": signature_groups.into_iter().take(10).collect::<Vec<_>>(),
        "topNearFunctionCandidates": near_function_candidates.into_iter().take(10).collect::<Vec<_>>(),
        "note": "Exact body, same-structure, same-signature, and near exported function cues only. Treat as review cues, not proof of semantic equivalence or an automatic merge.",
    })
}

pub(super) fn lint_enforcement(triage: Option<&Value>) -> Value {
    let Some(rules) = triage
        .and_then(|artifact| artifact.get("boundaries"))
        .and_then(Value::as_array)
    else {
        return unavailable("triage.json missing — run triage-repo.mjs first");
    };
    let has_boundary_rule = rules.iter().any(|rule| {
        matches!(
            rule.get("rule").and_then(Value::as_str),
            Some("no-restricted-imports")
                | Some("no-restricted-paths")
                | Some("eslint-plugin-boundaries")
        )
    });
    json!({
        "gate": if has_boundary_rule { "ok" } else { "watch" },
        "rulesDetected": rules.len(),
        "boundaryRulePresent": has_boundary_rule,
        "rules": rules,
    })
}

pub(super) fn barrel_amplification(barrels: Option<&Value>) -> Value {
    let Some(barrels) = barrels else {
        return unavailable("barrels.json missing — run check-barrel-discipline.mjs first");
    };
    if barrels.get("mode").and_then(Value::as_str) == Some("single-package") {
        return json!({
            "gate": "ok",
            "reason": "single-package repo — no workspace barrels to discipline",
        });
    }
    let mut by_package = Vec::<Value>::new();
    let mut worst_compliance = 1.0f64;
    if let Some(packages) = barrels.get("byPackage").and_then(Value::as_object) {
        for (pkg, data) in packages {
            let compliance = data
                .get("policyCompliance")
                .and_then(Value::as_str)
                .unwrap_or("");
            let compliance_num = parse_percent(compliance);
            if let Some(pct) = compliance_num {
                worst_compliance = worst_compliance.min(pct);
            }
            by_package.push(json!({
                "pkg": pkg,
                "rootImports": data.get("rootImports").cloned().unwrap_or(Value::Null),
                "subpathImports": data.get("subpathImports").cloned().unwrap_or(Value::Null),
                "total": data.get("total").cloned().unwrap_or(Value::Null),
                "compliance": data.get("policyCompliance").cloned().unwrap_or(Value::Null),
                "complianceNum": compliance_num.map_or(Value::Null, |value| json!(value)),
            }));
        }
    }
    let gate = if worst_compliance < 0.5 {
        "fix"
    } else if worst_compliance < 0.8 {
        "watch"
    } else {
        "ok"
    };
    json!({
        "gate": gate,
        "worstCompliance": round3(worst_compliance),
        "byPackage": by_package,
    })
}

pub(super) fn silent_catch(facts: &SilentCatchFacts) -> Value {
    let watch_count = facts.non_empty_anonymous_sites.len() + facts.unused_param_sites.len();
    let gate = if facts.sites.len() > 3 {
        "fix"
    } else if !facts.sites.is_empty() || watch_count > 0 {
        "watch"
    } else {
        "ok"
    };
    json!({
        "gate": gate,
        "analysis": facts.analysis,
        "count": facts.sites.len(),
        "emptyUndocumentedCount": facts.sites.len(),
        "parseErrors": facts.parse_errors,
        "sites": facts.sites,
        "documentedCount": facts.documented_sites.len(),
        "emptyDocumentedCount": facts.documented_sites.len(),
        "documentedSites": facts.documented_sites,
        "anonymousCount": facts.anonymous_sites.len(),
        "anonymousSites": facts.anonymous_sites,
        "nonEmptyAnonymousCount": facts.non_empty_anonymous_sites.len(),
        "nonEmptyAnonymousSites": facts.non_empty_anonymous_sites,
        "unusedParamCount": facts.unused_param_sites.len(),
        "unusedParamSites": facts.unused_param_sites,
    })
}
