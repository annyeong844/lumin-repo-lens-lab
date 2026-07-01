use serde::Serialize;
use serde_json::{json, Map, Number, Value};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestCoreSummary {
    pub scan_range: ScanRangeSummary,
    pub confidence: ConfidenceSummary,
    pub sfc_evidence: Option<SfcEvidenceSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRangeSummary {
    pub root: String,
    pub include_tests: bool,
    pub production: bool,
    pub excludes: Vec<String>,
    pub auto_excludes: Vec<String>,
    pub languages: Value,
    pub files: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfidenceSummary {
    pub parse_errors: Value,
    pub unresolved_internal_ratio: Value,
    pub external_imports: Value,
    pub resolved_internal: Value,
    pub unresolved_internal: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcEvidenceSummary {
    pub artifact: &'static str,
    pub status: &'static str,
    pub script_import_consumer_count: Value,
    pub reachability_only_count: Value,
    pub review_only_evidence_count: Value,
    pub total_evidence_count: Value,
    pub by_lane: SfcEvidenceByLane,
    pub scan_gap_still_applies: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcEvidenceByLane {
    pub script_import_consumers: Value,
    pub script_src_reachability: Value,
    pub style_asset_references: Value,
    pub template_component_refs: Value,
    pub global_component_registrations: Value,
    pub generated_component_manifests: Value,
    pub framework_convention_components: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestCoreOptions {
    pub root: String,
    pub include_tests: bool,
    pub production: bool,
    pub excludes: Vec<String>,
    pub auto_excludes: Vec<String>,
}

pub fn summarize_manifest_core(
    options: ManifestCoreOptions,
    triage: Option<&Value>,
    symbols: Option<&Value>,
) -> ManifestCoreSummary {
    ManifestCoreSummary {
        scan_range: summarize_scan_range(&options, triage),
        confidence: summarize_confidence(symbols),
        sfc_evidence: summarize_sfc_evidence(symbols),
    }
}

fn summarize_scan_range(options: &ManifestCoreOptions, triage: Option<&Value>) -> ScanRangeSummary {
    ScanRangeSummary {
        root: options.root.clone(),
        include_tests: options.include_tests,
        production: options.production,
        excludes: options.excludes.clone(),
        auto_excludes: options.auto_excludes.clone(),
        languages: languages_from_triage(triage),
        files: triage_files(triage),
    }
}

fn summarize_confidence(symbols: Option<&Value>) -> ConfidenceSummary {
    ConfidenceSummary {
        parse_errors: parse_errors(symbols),
        unresolved_internal_ratio: nested_symbols_field(symbols, "uses", "unresolvedInternalRatio"),
        external_imports: nested_symbols_field(symbols, "uses", "external"),
        resolved_internal: nested_symbols_field(symbols, "uses", "resolvedInternal"),
        unresolved_internal: nested_symbols_field(symbols, "uses", "unresolvedInternal"),
    }
}

pub fn summarize_sfc_evidence(symbols: Option<&Value>) -> Option<SfcEvidenceSummary> {
    let uses = symbols
        .and_then(|symbols| symbols.get("uses"))
        .and_then(Value::as_object);
    let by_lane = SfcEvidenceByLane {
        script_import_consumers: number_or_zero(uses, "sfcScriptConsumers"),
        script_src_reachability: number_or_zero(uses, "sfcScriptSrcReachability"),
        style_asset_references: number_or_zero(uses, "sfcStyleAssetReferences"),
        template_component_refs: number_or_zero(uses, "sfcTemplateComponentRefs"),
        global_component_registrations: number_or_zero(uses, "sfcGlobalComponentRegistrations"),
        generated_component_manifests: number_or_zero(uses, "sfcGeneratedComponentManifests"),
        framework_convention_components: number_or_zero(uses, "sfcFrameworkConventionComponents"),
    };
    let total_evidence_count = number_sum([
        &by_lane.script_import_consumers,
        &by_lane.script_src_reachability,
        &by_lane.style_asset_references,
        &by_lane.template_component_refs,
        &by_lane.global_component_registrations,
        &by_lane.generated_component_manifests,
        &by_lane.framework_convention_components,
    ]);
    if total_evidence_count <= 0.0 {
        return None;
    }
    let review_only_evidence_count = number_sum([
        &by_lane.style_asset_references,
        &by_lane.template_component_refs,
        &by_lane.global_component_registrations,
        &by_lane.generated_component_manifests,
        &by_lane.framework_convention_components,
    ]);

    Some(SfcEvidenceSummary {
        artifact: "symbols.json",
        status: "complete",
        script_import_consumer_count: by_lane.script_import_consumers.clone(),
        reachability_only_count: by_lane.script_src_reachability.clone(),
        review_only_evidence_count: numeric_json(review_only_evidence_count),
        total_evidence_count: numeric_json(total_evidence_count),
        by_lane,
        scan_gap_still_applies: true,
    })
}

fn languages_from_triage(triage: Option<&Value>) -> Value {
    let Some(triage) = triage.and_then(Value::as_object) else {
        return Value::Null;
    };
    for field in ["byLanguage", "languages"] {
        if let Some(languages) = object_keys(triage.get(field)) {
            return json!(languages);
        }
    }
    if let Some(languages) = triage
        .get("summary")
        .and_then(Value::as_object)
        .and_then(|summary| object_keys(summary.get("byLanguage")))
    {
        return json!(languages);
    }

    let shape = triage.get("shape").and_then(Value::as_object);
    let mut languages = Vec::new();
    if positive_number(shape, "tsFiles") {
        languages.push("ts");
    }
    if positive_number(shape, "jsFiles") {
        languages.push("js");
    }
    if positive_number(shape, "pyFiles") {
        languages.push("py");
    }
    if positive_number(shape, "goFiles") {
        languages.push("go");
    }
    if positive_number(shape, "rustFiles") || positive_number(shape, "rsFiles") {
        languages.push("rs");
    }
    if languages.is_empty() {
        Value::Null
    } else {
        json!(languages)
    }
}

fn triage_files(triage: Option<&Value>) -> Value {
    let Some(triage) = triage.and_then(Value::as_object) else {
        return Value::Null;
    };
    triage
        .get("summary")
        .and_then(Value::as_object)
        .and_then(|summary| non_null_field(summary, "files"))
        .or_else(|| non_null_field(triage, "files"))
        .or_else(|| {
            triage
                .get("shape")
                .and_then(Value::as_object)
                .and_then(|shape| non_null_field(shape, "totalFiles"))
        })
        .cloned()
        .unwrap_or(Value::Null)
}

fn parse_errors(symbols: Option<&Value>) -> Value {
    let Some(symbols) = symbols.and_then(Value::as_object) else {
        return json!(0);
    };
    if let Some(count) = symbols
        .get("meta")
        .and_then(Value::as_object)
        .and_then(|meta| meta.get("warnings"))
        .and_then(Value::as_array)
        .and_then(|warnings| {
            warnings.iter().find_map(|warning| {
                let warning = warning.as_object()?;
                let is_parse_error = ["kind", "type", "code"].iter().any(|field| {
                    warning.get(*field).and_then(Value::as_str) == Some("parse-errors")
                });
                if is_parse_error {
                    warning.get("count").filter(|count| !count.is_null())
                } else {
                    None
                }
            })
        })
    {
        return count.clone();
    }
    symbols
        .get("filesWithParseErrors")
        .and_then(Value::as_array)
        .map(|files| json!(files.len()))
        .unwrap_or_else(|| json!(0))
}

fn nested_symbols_field(symbols: Option<&Value>, object_field: &str, value_field: &str) -> Value {
    symbols
        .and_then(|symbols| symbols.get(object_field))
        .and_then(Value::as_object)
        .and_then(|object| object.get(value_field))
        .cloned()
        .unwrap_or(Value::Null)
}

fn object_keys(value: Option<&Value>) -> Option<Vec<String>> {
    value.and_then(Value::as_object).map(|object| {
        object
            .keys()
            .map(ToOwned::to_owned)
            .collect::<Vec<String>>()
    })
}

fn positive_number(object: Option<&Map<String, Value>>, field: &str) -> bool {
    object
        .and_then(|object| object.get(field))
        .and_then(Value::as_f64)
        .is_some_and(|value| value > 0.0)
}

fn non_null_field<'a>(object: &'a Map<String, Value>, field: &str) -> Option<&'a Value> {
    object.get(field).filter(|value| !value.is_null())
}

fn number_or_zero(object: Option<&Map<String, Value>>, field: &str) -> Value {
    object
        .and_then(|object| object.get(field))
        .filter(|value| value.is_number())
        .cloned()
        .unwrap_or_else(|| json!(0))
}

fn number_sum<'a>(values: impl IntoIterator<Item = &'a Value>) -> f64 {
    values
        .into_iter()
        .filter_map(Value::as_f64)
        .filter(|value| value.is_finite())
        .sum()
}

fn numeric_json(value: f64) -> Value {
    if value.fract() == 0.0 {
        return json!(value as i64);
    }
    Number::from_f64(value)
        .map(Value::Number)
        .unwrap_or(Value::Null)
}
