use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap};

pub const RESOLVER_BLOCKED_CANDIDATE_HINT_SAMPLE_LIMIT: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverDiagnosticsSummary {
    pub resolver_version: Value,
    pub resolver_capability_artifact: Value,
    pub resolver_diagnostics_artifact: Value,
    pub unresolved_internal: Value,
    pub unresolved_internal_ratio: Value,
    pub blind_zone_count: Value,
    pub blocked_candidate_hint_count: Value,
    pub blocked_candidate_hint_sample_limit: Value,
    pub blocked_candidate_hints: Vec<Value>,
    pub blocked_candidate_hint_reason_counts: Vec<BlockedCandidateHintReasonCount>,
    pub blocked_candidate_hint_family_counts: Vec<BlockedCandidateHintFamilyCount>,
    pub candidate_target_count: Value,
    pub top_families: Value,
    pub top_affected_package_scopes: Value,
    pub top_unresolved_reasons: Vec<UnresolvedReasonCount>,
    pub top_specifier_roots: Vec<TopSpecifierRoot>,
    pub top_unresolved_specifiers: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockedCandidateHintReasonCount {
    pub reason: String,
    pub count: usize,
    pub families: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockedCandidateHintFamilyCount {
    pub family: String,
    pub count: usize,
    pub reasons: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnresolvedReasonCount {
    pub reason: String,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopSpecifierRoot {
    pub specifier_root: String,
    pub count: usize,
    pub reasons: BTreeMap<String, usize>,
    pub examples: Vec<TopSpecifierRootExample>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopSpecifierRootExample {
    pub specifier: Value,
    pub consumer_file: Value,
}

pub fn summarize_resolver_diagnostics(
    symbols: Option<&Value>,
    resolver_capabilities: Option<&Value>,
    resolver_diagnostics: Option<&Value>,
) -> ResolverDiagnosticsSummary {
    let resolver_capabilities_object = resolver_capabilities.and_then(Value::as_object);
    let resolver_diagnostics_object = resolver_diagnostics.and_then(Value::as_object);
    let resolver_diagnostics_summary =
        resolver_diagnostics_object.and_then(|artifact| object_field(artifact, "summary"));
    let blocked_candidate_hints = resolver_diagnostics_object
        .and_then(|artifact| artifact.get("blockedCandidateHints"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    ResolverDiagnosticsSummary {
        resolver_version: resolver_diagnostics_object
            .and_then(|artifact| artifact.get("resolverVersion"))
            .or_else(|| {
                resolver_capabilities_object.and_then(|artifact| artifact.get("resolverVersion"))
            })
            .cloned()
            .unwrap_or(Value::Null),
        resolver_capability_artifact: if resolver_capabilities.is_some() {
            json!("resolver-capabilities.json")
        } else {
            Value::Null
        },
        resolver_diagnostics_artifact: if resolver_diagnostics.is_some() {
            json!("resolver-diagnostics.json")
        } else {
            Value::Null
        },
        unresolved_internal: nested_symbols_field(symbols, "uses", "unresolvedInternal"),
        unresolved_internal_ratio: nested_symbols_field(symbols, "uses", "unresolvedInternalRatio"),
        blind_zone_count: nested_field_or_null(resolver_diagnostics_summary, "blindZoneCount"),
        blocked_candidate_hint_count: nested_field_or_null(
            resolver_diagnostics_summary,
            "blockedCandidateHintCount",
        ),
        blocked_candidate_hint_sample_limit: if resolver_diagnostics.is_some() {
            json!(RESOLVER_BLOCKED_CANDIDATE_HINT_SAMPLE_LIMIT)
        } else {
            Value::Null
        },
        blocked_candidate_hints: blocked_candidate_hints
            .iter()
            .take(RESOLVER_BLOCKED_CANDIDATE_HINT_SAMPLE_LIMIT)
            .cloned()
            .collect(),
        blocked_candidate_hint_reason_counts: build_blocked_candidate_hint_reason_counts(
            blocked_candidate_hints,
        ),
        blocked_candidate_hint_family_counts: build_blocked_candidate_hint_family_counts(
            blocked_candidate_hints,
        ),
        candidate_target_count: nested_field_or_null(
            resolver_diagnostics_summary,
            "candidateTargetCount",
        ),
        top_families: nested_field_or_empty_array(resolver_diagnostics_summary, "topFamilies"),
        top_affected_package_scopes: nested_field_or_empty_array(
            resolver_diagnostics_summary,
            "topAffectedPackageScopes",
        ),
        top_unresolved_reasons: resolver_diagnostics_summary
            .and_then(|summary| summary.get("topUnresolvedReasons"))
            .and_then(Value::as_array)
            .map(|items| json_reason_counts(items))
            .unwrap_or_else(|| top_unresolved_reasons(symbols)),
        top_specifier_roots: resolver_diagnostics_summary
            .and_then(|summary| summary.get("topSpecifierRoots"))
            .and_then(Value::as_array)
            .map(|items| json_specifier_roots(items))
            .unwrap_or_else(|| build_top_specifier_roots(symbols)),
        top_unresolved_specifiers: symbols
            .and_then(|symbols| symbols.get("topUnresolvedSpecifiers"))
            .and_then(Value::as_array)
            .map(|items| items.iter().take(20).cloned().collect())
            .unwrap_or_default(),
    }
}

fn top_unresolved_reasons(symbols: Option<&Value>) -> Vec<UnresolvedReasonCount> {
    let from_summary = symbols
        .and_then(|symbols| symbols.get("unresolvedInternalSummaryByReason"))
        .and_then(count_object_from_summary);
    let mut reasons = from_summary.unwrap_or_else(|| {
        let mut counts = HashMap::<String, u64>::new();
        for record in unresolved_records(symbols) {
            let reason = record
                .as_object()
                .and_then(|record| record.get("reason"))
                .and_then(Value::as_str)
                .unwrap_or("unknown-internal-resolution");
            *counts.entry(reason.to_string()).or_default() += 1;
        }
        counts
            .into_iter()
            .map(|(reason, count)| UnresolvedReasonCount { reason, count })
            .collect()
    });
    reasons.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.reason.cmp(&right.reason))
    });
    reasons.truncate(10);
    reasons
}

fn build_top_specifier_roots(symbols: Option<&Value>) -> Vec<TopSpecifierRoot> {
    let mut groups = BTreeMap::<String, TopSpecifierRootBuilder>::new();
    for record in unresolved_records(symbols) {
        let Some(record_object) = record.as_object() else {
            continue;
        };
        let Some(specifier_root) = record_object
            .get("specifier")
            .and_then(Value::as_str)
            .and_then(unresolved_specifier_root)
        else {
            continue;
        };
        let group = groups
            .entry(specifier_root.clone())
            .or_insert_with(|| TopSpecifierRootBuilder::new(specifier_root));
        let reason = record_object
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("unknown-internal-resolution");
        group.count += 1;
        *group.reasons.entry(reason.to_string()).or_default() += 1;
        group.examples.push(TopSpecifierRootExample {
            specifier: record_object
                .get("specifier")
                .cloned()
                .unwrap_or(Value::Null),
            consumer_file: record_object
                .get("consumerFile")
                .cloned()
                .unwrap_or(Value::Null),
        });
    }

    let mut roots = groups
        .into_values()
        .map(TopSpecifierRootBuilder::finish)
        .collect::<Vec<_>>();
    roots.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.specifier_root.cmp(&right.specifier_root))
    });
    roots.truncate(20);
    roots
}

fn build_blocked_candidate_hint_reason_counts(
    hints: &[Value],
) -> Vec<BlockedCandidateHintReasonCount> {
    let mut groups = BTreeMap::<String, HintReasonCountBuilder>::new();
    for hint in hints {
        let Some(hint_object) = hint.as_object() else {
            continue;
        };
        let Some(reason) = hint_object.get("reason").and_then(Value::as_str) else {
            continue;
        };
        if reason.is_empty() {
            continue;
        }
        let family = hint_object
            .get("family")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let group = groups
            .entry(reason.to_string())
            .or_insert_with(|| HintReasonCountBuilder::new(reason.to_string()));
        group.count += 1;
        *group.families.entry(family.to_string()).or_default() += 1;
    }

    let mut counts = groups
        .into_values()
        .map(HintReasonCountBuilder::finish)
        .collect::<Vec<_>>();
    counts.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.reason.cmp(&right.reason))
    });
    counts.truncate(20);
    counts
}

fn build_blocked_candidate_hint_family_counts(
    hints: &[Value],
) -> Vec<BlockedCandidateHintFamilyCount> {
    let mut groups = BTreeMap::<String, HintFamilyCountBuilder>::new();
    for hint in hints {
        let hint_object = hint.as_object();
        let family = hint_object
            .and_then(|hint| hint.get("family"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let reason = hint_object
            .and_then(|hint| hint.get("reason"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let group = groups
            .entry(family.to_string())
            .or_insert_with(|| HintFamilyCountBuilder::new(family.to_string()));
        group.count += 1;
        *group.reasons.entry(reason.to_string()).or_default() += 1;
    }

    let mut counts = groups
        .into_values()
        .map(HintFamilyCountBuilder::finish)
        .collect::<Vec<_>>();
    counts.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.family.cmp(&right.family))
    });
    counts.truncate(20);
    counts
}

fn unresolved_specifier_root(specifier: &str) -> Option<String> {
    if specifier.is_empty() {
        return None;
    }
    if specifier.starts_with("@/") || specifier.starts_with("~/") || specifier.starts_with("#/") {
        return Some(specifier[..2].to_string());
    }
    if specifier.starts_with('@') {
        let mut parts = specifier.split('/');
        let first = parts.next().unwrap_or_default();
        let second = parts.next().unwrap_or_default();
        if !first.is_empty() && !second.is_empty() {
            return Some(format!("{first}/{second}"));
        }
    }
    specifier
        .split('/')
        .next()
        .filter(|first| !first.is_empty())
        .map(ToOwned::to_owned)
}

fn count_object_from_summary(summary: &Value) -> Option<Vec<UnresolvedReasonCount>> {
    let summary = summary.as_object()?;
    let mut out = Vec::new();
    for (reason, value) in summary {
        let count = value
            .as_object()
            .and_then(|value| value.get("count"))
            .and_then(Value::as_u64)
            .or_else(|| value.as_u64());
        if let Some(count) = count {
            out.push(UnresolvedReasonCount {
                reason: reason.clone(),
                count,
            });
        }
    }
    (!out.is_empty()).then_some(out)
}

fn json_reason_counts(items: &[Value]) -> Vec<UnresolvedReasonCount> {
    items
        .iter()
        .filter_map(|item| {
            let item = item.as_object()?;
            Some(UnresolvedReasonCount {
                reason: item.get("reason")?.as_str()?.to_string(),
                count: item.get("count")?.as_u64()?,
            })
        })
        .collect()
}

fn json_specifier_roots(items: &[Value]) -> Vec<TopSpecifierRoot> {
    items
        .iter()
        .filter_map(|item| {
            let item = item.as_object()?;
            Some(TopSpecifierRoot {
                specifier_root: item.get("specifierRoot")?.as_str()?.to_string(),
                count: item.get("count")?.as_u64()?.try_into().ok()?,
                reasons: value_object_to_count_map(item.get("reasons")),
                examples: item
                    .get("examples")
                    .and_then(Value::as_array)
                    .map(|examples| {
                        examples
                            .iter()
                            .filter_map(|example| {
                                let example = example.as_object()?;
                                Some(TopSpecifierRootExample {
                                    specifier: example
                                        .get("specifier")
                                        .cloned()
                                        .unwrap_or(Value::Null),
                                    consumer_file: example
                                        .get("consumerFile")
                                        .cloned()
                                        .unwrap_or(Value::Null),
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect()
}

fn value_object_to_count_map(value: Option<&Value>) -> BTreeMap<String, usize> {
    value
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .filter_map(|(key, value)| {
                    value
                        .as_u64()
                        .and_then(|value| value.try_into().ok())
                        .map(|value| (key.clone(), value))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn unresolved_records(symbols: Option<&Value>) -> &[Value] {
    symbols
        .and_then(|symbols| symbols.get("unresolvedInternalSpecifierRecords"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn nested_symbols_field(symbols: Option<&Value>, object_field: &str, value_field: &str) -> Value {
    symbols
        .and_then(|symbols| symbols.get(object_field))
        .and_then(Value::as_object)
        .and_then(|object| object.get(value_field))
        .cloned()
        .unwrap_or(Value::Null)
}

fn object_field<'a>(object: &'a Map<String, Value>, field: &str) -> Option<&'a Map<String, Value>> {
    object.get(field).and_then(Value::as_object)
}

fn nested_field_or_null(object: Option<&Map<String, Value>>, field: &str) -> Value {
    object
        .and_then(|object| object.get(field))
        .cloned()
        .unwrap_or(Value::Null)
}

fn nested_field_or_empty_array(object: Option<&Map<String, Value>>, field: &str) -> Value {
    object
        .and_then(|object| object.get(field))
        .cloned()
        .unwrap_or_else(|| json!([]))
}

#[derive(Debug)]
struct TopSpecifierRootBuilder {
    specifier_root: String,
    count: usize,
    reasons: BTreeMap<String, usize>,
    examples: Vec<TopSpecifierRootExample>,
}

impl TopSpecifierRootBuilder {
    fn new(specifier_root: String) -> Self {
        Self {
            specifier_root,
            count: 0,
            reasons: BTreeMap::new(),
            examples: Vec::new(),
        }
    }

    fn finish(mut self) -> TopSpecifierRoot {
        self.examples
            .sort_by(|left, right| example_sort_key(left).cmp(&example_sort_key(right)));
        self.examples.truncate(5);
        TopSpecifierRoot {
            specifier_root: self.specifier_root,
            count: self.count,
            reasons: self.reasons,
            examples: self.examples,
        }
    }
}

fn example_sort_key(example: &TopSpecifierRootExample) -> String {
    format!(
        "{}|{}",
        example.consumer_file.as_str().unwrap_or(""),
        example.specifier.as_str().unwrap_or("")
    )
}

#[derive(Debug)]
struct HintReasonCountBuilder {
    reason: String,
    count: usize,
    families: BTreeMap<String, usize>,
}

impl HintReasonCountBuilder {
    fn new(reason: String) -> Self {
        Self {
            reason,
            count: 0,
            families: BTreeMap::new(),
        }
    }

    fn finish(self) -> BlockedCandidateHintReasonCount {
        BlockedCandidateHintReasonCount {
            reason: self.reason,
            count: self.count,
            families: self.families,
        }
    }
}

#[derive(Debug)]
struct HintFamilyCountBuilder {
    family: String,
    count: usize,
    reasons: BTreeMap<String, usize>,
}

impl HintFamilyCountBuilder {
    fn new(family: String) -> Self {
        Self {
            family,
            count: 0,
            reasons: BTreeMap::new(),
        }
    }

    fn finish(self) -> BlockedCandidateHintFamilyCount {
        BlockedCandidateHintFamilyCount {
            family: self.family,
            count: self.count,
            reasons: self.reasons,
        }
    }
}
