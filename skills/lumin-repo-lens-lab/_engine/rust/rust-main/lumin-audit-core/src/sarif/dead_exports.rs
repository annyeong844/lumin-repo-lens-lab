use std::collections::BTreeMap;

use serde_json::{Map, Value};

use super::support::{
    array_field, copy_field, insert_optional_string, insert_string, insert_value, make_result,
    number_field, string_field, SarifState,
};

pub(super) fn collect_dead_export_results(
    state: &mut SarifState,
    root: &str,
    fix_plan: Option<&Value>,
    runtime_evidence: Option<&Value>,
    staleness: Option<&Value>,
    dead_classify: Option<&Value>,
    symbols: Option<&Value>,
) {
    if let Some(fix_plan) = fix_plan {
        state.artifacts_used.push("fix-plan.json");
        emit_fix_plan_entries(state, root, fix_plan, "safeFixes", "SAFE_FIX");
        emit_fix_plan_entries(state, root, fix_plan, "reviewFixes", "REVIEW_FIX");
        emit_fix_plan_entries(state, root, fix_plan, "degraded", "DEGRADED");
        return;
    }

    if let Some(runtime_evidence) = runtime_evidence {
        if !array_field(runtime_evidence, "merged").is_empty() {
            state.artifacts_used.push("runtime-evidence.json");
            let staleness_by = staleness.and_then(staleness_lookup);
            if staleness_by.is_some() {
                state.artifacts_used.push("staleness.json");
            }
            for merged in array_field(runtime_evidence, "merged") {
                if string_field(merged, "grounding").as_deref() == Some("blind") {
                    continue;
                }
                let key = finding_key(merged);
                let stale = key.as_ref().and_then(|key| {
                    staleness_by
                        .as_ref()
                        .and_then(|lookup| lookup.get(key).copied())
                });
                let symbol = string_field(merged, "symbol").unwrap_or_default();
                let kind = string_field(merged, "kind").unwrap_or_default();
                let runtime_status =
                    string_field(merged, "runtimeStatus").unwrap_or_else(|| "not-measured".into());
                let grounding =
                    string_field(merged, "grounding").unwrap_or_else(|| "grounded".into());
                let confidence =
                    string_field(merged, "confidence").unwrap_or_else(|| "medium".into());
                let mut parts = vec![
                    format!("Dead export `{symbol}` ({kind})"),
                    format!("runtime: {runtime_status}"),
                ];
                if let Some(stale) = stale {
                    if let Some(tier) = string_field(stale, "stalenessTier") {
                        parts.push(format!("staleness: {tier}"));
                    }
                }
                parts.push(format!("grounding: {grounding}/{confidence}"));

                let mut properties = Map::new();
                insert_string(&mut properties, "symbol", symbol);
                insert_string(&mut properties, "kind", kind);
                insert_string(&mut properties, "grounding", grounding.clone());
                insert_string(&mut properties, "confidence", confidence.clone());
                insert_string(&mut properties, "runtimeStatus", runtime_status.clone());
                insert_value(
                    &mut properties,
                    "hitsInSymbol",
                    merged
                        .get("hitsInSymbol")
                        .cloned()
                        .unwrap_or_else(|| Value::from(0)),
                );
                if let Some(note) = merged.get("note").cloned() {
                    insert_value(&mut properties, "note", note);
                }
                if let Some(stale) = stale {
                    copy_field(stale, &mut properties, "stalenessTier");
                    copy_field(stale, &mut properties, "lineLastTouchedDaysAgo");
                    copy_field(stale, &mut properties, "symbolMentionStatus");
                }
                state.results.push(make_result(
                    "GA001",
                    level_for_dead(&grounding, &confidence, &runtime_status),
                    parts.join(" | "),
                    &string_field(merged, "file").unwrap_or_default(),
                    number_field(merged, "line"),
                    properties,
                    root,
                ));
            }
            return;
        }
    }

    if let Some(dead_classify) = dead_classify {
        state.artifacts_used.push("dead-classify.json");
        emit_dead_classify_entries(
            state,
            root,
            dead_classify,
            "proposal_C_remove_symbol",
            "warning",
            "C",
        );
        emit_dead_classify_entries(
            state,
            root,
            dead_classify,
            "proposal_A_demote_to_internal",
            "warning",
            "A",
        );
        emit_dead_classify_entries(state, root, dead_classify, "proposal_B_review", "note", "B");
        emit_dead_classify_entries(
            state,
            root,
            dead_classify,
            "proposal_remove_export_specifier",
            "note",
            "specifier",
        );
        return;
    }

    if let Some(symbols) = symbols {
        let dead = array_field(symbols, "deadProdList");
        if dead.is_empty() {
            return;
        }
        state.artifacts_used.push("symbols.json");
        for finding in dead {
            let symbol = string_field(finding, "symbol").unwrap_or_default();
            let kind = string_field(finding, "kind").unwrap_or_default();
            let mut properties = Map::new();
            insert_string(&mut properties, "symbol", symbol.clone());
            insert_string(&mut properties, "kind", kind.clone());
            insert_string(&mut properties, "grounding", "grounded");
            insert_string(&mut properties, "confidence", "low");
            insert_string(&mut properties, "runtimeStatus", "not-measured");
            state.results.push(make_result(
                "GA001",
                "note",
                format!(
                    "Dead export `{symbol}` ({kind}) — pre-policy static AST; run classify-dead-exports.mjs for policy-filtered verdict."
                ),
                &string_field(finding, "file").unwrap_or_default(),
                number_field(finding, "line"),
                properties,
                root,
            ));
        }
    }
}

fn emit_fix_plan_entries(
    state: &mut SarifState,
    root: &str,
    fix_plan: &Value,
    field: &str,
    tier: &str,
) {
    let Some(level) = tier_to_sarif_level(tier) else {
        return;
    };
    for scored in array_field(fix_plan, field) {
        let finding = scored.get("finding").unwrap_or(&Value::Null);
        let evidence = scored.get("evidence").unwrap_or(&Value::Null);
        let runtime = evidence.get("runtime").unwrap_or(&Value::Null);
        let stale = evidence.get("staleness").unwrap_or(&Value::Null);
        let symbol = string_field(finding, "symbol").unwrap_or_default();
        let kind = string_field(finding, "kind").unwrap_or_default();
        let reason = string_field(scored, "reason").unwrap_or_default();
        let mut parts = vec![
            format!("Dead export `{symbol}` ({kind})"),
            format!("tier: {tier}"),
        ];
        if let Some(status) = string_field(runtime, "status") {
            parts.push(format!("runtime: {status}"));
        }
        if let Some(tier) = string_field(stale, "tier") {
            parts.push(format!("staleness: {tier}"));
        }
        parts.push(format!("({reason})"));

        let mut properties = Map::new();
        insert_string(&mut properties, "symbol", symbol);
        insert_string(&mut properties, "kind", kind);
        insert_string(&mut properties, "tier", tier);
        insert_string(&mut properties, "reason", reason);
        insert_optional_string(
            &mut properties,
            "proposalBucket",
            string_field(finding, "bucket"),
        );
        insert_string(
            &mut properties,
            "grounding",
            string_field(runtime, "grounding").unwrap_or_else(|| "grounded".to_string()),
        );
        insert_string(
            &mut properties,
            "confidence",
            string_field(runtime, "confidence").unwrap_or_else(|| "medium".to_string()),
        );
        insert_string(
            &mut properties,
            "runtimeStatus",
            string_field(runtime, "status").unwrap_or_else(|| "not-measured".to_string()),
        );
        insert_value(
            &mut properties,
            "hitsInSymbol",
            runtime
                .get("hitsInSymbol")
                .cloned()
                .unwrap_or_else(|| Value::from(0)),
        );
        if let Some(staleness_tier) = string_field(stale, "tier") {
            insert_string(&mut properties, "stalenessTier", staleness_tier);
        }
        copy_field(stale, &mut properties, "lineLastTouchedDaysAgo");
        copy_field(finding, &mut properties, "fileInternalUses");
        copy_field(finding, &mut properties, "predicatePartner");
        copy_field(finding, &mut properties, "localName");

        state.results.push(make_result(
            "GA001",
            level,
            parts.join(" | "),
            &string_field(finding, "file").unwrap_or_default(),
            number_field(finding, "line"),
            properties,
            root,
        ));
    }
}

fn emit_dead_classify_entries(
    state: &mut SarifState,
    root: &str,
    dead_classify: &Value,
    field: &str,
    level: &str,
    action_field: &str,
) {
    for proposal in array_field(dead_classify, field) {
        let symbol = string_field(proposal, "symbol").unwrap_or_default();
        let kind = string_field(proposal, "kind").unwrap_or_default();
        let action = string_field(proposal, "action").unwrap_or_default();
        let mut properties = Map::new();
        insert_string(&mut properties, "symbol", symbol.clone());
        insert_string(&mut properties, "kind", kind.clone());
        insert_string(&mut properties, "grounding", "grounded");
        insert_string(&mut properties, "confidence", "medium");
        insert_string(&mut properties, "runtimeStatus", "not-measured");
        insert_string(&mut properties, "proposalBucket", action_field);
        copy_field(proposal, &mut properties, "localName");
        copy_field(proposal, &mut properties, "fileInternalUses");
        copy_field(proposal, &mut properties, "predicatePartner");
        state.results.push(make_result(
            "GA001",
            level,
            format!("Dead export `{symbol}` ({kind}) — {action}"),
            &string_field(proposal, "file").unwrap_or_default(),
            number_field(proposal, "line"),
            properties,
            root,
        ));
    }
}

fn staleness_lookup(staleness: &Value) -> Option<BTreeMap<String, &Value>> {
    let enriched = staleness.get("enriched")?.as_array()?;
    let mut lookup = BTreeMap::new();
    for entry in enriched {
        if let Some(key) = finding_key(entry) {
            lookup.insert(key, entry);
        }
    }
    Some(lookup)
}

fn finding_key(value: &Value) -> Option<String> {
    Some(format!(
        "{}|{}|{}",
        string_field(value, "file")?,
        string_field(value, "symbol")?,
        number_field(value, "line")?
    ))
}

fn level_for_dead(grounding: &str, confidence: &str, runtime_status: &str) -> &'static str {
    if runtime_status == "executed" {
        return "note";
    }
    if grounding == "grounded" && confidence == "high" {
        return "warning";
    }
    if grounding == "grounded" {
        return "warning";
    }
    "note"
}

fn tier_to_sarif_level(tier: &str) -> Option<&'static str> {
    match tier {
        "SAFE_FIX" => Some("warning"),
        "REVIEW_FIX" | "DEGRADED" => Some("note"),
        "MUTED" => None,
        _ => Some("note"),
    }
}
