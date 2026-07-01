use anyhow::{bail, Result};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::scan_scope::{scan_scope_status_for_path, to_repo_relative, ScanScopeOptions};

pub const GENERATED_ARTIFACT_POLICY_VERSION: &str = "generated-artifact-policy-v1";
pub const GENERATED_ARTIFACT_MISSING_REASON: &str = "workspace-generated-artifact-missing";

#[derive(Debug, Clone)]
pub struct GeneratedArtifactsOptions {
    pub include_tests: bool,
    pub excludes: Vec<String>,
    pub mode: GeneratedArtifactsMode,
}

impl Default for GeneratedArtifactsOptions {
    fn default() -> Self {
        Self {
            include_tests: true,
            excludes: Vec::new(),
            mode: GeneratedArtifactsMode::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedArtifactsMode {
    Default,
    Present,
    Prepared,
}

impl Default for GeneratedArtifactsMode {
    fn default() -> Self {
        Self::Default
    }
}

impl GeneratedArtifactsMode {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim() {
            "" | "default" => Ok(Self::Default),
            "present" => Ok(Self::Present),
            "prepared" => Ok(Self::Prepared),
            mode => bail!(
                "unsupported --generated-artifacts mode: {mode}. Use default|present|prepared."
            ),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Present => "present",
            Self::Prepared => "prepared",
        }
    }

    fn records_present_targets(self) -> bool {
        !matches!(self, Self::Default)
    }
}

impl Serialize for GeneratedArtifactsMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedArtifactsSummary {
    pub mode: GeneratedArtifactsMode,
    pub generated_artifact_policy_version: &'static str,
    pub executed_generators: bool,
    pub reason_summary: BTreeMap<String, u64>,
    pub top_generated_misses: Vec<GeneratedMiss>,
    pub generated_consumer_blind_zone_count: usize,
    pub top_generated_consumer_blind_zones: Vec<GeneratedConsumerBlindZoneSummary>,
    pub present_but_out_of_scope_count: usize,
    pub present_but_out_of_scope: Vec<PresentButOutOfScope>,
    pub supported_generators: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedMiss {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specifier: Option<String>,
    pub matched_package: Option<String>,
    pub target_subpath: Option<String>,
    pub count: u64,
    pub generator_family: Option<String>,
    pub confidence: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedConsumerBlindZoneSummary {
    pub scope_package_root: String,
    pub count: u64,
    pub statuses: BTreeMap<String, u64>,
    pub top_specifiers: Vec<GeneratedConsumerTopSpecifier>,
    pub examples: Vec<GeneratedConsumerBlindZoneExample>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GeneratedConsumerTopSpecifier {
    pub specifier: String,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedConsumerBlindZoneExample {
    pub specifier: Option<String>,
    pub consumer_file: Option<String>,
    pub candidate_path: Option<String>,
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_scope_reason: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentButOutOfScope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specifier: Option<String>,
    pub consumer_file: Option<String>,
    pub matched_package: Option<String>,
    pub target_subpath: Option<String>,
    pub candidate_path: String,
    pub reason: &'static str,
    pub mode: GeneratedArtifactsMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_status: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_reason: Option<&'static str>,
}

#[derive(Debug, Clone)]
struct GeneratedMissAccumulator {
    miss: GeneratedMiss,
}

#[derive(Debug, Clone)]
struct BlindZoneGroup<'a> {
    scope_package_root: String,
    count: u64,
    statuses: BTreeMap<String, u64>,
    specifiers: BTreeMap<String, u64>,
    zones: Vec<&'a Value>,
}

pub fn summarize_generated_artifacts(
    root: &Path,
    symbols: Option<&Value>,
    options: &GeneratedArtifactsOptions,
) -> GeneratedArtifactsSummary {
    let mut reason_summary = BTreeMap::new();
    let mut misses: BTreeMap<String, GeneratedMissAccumulator> = BTreeMap::new();
    let mut present_but_out_of_scope = Vec::new();
    let mut present_keys = BTreeSet::new();

    let generated_consumer_blind_zones = symbols
        .and_then(|symbols| symbols.get("generatedConsumerBlindZones"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    let records = symbols
        .and_then(|symbols| symbols.get("unresolvedInternalSpecifierRecords"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    for record in records {
        let Some(record) = record.as_object() else {
            continue;
        };
        if record.get("reason").and_then(Value::as_str) != Some(GENERATED_ARTIFACT_MISSING_REASON) {
            continue;
        }
        *reason_summary
            .entry(GENERATED_ARTIFACT_MISSING_REASON.to_string())
            .or_insert(0) += 1;

        let generated_artifact = record.get("generatedArtifact").and_then(Value::as_object);
        let specifier = string_field(record.get("specifier"));
        let matched_package = object_string_field(generated_artifact, "matchedPackage");
        let target_subpath = object_string_field(generated_artifact, "targetSubpath");
        let generator_family = object_string_field(generated_artifact, "generatorFamily");
        let confidence = object_string_field(generated_artifact, "confidence");
        let key = [
            specifier.as_deref().unwrap_or(""),
            matched_package.as_deref().unwrap_or(""),
            target_subpath.as_deref().unwrap_or(""),
            generator_family.as_deref().unwrap_or(""),
            confidence.as_deref().unwrap_or(""),
        ]
        .join("|");
        misses
            .entry(key)
            .and_modify(|entry| entry.miss.count += 1)
            .or_insert_with(|| GeneratedMissAccumulator {
                miss: GeneratedMiss {
                    specifier: specifier.clone(),
                    matched_package: matched_package.clone(),
                    target_subpath: target_subpath.clone(),
                    count: 1,
                    generator_family: generator_family.clone(),
                    confidence: confidence.clone(),
                },
            });

        if options.mode.records_present_targets() {
            collect_present_out_of_scope(
                root,
                record.get("targetCandidates"),
                specifier,
                string_field(record.get("consumerFile")),
                matched_package,
                target_subpath,
                options,
                &mut present_keys,
                &mut present_but_out_of_scope,
            );
        }
    }

    let mut top_generated_misses = misses
        .into_values()
        .map(|entry| entry.miss)
        .collect::<Vec<_>>();
    top_generated_misses.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| {
                optional_string(left.matched_package.as_ref())
                    .cmp(optional_string(right.matched_package.as_ref()))
            })
            .then_with(|| {
                optional_string(left.specifier.as_ref())
                    .cmp(optional_string(right.specifier.as_ref()))
            })
    });
    top_generated_misses.truncate(20);

    present_but_out_of_scope.sort_by(|left, right| {
        left.candidate_path
            .cmp(&right.candidate_path)
            .then_with(|| {
                optional_string(left.specifier.as_ref())
                    .cmp(optional_string(right.specifier.as_ref()))
            })
            .then_with(|| {
                optional_string(left.consumer_file.as_ref())
                    .cmp(optional_string(right.consumer_file.as_ref()))
            })
    });

    GeneratedArtifactsSummary {
        mode: options.mode,
        generated_artifact_policy_version: GENERATED_ARTIFACT_POLICY_VERSION,
        executed_generators: false,
        reason_summary,
        top_generated_misses,
        generated_consumer_blind_zone_count: generated_consumer_blind_zones.len(),
        top_generated_consumer_blind_zones: build_generated_consumer_blind_zone_summary(
            generated_consumer_blind_zones,
        ),
        present_but_out_of_scope_count: present_but_out_of_scope.len(),
        present_but_out_of_scope,
        supported_generators: Vec::new(),
    }
}

fn collect_present_out_of_scope(
    root: &Path,
    target_candidates: Option<&Value>,
    specifier: Option<String>,
    consumer_file: Option<String>,
    matched_package: Option<String>,
    target_subpath: Option<String>,
    options: &GeneratedArtifactsOptions,
    present_keys: &mut BTreeSet<String>,
    present_but_out_of_scope: &mut Vec<PresentButOutOfScope>,
) {
    let Some(candidates) = target_candidates.and_then(Value::as_array) else {
        return;
    };
    for candidate in candidates {
        let Some(candidate) = candidate.as_str() else {
            continue;
        };
        let Some(candidate_path) = to_repo_relative(root, candidate) else {
            continue;
        };
        let abs_candidate = root.join(candidate_path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !abs_candidate.exists() {
            continue;
        }
        let scope = scan_scope_status_for_path(
            root,
            &abs_candidate,
            &ScanScopeOptions {
                include_tests: options.include_tests,
                exclude: options.excludes.clone(),
                ..ScanScopeOptions::default()
            },
        );
        if scope.included {
            continue;
        }
        let present_key = [
            specifier.as_deref().unwrap_or(""),
            consumer_file.as_deref().unwrap_or(""),
            candidate_path.as_str(),
            options.mode.as_str(),
        ]
        .join("|");
        if !present_keys.insert(present_key) {
            continue;
        }
        let (stale_status, stale_reason) =
            if matches!(options.mode, GeneratedArtifactsMode::Prepared) {
                (Some("unknown"), Some("generator-input-hash-not-recorded"))
            } else {
                (None, None)
            };
        present_but_out_of_scope.push(PresentButOutOfScope {
            specifier: specifier.clone(),
            consumer_file: consumer_file.clone(),
            matched_package: matched_package.clone(),
            target_subpath: target_subpath.clone(),
            candidate_path,
            reason: "present-but-out-of-scope",
            mode: options.mode,
            stale_status,
            stale_reason,
        });
    }
}

fn build_generated_consumer_blind_zone_summary(
    zones: &[Value],
) -> Vec<GeneratedConsumerBlindZoneSummary> {
    let mut groups: BTreeMap<String, BlindZoneGroup<'_>> = BTreeMap::new();
    for zone in zones {
        let Some(zone_object) = zone.as_object() else {
            continue;
        };
        let scope_package_root = object_string_field(Some(zone_object), "scopePackageRoot")
            .unwrap_or_else(|| "unknown".to_string());
        let group = groups
            .entry(scope_package_root.clone())
            .or_insert_with(|| BlindZoneGroup {
                scope_package_root,
                count: 0,
                statuses: BTreeMap::new(),
                specifiers: BTreeMap::new(),
                zones: Vec::new(),
            });
        group.count += 1;
        let status = object_string_field(Some(zone_object), "status")
            .unwrap_or_else(|| "unknown".to_string());
        *group.statuses.entry(status).or_insert(0) += 1;
        let specifier = object_string_field(Some(zone_object), "specifier")
            .unwrap_or_else(|| "unknown".to_string());
        *group.specifiers.entry(specifier).or_insert(0) += 1;
        group.zones.push(zone);
    }

    let mut summaries = groups
        .into_values()
        .map(|group| {
            let mut top_specifiers = group
                .specifiers
                .into_iter()
                .map(|(specifier, count)| GeneratedConsumerTopSpecifier { specifier, count })
                .collect::<Vec<_>>();
            top_specifiers.sort_by(|left, right| {
                right
                    .count
                    .cmp(&left.count)
                    .then_with(|| left.specifier.cmp(&right.specifier))
            });
            top_specifiers.truncate(5);

            GeneratedConsumerBlindZoneSummary {
                scope_package_root: group.scope_package_root,
                count: group.count,
                statuses: group.statuses,
                top_specifiers,
                examples: sorted_generated_consumer_zone_examples(&group.zones),
            }
        })
        .collect::<Vec<_>>();
    summaries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.scope_package_root.cmp(&right.scope_package_root))
    });
    summaries.truncate(20);
    summaries
}

fn sorted_generated_consumer_zone_examples(
    zones: &[&Value],
) -> Vec<GeneratedConsumerBlindZoneExample> {
    let mut zones = zones.to_vec();
    zones.sort_by(|left, right| {
        string_value(left.get("consumerFile"))
            .cmp(&string_value(right.get("consumerFile")))
            .then_with(|| {
                string_value(left.get("candidatePath"))
                    .cmp(&string_value(right.get("candidatePath")))
            })
            .then_with(|| {
                string_value(left.get("specifier")).cmp(&string_value(right.get("specifier")))
            })
    });
    zones.truncate(5);
    zones
        .into_iter()
        .map(|zone| GeneratedConsumerBlindZoneExample {
            specifier: string_field(zone.get("specifier")),
            consumer_file: string_field(zone.get("consumerFile")),
            candidate_path: string_field(zone.get("candidatePath")),
            status: string_field(zone.get("status")),
            scan_scope_reason: string_field(zone.get("scanScopeReason")),
            mode: string_field(zone.get("mode")),
        })
        .collect()
}

fn object_string_field(
    object: Option<&serde_json::Map<String, Value>>,
    field: &str,
) -> Option<String> {
    object
        .and_then(|object| object.get(field))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn string_field(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(ToOwned::to_owned)
}

fn string_value(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).unwrap_or("").to_string()
}

fn optional_string(value: Option<&String>) -> &str {
    value.map(String::as_str).unwrap_or("")
}
