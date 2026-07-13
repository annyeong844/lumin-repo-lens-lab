use super::findings::{lookup_key, normalize_path, normalize_path_text, FindingRecord};
use super::protocol::{PublicDeepImportRisk, RankFixesRequest};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub(super) fn runtime_by_key(runtime_evidence: Option<&Value>) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    if let Some(records) = runtime_evidence
        .and_then(|value| value.get("merged"))
        .and_then(Value::as_array)
    {
        for record in records {
            let file = normalize_path(record.get("file").unwrap_or(&Value::Null));
            let symbol = record
                .get("symbol")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let line = record.get("line").cloned().unwrap_or(Value::Null);
            map.insert(lookup_key(&file, symbol, &line), record.clone());
        }
    }
    map
}

pub(super) fn staleness_by_key(staleness: Option<&Value>) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    if let Some(records) = staleness
        .and_then(|value| value.get("enriched"))
        .and_then(Value::as_array)
    {
        for record in records {
            let file = normalize_path(record.get("file").unwrap_or(&Value::Null));
            let symbol = record
                .get("symbol")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let line = record.get("line").cloned().unwrap_or(Value::Null);
            map.insert(lookup_key(&file, symbol, &line), record.clone());
        }
    }
    map
}

pub(super) fn resolver_summary(symbols: Option<&Value>) -> Option<Value> {
    let symbols = symbols?;
    if let Some(uses) = symbols.get("uses") {
        if let Some(ratio) = uses.get("unresolvedInternalRatio").and_then(Value::as_f64) {
            let resolved = uses
                .get("resolvedInternal")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let unresolved = uses
                .get("unresolvedInternal")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            return Some(json!({
                "unresolvedRatio": ratio,
                "unresolvedUses": unresolved,
                "totalUses": resolved + unresolved,
                "externalUses": uses.get("external").cloned().unwrap_or(Value::Null),
                "source": "uses.unresolvedInternalRatio",
            }));
        }
    }
    let total_resolved = symbols.get("totalUsesResolved")?.as_f64()?;
    let unresolved = symbols.get("unresolvedUses")?.as_f64()?;
    let total = total_resolved + unresolved;
    Some(json!({
        "unresolvedRatio": if total > 0.0 { unresolved / total } else { 0.0 },
        "unresolvedUses": unresolved,
        "totalUses": total,
        "source": "legacy (unresolvedUses/total — may include externals)",
    }))
}

pub(super) fn resolver_blindness_meta(resolver: Option<&Value>) -> Value {
    let Some(resolver) = resolver else {
        return Value::Null;
    };
    let ratio = resolver
        .get("unresolvedRatio")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    json!({
        "ratio": round4(ratio),
        "unresolvedUses": resolver.get("unresolvedUses").cloned().unwrap_or(Value::Null),
        "totalUses": resolver.get("totalUses").cloned().unwrap_or(Value::Null),
        "externalUses": resolver.get("externalUses").cloned().unwrap_or(Value::Null),
        "source": resolver.get("source").cloned().unwrap_or(Value::Null),
        "gate": if ratio >= 0.15 { "tripped" } else { "ok" },
    })
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

pub(super) fn evidence_for_finding(
    finding: &FindingRecord,
    runtime: Option<&Value>,
    staleness: Option<&Value>,
    resolver: Option<&Value>,
    request: &RankFixesRequest,
) -> Value {
    let mut evidence = Map::new();
    if let Some(runtime) = runtime {
        evidence.insert(
            "runtime".to_string(),
            json!({
                "status": runtime.get("runtimeStatus").cloned().unwrap_or(Value::Null),
                "grounding": runtime.get("grounding").cloned().unwrap_or(Value::Null),
                "confidence": runtime.get("confidence").cloned().unwrap_or(Value::Null),
                "hitsInSymbol": runtime.get("hitsInSymbol").cloned().unwrap_or(Value::Null),
            }),
        );
    }
    if let Some(staleness) = staleness {
        evidence.insert(
            "staleness".to_string(),
            json!({
                "tier": staleness.get("stalenessTier").cloned().unwrap_or(Value::Null),
                "grounding": staleness.get("grounding").cloned().unwrap_or(Value::Null),
                "lineLastTouchedDaysAgo": staleness.get("lineLastTouchedDaysAgo").cloned().unwrap_or(Value::Null),
            }),
        );
    }
    if let Some(resolver) = resolver {
        evidence.insert("resolver".to_string(), resolver.clone());
    }
    evidence.insert(
        "contract".to_string(),
        public_contract_for_file(&finding.file, request),
    );
    evidence.insert(
        "entrySurface".to_string(),
        json!({
            "htmlEntrypointBlindZone": html_entry_surface_blind_zone_for_file(
                &finding.file,
                request.artifacts.entry_surface.as_ref(),
            )
        }),
    );
    evidence.insert("policy".to_string(), json!({ "excluded": false }));
    Value::Object(evidence)
}

fn public_contract_for_file(file: &str, request: &RankFixesRequest) -> Value {
    match request.public_deep_import_risk_by_file.get(file) {
        Some(detail) => json!({
            "publicDeepImportRisk": detail.risk.unwrap_or(true),
            "publicDeepImportRiskDetail": public_risk_detail_value(detail),
        }),
        None => json!({
            "publicDeepImportRisk": true,
            "publicDeepImportRiskDetail": {
                "risk": Value::Null,
                "reason": "public-deep-import-risk-unknown",
                "file": file,
            }
        }),
    }
}

fn public_risk_detail_value(detail: &PublicDeepImportRisk) -> Value {
    let mut object = Map::new();
    object.insert(
        "risk".to_string(),
        detail.risk.map(Value::Bool).unwrap_or(Value::Null),
    );
    for (key, value) in &detail.detail {
        object.insert(key.clone(), value.clone());
    }
    Value::Object(object)
}

fn html_entry_surface_blind_zone_for_file(
    file: &str,
    entry_surface: Option<&Value>,
) -> Option<Value> {
    let unresolved = entry_surface?
        .get("unresolvedHtmlEntrypoints")
        .and_then(Value::as_array)?;
    let mut matches = unresolved
        .iter()
        .filter(|entry| html_target_could_refer_to_file(file, entry))
        .cloned()
        .collect::<Vec<_>>();
    matches.sort_by_key(html_sort_key);
    if matches.is_empty() {
        return None;
    }
    Some(json!({
        "area": "html-entry-surface",
        "reason": "html-module-script-target-missing",
        "impact": "entry-surface-unresolved",
        "relevance": "candidate-file-matches-html-target-suffix",
        "effect": "HTML module script target could refer to this file through a static server root that Lumin does not model.",
        "matches": matches.iter().take(5).map(|entry| {
            json!({
                "htmlFile": entry.get("htmlFile").cloned().unwrap_or(Value::Null),
                "src": entry.get("src").cloned().unwrap_or(Value::Null),
                "candidateFile": entry.get("resolvedFile").cloned().unwrap_or(Value::Null),
                "packageName": entry.get("packageName").cloned().unwrap_or(Value::Null),
            })
        }).collect::<Vec<_>>(),
        "total": matches.len(),
    }))
}

fn html_target_could_refer_to_file(file: &str, entry: &Value) -> bool {
    let rel_file = normalize_path_text(file);
    let candidate = entry
        .get("resolvedFile")
        .and_then(Value::as_str)
        .map(normalize_path_text)
        .unwrap_or_default();
    !rel_file.is_empty()
        && !candidate.is_empty()
        && (rel_file == candidate || rel_file.ends_with(&format!("/{candidate}")))
}

fn html_sort_key(value: &Value) -> (String, String, String) {
    (
        value
            .get("htmlFile")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        value
            .get("src")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        value
            .get("resolvedFile")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    )
}
