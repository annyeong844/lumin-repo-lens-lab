use serde_json::{Map, Value};
use std::collections::BTreeSet;

const TAINT_UNRESOLVED_SPEC_MATCH: &str = "unresolved-specifier-could-match";
const TAINT_UNRESOLVED_SPEC_MATCH_UNKNOWN: &str = "unresolved-specifier-could-match-unknown";
const TAINT_RESOLVER_BLIND_ZONE_RELEVANT: &str = "resolver-blind-zone-relevant";
const TAINT_GENERATED_ARTIFACT_MISSING_RELEVANT: &str = "generated-artifact-missing-relevant";
const TAINT_DEFINING_FILE_PARSE_ERROR: &str = "defining-file-parse-error";
const TAINT_PARSE_ERRORS_ELSEWHERE: &str = "parse-errors-present";
const GENERATED_ARTIFACT_MISSING_REASON: &str = "workspace-generated-artifact-missing";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Tier {
    SafeFix,
    ReviewFix,
    Degraded,
    Muted,
}

impl Tier {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Tier::SafeFix => "SAFE_FIX",
            Tier::ReviewFix => "REVIEW_FIX",
            Tier::Degraded => "DEGRADED",
            Tier::Muted => "MUTED",
        }
    }
}

#[derive(Debug, Default)]
struct TaintState {
    per_finding_taint_present: bool,
    has_blocking_taint: bool,
    has_soft_taint: bool,
}

#[derive(Debug)]
struct SafeActionState {
    action_blockers: Vec<String>,
    has_safe_action: bool,
    preserves_declaration_binding: bool,
    is_resolvable_declaration_dependency_bucket: bool,
}

#[derive(Debug)]
struct SupportState {
    strong_runtime: bool,
    has_entry_reach_support: bool,
    has_independent_support: bool,
}

#[derive(Debug)]
pub(super) struct TierResult {
    pub(super) tier: Tier,
    pub(super) reason: String,
    pub(super) confidence: Option<&'static str>,
    pub(super) confidence_detail: Option<&'static str>,
    pub(super) blocked_promotion: bool,
    pub(super) blocked_by: Vec<Value>,
}

impl TierResult {
    fn new(tier: Tier, reason: impl Into<String>) -> Self {
        Self {
            tier,
            reason: reason.into(),
            confidence: None,
            confidence_detail: None,
            blocked_promotion: false,
            blocked_by: Vec::new(),
        }
    }
}

pub(super) fn tier_for_finding(finding: &Value, evidence: &Value) -> TierResult {
    let runtime = evidence.get("runtime");
    let staleness = evidence.get("staleness");
    let contract = evidence.get("contract").unwrap_or(&Value::Null);
    let entry_surface = evidence.get("entrySurface").unwrap_or(&Value::Null);
    let policy = evidence.get("policy").unwrap_or(&Value::Null);
    let resolver = evidence.get("resolver");

    if let Some(result) = policy_exclusion_result(policy) {
        return result;
    }
    if let Some(result) = runtime_contradiction_result(runtime) {
        return result;
    }
    let taints = taint_state(finding);
    if let Some(result) = blocking_taint_result(finding, &taints) {
        return result;
    }
    if let Some(result) = legacy_resolver_blindness_result(&taints, resolver) {
        return result;
    }
    let has_weak_runtime_status = weak_runtime_status(runtime);
    if let Some(result) = incomplete_bucket_result(finding) {
        return result;
    }
    let safe_action = safe_action_state(finding);
    if let Some(result) = missing_safe_action_result(&safe_action) {
        return result;
    }
    if let Some(result) = declaration_dependency_result(finding, &safe_action) {
        return result;
    }
    if let Some(result) = bucket_b_result(finding, &safe_action) {
        return result;
    }
    let support = support_state(finding, runtime);
    if let Some(result) = html_entrypoint_blind_zone_result(entry_surface) {
        return result;
    }
    if let Some(result) = public_deep_import_risk_result(contract) {
        return result;
    }
    if !taints.has_soft_taint && !has_weak_runtime_status {
        return safe_fix_result(finding, runtime, staleness, &support);
    }
    if let Some(result) = weaker_evidence_review_result(
        finding,
        runtime,
        &taints,
        &safe_action,
        has_weak_runtime_status,
    ) {
        return result;
    }
    TierResult::new(
        Tier::Degraded,
        format!(
            "unclassified bucket={}",
            finding
                .get("bucket")
                .and_then(Value::as_str)
                .unwrap_or_default()
        ),
    )
}

fn policy_exclusion_result(policy: &Value) -> Option<TierResult> {
    if policy.get("excluded").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    let reason = policy
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    Some(TierResult::new(
        Tier::Muted,
        format!("policy-excluded: {reason}"),
    ))
}

fn runtime_contradiction_result(runtime: Option<&Value>) -> Option<TierResult> {
    let runtime = runtime?;
    if runtime.get("status").and_then(Value::as_str) != Some("executed") {
        return None;
    }
    let hits = runtime
        .get("hitsInSymbol")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    Some(TierResult::new(
        Tier::Degraded,
        format!("runtime-executed ({hits} hits)"),
    ))
}

fn taint_state(finding: &Value) -> TaintState {
    let taints = finding.get("taintedBy").and_then(Value::as_array);
    let Some(taints) = taints else {
        return TaintState::default();
    };
    let mut state = TaintState {
        per_finding_taint_present: true,
        ..TaintState::default()
    };
    for taint in taints {
        match taint
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or_default()
        {
            TAINT_UNRESOLVED_SPEC_MATCH | TAINT_DEFINING_FILE_PARSE_ERROR => {
                state.has_blocking_taint = true;
            }
            TAINT_UNRESOLVED_SPEC_MATCH_UNKNOWN
            | TAINT_RESOLVER_BLIND_ZONE_RELEVANT
            | TAINT_GENERATED_ARTIFACT_MISSING_RELEVANT
            | TAINT_PARSE_ERRORS_ELSEWHERE => {
                state.has_soft_taint = true;
            }
            _ => {}
        }
    }
    state
}

fn blocking_taint_result(finding: &Value, taints: &TaintState) -> Option<TierResult> {
    if !taints.has_blocking_taint {
        return None;
    }
    let blocker = finding
        .get("taintedBy")
        .and_then(Value::as_array)?
        .iter()
        .find(|taint| {
            matches!(
                taint
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                TAINT_UNRESOLVED_SPEC_MATCH | TAINT_DEFINING_FILE_PARSE_ERROR
            )
        })?;
    match blocker
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        TAINT_UNRESOLVED_SPEC_MATCH => {
            let spec = blocker
                .get("specifiers")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .and_then(Value::as_str)
                .unwrap_or("<specifier>");
            let total = blocker.get("total").and_then(Value::as_i64).unwrap_or(0);
            Some(TierResult::new(
                Tier::Degraded,
                format!(
                    "unresolved-spec-could-match: {spec} ({total} match{})",
                    if total == 1 { "" } else { "es" }
                ),
            ))
        }
        _ => {
            let file = blocker
                .get("file")
                .and_then(Value::as_str)
                .unwrap_or_default();
            Some(TierResult::new(
                Tier::Degraded,
                format!("defining-file-parse-error: {file}"),
            ))
        }
    }
}

fn legacy_resolver_blindness_result(
    taints: &TaintState,
    resolver: Option<&Value>,
) -> Option<TierResult> {
    if taints.per_finding_taint_present {
        return None;
    }
    let ratio = resolver?.get("unresolvedRatio").and_then(Value::as_f64)?;
    if ratio < 0.15 {
        return None;
    }
    Some(TierResult::new(
        Tier::Degraded,
        format!("resolver-blind (unresolvedRatio={ratio:.3}, no per-finding taint)"),
    ))
}

fn weak_runtime_status(runtime: Option<&Value>) -> bool {
    matches!(
        runtime
            .and_then(|runtime| runtime.get("status"))
            .and_then(Value::as_str),
        Some("uncovered" | "type-only")
    )
}

fn incomplete_bucket_result(finding: &Value) -> Option<TierResult> {
    if finding.get("bucket").and_then(Value::as_str) != Some("unprocessed") {
        return None;
    }
    Some(TierResult::new(
        Tier::Degraded,
        format!(
            "classify-incomplete: {}",
            finding
                .get("action")
                .and_then(Value::as_str)
                .unwrap_or("candidate was not fully classified")
        ),
    ))
}

fn safe_action_state(finding: &Value) -> SafeActionState {
    let action_blockers = array_strings(
        finding
            .get("safeAction")
            .and_then(|safe_action| safe_action.get("actionBlockers"))
            .or_else(|| finding.get("actionBlockers")),
    );
    let safe_action_kind = finding
        .get("safeAction")
        .and_then(|safe_action| safe_action.get("kind"))
        .and_then(Value::as_str);
    let proof_complete = finding
        .get("safeAction")
        .and_then(|safe_action| safe_action.get("proofComplete"))
        .and_then(Value::as_bool)
        == Some(true);
    let preserves_declaration_binding = matches!(
        safe_action_kind,
        Some("demote_export_declaration" | "remove_export_specifier")
    );
    let declaration_dependency = finding
        .get("declarationExportDependency")
        .and_then(Value::as_bool)
        == Some(true);
    SafeActionState {
        has_safe_action: safe_action_kind.is_some() && proof_complete && action_blockers.is_empty(),
        action_blockers,
        preserves_declaration_binding,
        is_resolvable_declaration_dependency_bucket: finding.get("bucket").and_then(Value::as_str)
            == Some("B")
            && declaration_dependency
            && preserves_declaration_binding,
    }
}

fn array_strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str().map(ToString::to_string))
        .collect()
}

fn missing_safe_action_result(safe_action: &SafeActionState) -> Option<TierResult> {
    if safe_action.has_safe_action {
        return None;
    }
    if !safe_action.action_blockers.is_empty() {
        return Some(TierResult::new(
            Tier::ReviewFix,
            format!(
                "action-blockers: {}",
                safe_action.action_blockers.join(", ")
            ),
        ));
    }
    Some(TierResult::new(
        Tier::ReviewFix,
        "missing-safe-action-proof",
    ))
}

fn declaration_dependency_result(
    finding: &Value,
    safe_action: &SafeActionState,
) -> Option<TierResult> {
    if finding
        .get("declarationExportDependency")
        .and_then(Value::as_bool)
        != Some(true)
        || safe_action.preserves_declaration_binding
    {
        return None;
    }
    let count = finding
        .get("declarationExportRefs")
        .and_then(|refs| refs.get("count"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    Some(TierResult::new(
        Tier::ReviewFix,
        format!(
            "declaration-dependency-not-preserved ({count} ref{})",
            if count == 1 { "" } else { "s" }
        ),
    ))
}

fn bucket_b_result(finding: &Value, safe_action: &SafeActionState) -> Option<TierResult> {
    if finding.get("bucket").and_then(Value::as_str) != Some("B")
        || safe_action.is_resolvable_declaration_dependency_bucket
    {
        return None;
    }
    Some(TierResult::new(
        Tier::ReviewFix,
        "bucket-B (design review required)",
    ))
}

fn support_state(finding: &Value, runtime: Option<&Value>) -> SupportState {
    let supported_by = finding.get("supportedBy").and_then(Value::as_array);
    let has_entry_reach_support = supported_by
        .into_iter()
        .flatten()
        .any(|support| support.get("kind").and_then(Value::as_str) == Some("entry-unreachable"));
    let has_independent_support = finding
        .get("supportedBy")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|support| {
            support.get("kind").and_then(Value::as_str) == Some("call-graph-no-observed-callers")
        });
    let strong_runtime = runtime
        .and_then(|runtime| runtime.get("status"))
        .and_then(Value::as_str)
        == Some("dead-confirmed")
        && runtime
            .and_then(|runtime| runtime.get("grounding"))
            .and_then(Value::as_str)
            == Some("grounded");
    SupportState {
        strong_runtime,
        has_entry_reach_support,
        has_independent_support,
    }
}

fn html_entrypoint_blind_zone_result(entry_surface: &Value) -> Option<TierResult> {
    let blind_zone = entry_surface.get("htmlEntrypointBlindZone")?;
    if blind_zone.is_null() {
        return None;
    }
    let mut result = TierResult::new(Tier::ReviewFix, "html-entry-surface-blind-zone");
    result.blocked_promotion = true;
    result.blocked_by.push(blind_zone.clone());
    Some(result)
}

fn public_deep_import_risk_result(contract: &Value) -> Option<TierResult> {
    if contract
        .get("publicDeepImportRisk")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return None;
    }
    let detail_reason = contract
        .get("publicDeepImportRiskDetail")
        .and_then(|detail| detail.get("reason"))
        .and_then(Value::as_str);
    Some(TierResult::new(
        Tier::ReviewFix,
        detail_reason
            .map(|reason| format!("public-deep-import-risk: {reason}"))
            .unwrap_or_else(|| "public-deep-import-risk".to_string()),
    ))
}

fn safe_fix_result(
    finding: &Value,
    runtime: Option<&Value>,
    staleness: Option<&Value>,
    support: &SupportState,
) -> TierResult {
    let mut bits = vec![
        "safe-action".to_string(),
        "static-graph-clean".to_string(),
        format!(
            "bucket-{}",
            finding
                .get("bucket")
                .and_then(Value::as_str)
                .unwrap_or_default()
        ),
    ];
    if support.has_entry_reach_support {
        bits.push("entry-unreachable".to_string());
    }
    if support.has_independent_support {
        bits.push("no-observed-callers".to_string());
    }
    if support.strong_runtime {
        bits.push("runtime-dead-confirmed".to_string());
    } else if let Some(status) = runtime
        .and_then(|runtime| runtime.get("status"))
        .and_then(Value::as_str)
    {
        bits.push(format!("runtime-{status}"));
    } else {
        bits.push("no-runtime".to_string());
    }
    if let Some(tier) = staleness
        .and_then(|staleness| staleness.get("tier"))
        .and_then(Value::as_str)
    {
        bits.push(format!("staleness-{tier}"));
    } else {
        bits.push("no-staleness".to_string());
    }
    let mut result = TierResult::new(Tier::SafeFix, bits.join(" + "));
    result.confidence = Some("medium");
    if support.has_entry_reach_support && support.has_independent_support {
        result.confidence = Some("high");
        result.confidence_detail = Some("high_two_lens_evidence");
    } else if support.has_entry_reach_support || support.has_independent_support {
        result.confidence = Some("medium");
        result.confidence_detail = Some("medium_with_evidence");
    }
    result
}

fn weaker_evidence_review_result(
    finding: &Value,
    runtime: Option<&Value>,
    taints: &TaintState,
    safe_action: &SafeActionState,
    has_weak_runtime_status: bool,
) -> Option<TierResult> {
    let bucket = finding
        .get("bucket")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !matches!(bucket, "C" | "A" | "specifier")
        && !safe_action.is_resolvable_declaration_dependency_bucket
    {
        return None;
    }
    let mut missing = Vec::new();
    if taints.has_soft_taint {
        missing.extend(soft_taint_reason_labels(finding));
    }
    if has_weak_runtime_status {
        if let Some(status) = runtime
            .and_then(|runtime| runtime.get("status"))
            .and_then(Value::as_str)
        {
            missing.push(format!("runtime={status}"));
        }
    }
    let blocked_by = generated_artifact_blocking_diagnostics(finding)
        .into_iter()
        .chain(resolver_blind_zone_blocking_diagnostics(finding))
        .collect::<Vec<_>>();
    let mut result = TierResult::new(
        Tier::ReviewFix,
        format!(
            "safe-action; missing: {}",
            if missing.is_empty() {
                "none".to_string()
            } else {
                unique_sorted(missing).join(", ")
            }
        ),
    );
    if !blocked_by.is_empty() {
        result.blocked_promotion = true;
        result.blocked_by = blocked_by;
    }
    Some(result)
}

fn soft_taint_reason_labels(finding: &Value) -> Vec<String> {
    finding
        .get("taintedBy")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|taint| {
            match taint
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or_default()
            {
                TAINT_PARSE_ERRORS_ELSEWHERE => Some("parse-errors-elsewhere".to_string()),
                TAINT_UNRESOLVED_SPEC_MATCH_UNKNOWN => {
                    Some(TAINT_UNRESOLVED_SPEC_MATCH_UNKNOWN.to_string())
                }
                TAINT_RESOLVER_BLIND_ZONE_RELEVANT => Some("resolver-blind-zone".to_string()),
                TAINT_GENERATED_ARTIFACT_MISSING_RELEVANT => {
                    Some("workspace-generated-artifact-missing".to_string())
                }
                _ => None,
            }
        })
        .collect()
}

fn unique_sorted(items: Vec<String>) -> Vec<String> {
    let mut set = BTreeSet::new();
    for item in items {
        set.insert(item);
    }
    set.into_iter().collect()
}

fn generated_artifact_blocking_diagnostics(finding: &Value) -> Vec<Value> {
    finding
        .get("taintedBy")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|taint| {
            taint.get("kind").and_then(Value::as_str)
                == Some(TAINT_GENERATED_ARTIFACT_MISSING_RELEVANT)
        })
        .map(|taint| {
            let fields = [
                "kind",
                "specifier",
                "specifiers",
                "total",
                "consumerFile",
                "fromHint",
                "matchedPackage",
                "targetSubpath",
                "generatorFamily",
                "confidence",
                "candidatePath",
                "status",
                "scopePackageRoot",
                "scanScopeReason",
                "staleStatus",
                "staleReason",
                "impact",
                "relevance",
                "effect",
            ];
            diagnostic_from_taint(taint, &fields, GENERATED_ARTIFACT_MISSING_REASON)
        })
        .collect()
}

fn resolver_blind_zone_blocking_diagnostics(finding: &Value) -> Vec<Value> {
    finding
        .get("taintedBy")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|taint| {
            taint.get("kind").and_then(Value::as_str) == Some(TAINT_RESOLVER_BLIND_ZONE_RELEVANT)
        })
        .map(|taint| {
            let fields = [
                "kind",
                "family",
                "specifier",
                "specifiers",
                "total",
                "consumerFile",
                "fromHint",
                "targetCandidates",
                "affectedPackageScope",
                "resolverStage",
                "outputLevel",
                "impact",
                "relevance",
                "effect",
            ];
            diagnostic_from_taint(taint, &fields, "resolver-blind-zone")
        })
        .collect()
}

fn diagnostic_from_taint(taint: &Value, fields: &[&str], default_reason: &str) -> Value {
    let mut object = Map::new();
    object.insert(
        "reason".to_string(),
        taint
            .get("reason")
            .cloned()
            .unwrap_or_else(|| Value::String(default_reason.to_string())),
    );
    for field in fields {
        if let Some(value) = taint.get(*field) {
            object.insert((*field).to_string(), value.clone());
        }
    }
    Value::Object(object)
}
