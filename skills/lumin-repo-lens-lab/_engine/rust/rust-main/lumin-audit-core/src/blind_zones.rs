mod language;
mod policy;
mod precision;
mod resolver;
mod value;

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy)]
pub struct BlindZoneInput<'a> {
    pub triage: Option<&'a Value>,
    pub symbols: Option<&'a Value>,
    pub dead_classify: Option<&'a Value>,
    pub entry_surface: Option<&'a Value>,
    pub resolver_diagnostics: Option<&'a Value>,
    pub rust_analysis: Option<&'a Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlindZoneSeverity {
    ScanGap,
    PrecisionGap,
    ConfidenceGap,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlindZoneSummary {
    pub area: String,
    pub severity: BlindZoneSeverity,
    pub effect: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

pub fn summarize_blind_zones(input: BlindZoneInput<'_>) -> Vec<BlindZoneSummary> {
    let _dead_classify = input.dead_classify;
    let support = language::language_support_state(input.symbols);
    let rust_analysis_complete = rust_analysis_complete(input.rust_analysis);
    let mut zones = language::detect_shape_zones(input.triage, support, rust_analysis_complete);

    if let Some(sfc) = language::sfc_zone(input.triage) {
        if !has_area(&zones, &["sfc-scan-gap"]) {
            zones.push(sfc);
        }
    }
    zones.extend(language::detect_by_language_zones(
        input.triage,
        support,
        rust_analysis_complete,
        &zones,
    ));
    if let Some(zone) = resolver::detect_resolver_zone(input.symbols, input.resolver_diagnostics) {
        zones.push(zone);
    }
    if let Some(zone) = precision::detect_parser_zone(input.symbols) {
        zones.push(zone);
    }
    if let Some(zone) = precision::detect_cjs_export_surface_zone(input.symbols) {
        zones.push(zone);
    }
    if let Some(zone) = precision::detect_cjs_require_opacity_zone(input.symbols) {
        zones.push(zone);
    }
    if let Some(zone) = precision::detect_html_entry_surface_zone(input.entry_surface) {
        zones.push(zone);
    }
    zones
}

pub(crate) fn zone(
    area: impl Into<String>,
    severity: BlindZoneSeverity,
    effect: impl Into<String>,
    details: Option<Value>,
) -> BlindZoneSummary {
    BlindZoneSummary {
        area: area.into(),
        severity,
        effect: effect.into(),
        details,
    }
}

pub(crate) fn has_area(zones: &[BlindZoneSummary], areas: &[&str]) -> bool {
    zones.iter().any(|zone| areas.contains(&zone.area.as_str()))
}

pub(crate) fn has_area_many(
    left: &[BlindZoneSummary],
    right: &[BlindZoneSummary],
    areas: &[&str],
) -> bool {
    has_area(left, areas) || has_area(right, areas)
}

fn rust_analysis_complete(rust_analysis: Option<&Value>) -> bool {
    rust_analysis
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        == Some("complete")
        && rust_analysis
            .and_then(|value| value.get("available"))
            .and_then(Value::as_bool)
            == Some(true)
}
