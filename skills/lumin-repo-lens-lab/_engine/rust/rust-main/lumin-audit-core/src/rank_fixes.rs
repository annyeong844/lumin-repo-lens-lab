use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

mod policy;

use policy::{tier_for_finding, Tier, TierResult};

pub const RANK_FIXES_REQUEST_SCHEMA_VERSION: &str = "lumin-rank-fixes-producer-request.v1";

const TOOL_NAME: &str = "rank-fixes.mjs";
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankFixesRequest {
    pub schema_version: String,
    pub root: String,
    pub generated: String,
    pub artifacts: RankFixesArtifacts,
    #[serde(default)]
    pub public_deep_import_risk_by_file: BTreeMap<String, PublicDeepImportRisk>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankFixesArtifacts {
    pub dead_classify: Value,
    #[serde(default)]
    pub runtime_evidence: Option<Value>,
    #[serde(default)]
    pub staleness: Option<Value>,
    #[serde(default)]
    pub symbols: Option<Value>,
    #[serde(default)]
    pub export_action_safety: Option<Value>,
    #[serde(default)]
    pub call_graph: Option<Value>,
    #[serde(default)]
    pub entry_surface: Option<Value>,
    #[serde(default)]
    pub module_reachability: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicDeepImportRisk {
    #[serde(default)]
    pub risk: Option<bool>,
    #[serde(flatten)]
    pub detail: BTreeMap<String, Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankFixesArtifact {
    pub meta: Value,
    pub summary: Value,
    pub safe_fixes: Vec<Value>,
    pub safe_fix_groups: Vec<Value>,
    pub review_fixes: Vec<Value>,
    pub degraded: Vec<Value>,
    pub muted: Vec<Value>,
}

#[derive(Debug, Clone)]
struct FindingRecord {
    value: Value,
    id: String,
    key: String,
    file: String,
    excluded_reason: Option<String>,
}

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

fn ordinary_findings(dead_classify: &Value) -> Vec<FindingRecord> {
    let mut records = Vec::new();
    records.extend(flatten_bucket(
        dead_classify,
        "proposal_C_remove_symbol",
        "C",
    ));
    records.extend(flatten_bucket(
        dead_classify,
        "proposal_A_demote_to_internal",
        "A",
    ));
    records.extend(flatten_bucket(dead_classify, "proposal_B_review", "B"));
    records.extend(flatten_bucket(
        dead_classify,
        "proposal_remove_export_specifier",
        "specifier",
    ));
    records.extend(flatten_bucket(
        dead_classify,
        "proposal_DEGRADED_unprocessed",
        "unprocessed",
    ));
    records
}

fn flatten_bucket(dead_classify: &Value, field: &str, bucket: &str) -> Vec<FindingRecord> {
    dead_classify
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| finding_record(item, bucket, None))
        .collect()
}

fn excluded_findings(dead_classify: &Value) -> Vec<FindingRecord> {
    dead_classify
        .get("excludedCandidates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let reason = item
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            let mut record = finding_record(item, "excluded", Some(reason.clone()))?;
            let object = record.value.as_object_mut()?;
            object.insert(
                "action".to_string(),
                Value::String(format!("Policy-excluded: {reason}")),
            );
            object.insert("_excludeReason".to_string(), Value::String(reason));
            if let Some(policy_evidence) = item.get("policyEvidence") {
                object.insert("policyEvidence".to_string(), policy_evidence.clone());
            }
            Some(record)
        })
        .collect()
}

fn finding_record(
    item: &Value,
    bucket: &str,
    excluded_reason: Option<String>,
) -> Option<FindingRecord> {
    let file = normalize_path(item.get("file")?);
    let symbol = item.get("symbol")?.as_str()?.to_string();
    let line = item.get("line").cloned().unwrap_or(Value::Null);
    let id = finding_id(&file, &symbol, &line);
    let key = lookup_key(&file, &symbol, &line);
    let mut object = item.as_object()?.clone();
    object.insert("id".to_string(), Value::String(id.clone()));
    object.insert("file".to_string(), Value::String(file.clone()));
    object.insert("bucket".to_string(), Value::String(bucket.to_string()));
    Some(FindingRecord {
        value: Value::Object(object),
        id,
        key,
        file,
        excluded_reason,
    })
}

fn merge_action_evidence(finding: &mut FindingRecord, action_by_id: &BTreeMap<String, Value>) {
    let Some(action_record) = action_by_id.get(&finding.id) else {
        return;
    };
    let Some(object) = finding.value.as_object_mut() else {
        return;
    };
    for field in ["safeAction", "actionBlockers", "localUseProof"] {
        if let Some(value) = action_record.get(field) {
            object.insert(field.to_string(), value.clone());
        }
    }
}

fn normalize_path(value: &Value) -> String {
    value
        .as_str()
        .unwrap_or_default()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

fn normalize_path_text(value: &str) -> String {
    value
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

fn line_key(value: &Value) -> String {
    match value {
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn finding_id(file: &str, symbol: &str, line: &Value) -> String {
    format!("dead-export:{file}:{symbol}:{}", line_key(line))
}

fn lookup_key(file: &str, symbol: &str, line: &Value) -> String {
    format!("{file}|{symbol}|{}", line_key(line))
}

fn finding_identity(file: &str, symbol: &str) -> String {
    format!("{file}::{symbol}")
}

fn action_by_id(export_action_safety: Option<&Value>) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    if let Some(by_id) = export_action_safety
        .and_then(|value| value.get("byId"))
        .and_then(Value::as_object)
    {
        for (id, record) in by_id {
            map.insert(id.clone(), record.clone());
        }
    }
    if let Some(records) = export_action_safety
        .and_then(|value| value.get("findings"))
        .and_then(Value::as_array)
    {
        for record in records {
            if let Some(id) = record.get("id").and_then(Value::as_str) {
                map.insert(id.to_string(), record.clone());
            }
        }
    }
    map
}

fn runtime_by_key(runtime_evidence: Option<&Value>) -> BTreeMap<String, Value> {
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

fn staleness_by_key(staleness: Option<&Value>) -> BTreeMap<String, Value> {
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

fn resolver_summary(symbols: Option<&Value>) -> Option<Value> {
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

fn resolver_blindness_meta(resolver: Option<&Value>) -> Value {
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

fn evidence_for_finding(
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

fn with_evidence_support(mut finding: Value, request: &RankFixesRequest) -> Value {
    if let Some(support) = entry_unreachable_support(&finding, request) {
        add_support(&mut finding, support);
    }
    if let Some(support) = call_graph_no_observed_callers_support(&finding, request) {
        add_support(&mut finding, support);
    }
    finding
}

fn add_support(finding: &mut Value, support: Value) {
    let kind = support.get("kind").and_then(Value::as_str);
    let Some(object) = finding.as_object_mut() else {
        return;
    };
    let entry = object
        .entry("supportedBy".to_string())
        .or_insert_with(|| Value::Array(Vec::new()));
    let Some(items) = entry.as_array_mut() else {
        return;
    };
    if kind.is_some()
        && items
            .iter()
            .any(|item| item.get("kind").and_then(Value::as_str) == kind)
    {
        return;
    }
    items.push(support);
}

fn entry_unreachable_support(finding: &Value, request: &RankFixesRequest) -> Option<Value> {
    let reachability = request.artifacts.module_reachability.as_ref()?;
    let entry_surface = request.artifacts.entry_surface.as_ref()?;
    let file = finding.get("file").and_then(Value::as_str)?;
    if !string_array_contains(reachability.get("unreachableFiles"), file) {
        return None;
    }
    if string_array_contains(reachability.get("runtimeReachableFiles"), file)
        || string_array_contains(reachability.get("typeReachableFiles"), file)
        || string_array_contains(reachability.get("boundedOutFiles"), file)
        || entry_files(entry_surface).contains(file)
        || opaque_dynamic_import_could_reach(file, request.artifacts.symbols.as_ref())
    {
        return None;
    }
    if completeness_for_file(file, reachability, entry_surface) != Some("high".to_string()) {
        return None;
    }
    match request.public_deep_import_risk_by_file.get(file) {
        Some(detail) if detail.risk == Some(false) => {}
        _ => return None,
    }
    Some(json!({
        "kind": "entry-unreachable",
        "artifact": "module-reachability.json",
        "completeness": "high",
    }))
}

fn entry_files(entry_surface: &Value) -> BTreeSet<String> {
    let mut files = BTreeSet::new();
    for field in [
        "entryFiles",
        "publicApiFiles",
        "frameworkEntrypointFiles",
        "configEntrypointFiles",
        "scriptEntrypointFiles",
        "htmlEntrypointFiles",
    ] {
        for file in string_array(entry_surface.get(field)) {
            files.insert(file);
        }
    }
    files
}

fn completeness_for_file(
    file: &str,
    reachability: &Value,
    entry_surface: &Value,
) -> Option<String> {
    let by_submodule = reachability
        .get("meta")
        .and_then(|meta| meta.get("completenessBySubmodule"))
        .or_else(|| entry_surface.get("completenessBySubmodule"))
        .and_then(Value::as_object)?;
    let mut best: Option<(&str, &Value)> = None;
    for (submodule, value) in by_submodule {
        let root = submodule.as_str();
        let matches = root == "."
            || file == root
            || file
                .strip_prefix(root)
                .is_some_and(|suffix| suffix.starts_with('/'));
        if !matches {
            continue;
        }
        if best
            .as_ref()
            .is_none_or(|(best_root, _)| root.len() > best_root.len())
        {
            best = Some((root, value));
        }
    }
    best.and_then(|(_, value)| value.as_str().map(ToString::to_string))
}

fn opaque_dynamic_import_could_reach(file: &str, symbols: Option<&Value>) -> bool {
    symbols
        .and_then(|symbols| symbols.get("dynamicImportOpacity"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|item| {
            item.get("targetDir")
                .and_then(Value::as_str)
                .map(normalize_path_text)
                .is_some_and(|target_dir| file.starts_with(&target_dir))
        })
}

fn call_graph_no_observed_callers_support(
    finding: &Value,
    request: &RankFixesRequest,
) -> Option<Value> {
    let call_graph = request.artifacts.call_graph.as_ref()?;
    if !has_bounded_member_call_stats(call_graph) || !is_function_like_finding(finding) {
        return None;
    }
    if is_framework_callback_like(finding)
        || !symbol_graph_fan_in_zero(finding, request.artifacts.symbols.as_ref())
    {
        return None;
    }
    if !call_graph_fan_in_zero(finding, call_graph) {
        return None;
    }
    let ratio = nearby_bounded_out_ratio(
        finding
            .get("file")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        call_graph,
    )?;
    if ratio >= 0.10 {
        return None;
    }
    Some(json!({
        "kind": "call-graph-no-observed-callers",
        "artifact": "call-graph.json",
    }))
}

fn has_bounded_member_call_stats(call_graph: &Value) -> bool {
    call_graph
        .get("meta")
        .and_then(|meta| meta.get("supports"))
        .and_then(|supports| supports.get("boundedMemberCallResolution"))
        .and_then(Value::as_bool)
        == Some(true)
        && call_graph.get("boundedOutMemberCallsByFile").is_some()
        && call_graph.get("memberCallsByFile").is_some()
}

fn is_function_like_finding(finding: &Value) -> bool {
    const FUNCTION_LIKE_KINDS: &[&str] = &[
        "FunctionDeclaration",
        "FunctionExpression",
        "ArrowFunctionExpression",
        "MethodDefinition",
        "TSDeclareFunction",
    ];
    let kind = finding.get("kind").and_then(Value::as_str);
    let node_kind = finding
        .get("safeAction")
        .and_then(|safe_action| safe_action.get("target"))
        .and_then(|target| target.get("nodeKind"))
        .and_then(Value::as_str);
    kind.is_some_and(|kind| FUNCTION_LIKE_KINDS.contains(&kind))
        || node_kind.is_some_and(|kind| FUNCTION_LIKE_KINDS.contains(&kind))
}

fn is_framework_callback_like(finding: &Value) -> bool {
    let file = finding
        .get("file")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let symbol = finding
        .get("symbol")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if (file.ends_with(".tsx") || file.ends_with(".jsx"))
        && symbol.chars().next().is_some_and(char::is_uppercase)
    {
        return true;
    }
    if symbol
        .strip_prefix("use")
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(char::is_uppercase)
    {
        return true;
    }
    let route_like = file.contains("/routes/")
        || file.contains("/pages/")
        || file.contains("/app/")
        || file.contains("/api/")
        || file.contains("/handlers/")
        || file.contains("/middleware/")
        || file.contains("/serverless/");
    route_like && (symbol == "default" || is_function_like_finding(finding))
}

fn symbol_graph_fan_in_zero(finding: &Value, symbols: Option<&Value>) -> bool {
    let identity = identity_for_finding(finding);
    symbols
        .and_then(|symbols| symbols.get("fanInByIdentity"))
        .and_then(|fan_in| fan_in.get(identity))
        .and_then(Value::as_i64)
        == Some(0)
}

fn call_graph_fan_in_zero(finding: &Value, call_graph: &Value) -> bool {
    let identity = identity_for_finding(finding);
    let definition_id = safe_action_definition_id(finding, call_graph);
    if call_graph
        .get("meta")
        .and_then(|meta| meta.get("supports"))
        .and_then(|supports| supports.get("callFanInByDefinitionId"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        if let Some(definition_id) = definition_id {
            if let Some(count) = call_graph
                .get("callFanInByDefinitionId")
                .and_then(|map| map.get(&definition_id))
                .and_then(Value::as_i64)
            {
                return count == 0;
            }
        }
    }
    if call_graph
        .get("meta")
        .and_then(|meta| meta.get("supports"))
        .and_then(|supports| supports.get("callFanInByIdentity"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        return call_graph
            .get("callFanInByIdentity")
            .and_then(|map| map.get(identity))
            .and_then(Value::as_i64)
            == Some(0);
    }
    false
}

fn safe_action_definition_id(finding: &Value, call_graph: &Value) -> Option<String> {
    finding
        .get("safeAction")
        .and_then(|safe_action| safe_action.get("target"))
        .and_then(|target| target.get("definitionId"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            call_graph
                .get("exportAliasMap")
                .and_then(|map| map.get(identity_for_finding(finding)))
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
}

fn identity_for_finding(finding: &Value) -> String {
    finding_identity(
        finding
            .get("file")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        finding
            .get("symbol")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
}

fn nearby_bounded_out_ratio(file: &str, call_graph: &Value) -> Option<f64> {
    let bounded = call_graph
        .get("boundedOutMemberCallsByFile")
        .and_then(|map| map.get(file))
        .and_then(Value::as_f64);
    let total = call_graph
        .get("memberCallsByFile")
        .and_then(|map| map.get(file))
        .and_then(Value::as_f64);
    if bounded.is_none() && total.is_none() {
        return Some(0.0);
    }
    Some(bounded.unwrap_or(0.0) / total.unwrap_or(0.0).max(1.0))
}

fn string_array_contains(value: Option<&Value>, needle: &str) -> bool {
    string_array(value).contains(needle)
}

fn string_array(value: Option<&Value>) -> BTreeSet<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(normalize_path_text)
        .collect()
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

fn scored_entry(finding: Value, evidence: Value, result: TierResult) -> Value {
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

fn by_tier(scored: Vec<Value>) -> BTreeMap<Tier, Vec<Value>> {
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

fn sort_tier_entries(by_tier: &mut BTreeMap<Tier, Vec<Value>>) {
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

fn summary(by_tier: &BTreeMap<Tier, Vec<Value>>, safe_fix_groups: usize) -> Value {
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

fn build_safe_fix_groups(safe_fixes: Option<&Vec<Value>>) -> Vec<Value> {
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
