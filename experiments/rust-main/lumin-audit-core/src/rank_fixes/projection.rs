use super::policy::{Tier, TierResult};
use serde_json::{json, Map, Value};
use std::cmp::Reverse;
use std::collections::BTreeMap;

pub(super) fn scored_entry(finding: Value, evidence: Value, result: TierResult) -> Value {
    let mut object = Map::new();
    object.insert("finding".to_string(), finding);
    object.insert("evidence".to_string(), evidence);
    object.insert(
        "tier".to_string(),
        Value::String(result.tier.as_str().to_string()),
    );
    object.insert("reason".to_string(), Value::String(result.reason));
    if let Some(confidence) = result.confidence {
        object.insert(
            "confidence".to_string(),
            Value::String(confidence.to_string()),
        );
    }
    if let Some(confidence_detail) = result.confidence_detail {
        object.insert(
            "confidenceDetail".to_string(),
            Value::String(confidence_detail.to_string()),
        );
    }
    if result.blocked_promotion {
        object.insert("blockedPromotion".to_string(), Value::Bool(true));
        object.insert("blockedBy".to_string(), Value::Array(result.blocked_by));
    }
    Value::Object(object)
}

pub(super) fn by_tier(scored: Vec<Value>) -> BTreeMap<Tier, Vec<Value>> {
    let mut by_tier: BTreeMap<Tier, Vec<Value>> = BTreeMap::from([
        (Tier::SafeFix, Vec::new()),
        (Tier::ReviewFix, Vec::new()),
        (Tier::Degraded, Vec::new()),
        (Tier::Muted, Vec::new()),
    ]);
    for entry in scored {
        let tier = match entry
            .get("tier")
            .and_then(Value::as_str)
            .unwrap_or_default()
        {
            "SAFE_FIX" => Tier::SafeFix,
            "REVIEW_FIX" => Tier::ReviewFix,
            "DEGRADED" => Tier::Degraded,
            "MUTED" => Tier::Muted,
            _ => Tier::Degraded,
        };
        by_tier.entry(tier).or_default().push(entry);
    }
    by_tier
}

pub(super) fn sort_tier_entries(by_tier: &mut BTreeMap<Tier, Vec<Value>>) {
    for entries in by_tier.values_mut() {
        entries.sort_by_key(sort_key);
    }
}

fn sort_key(score: &Value) -> (String, i64, String) {
    let finding = score.get("finding").unwrap_or(&Value::Null);
    (
        finding
            .get("file")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        finding.get("line").and_then(Value::as_i64).unwrap_or(0),
        finding
            .get("symbol")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    )
}

pub(super) fn summary(by_tier: &BTreeMap<Tier, Vec<Value>>, safe_fix_groups: usize) -> Value {
    let safe = by_tier.get(&Tier::SafeFix).map_or(0, Vec::len);
    let review = by_tier.get(&Tier::ReviewFix).map_or(0, Vec::len);
    let degraded = by_tier.get(&Tier::Degraded).map_or(0, Vec::len);
    let muted = by_tier.get(&Tier::Muted).map_or(0, Vec::len);
    let mut object = Map::new();
    object.insert("SAFE_FIX".to_string(), Value::from(safe));
    object.insert("REVIEW_FIX".to_string(), Value::from(review));
    object.insert("DEGRADED".to_string(), Value::from(degraded));
    object.insert("MUTED".to_string(), Value::from(muted));
    object.insert(
        "total".to_string(),
        Value::from(safe + review + degraded + muted),
    );
    if let Some(review_reasons) = public_deep_import_review_reasons(by_tier.get(&Tier::ReviewFix)) {
        object.insert("reviewReasons".to_string(), review_reasons);
    }
    object.insert("safeFixGroups".to_string(), Value::from(safe_fix_groups));
    Value::Object(object)
}

fn public_deep_import_review_reasons(review_fixes: Option<&Vec<Value>>) -> Option<Value> {
    let mut reasons = BTreeMap::new();
    for entry in review_fixes? {
        let Some(contract) = entry
            .get("evidence")
            .and_then(|evidence| evidence.get("contract"))
        else {
            continue;
        };
        if contract
            .get("publicDeepImportRisk")
            .and_then(Value::as_bool)
            != Some(true)
        {
            continue;
        }
        let reason = contract
            .get("publicDeepImportRiskDetail")
            .and_then(|detail| detail.get("reason"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        *reasons.entry(reason).or_insert(0usize) += 1;
    }
    if reasons.is_empty() {
        None
    } else {
        Some(json!({ "publicDeepImportRisk": reasons }))
    }
}

#[derive(Default)]
struct SafeFixGroupAccumulator {
    count: usize,
    symbols: Vec<String>,
    lines: Vec<Value>,
}

pub(super) fn build_safe_fix_groups(safe_fixes: Option<&Vec<Value>>) -> Vec<Value> {
    let mut groups: BTreeMap<(String, String), SafeFixGroupAccumulator> = BTreeMap::new();
    for score in safe_fixes.into_iter().flatten() {
        let finding = score.get("finding").unwrap_or(&Value::Null);
        let file = finding
            .get("file")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let action_kind = finding
            .get("safeAction")
            .and_then(|safe_action| safe_action.get("kind"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let entry = groups.entry((file, action_kind)).or_default();
        entry.count += 1;
        entry.symbols.push(
            finding
                .get("symbol")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        );
        entry
            .lines
            .push(finding.get("line").cloned().unwrap_or(Value::Null));
    }
    let mut projected = groups
        .into_iter()
        .map(|((file, action_kind), group)| {
            json!({
                "file": file,
                "actionKind": action_kind,
                "count": group.count,
                "symbols": group.symbols,
                "lines": group.lines,
            })
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|group| {
        (
            Reverse(group.get("count").and_then(Value::as_u64).unwrap_or(0)),
            group
                .get("file")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            group
                .get("actionKind")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        )
    });
    projected
}
