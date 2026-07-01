use serde_json::{json, Value};
use std::collections::BTreeMap;

use super::policy::SHAPE_UNKNOWN_FILE_SHARE;
use super::value::{by_language, get_path, language_count, number_u64};
use super::{has_area_many, zone, BlindZoneSeverity, BlindZoneSummary};

const JS_FAMILY_LANGS: &[&str] = &["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"];
const SFC_FAMILY_LANGS: &[&str] = &["vue", "svelte", "astro"];

#[derive(Debug, Clone, Copy)]
pub(crate) struct LanguageSupportState<'a> {
    language_support: Option<&'a Value>,
    python_enabled: bool,
    go_enabled: bool,
}

pub(crate) fn language_support_state(symbols: Option<&Value>) -> LanguageSupportState<'_> {
    let language_support = symbols
        .and_then(|symbols| get_path(symbols, &["meta", "languageSupport"]))
        .filter(|value| value.is_object());
    LanguageSupportState {
        language_support,
        python_enabled: language_enabled_or_default(language_support, "python"),
        go_enabled: language_enabled_or_default(language_support, "go"),
    }
}

pub(crate) fn sfc_zone(triage: Option<&Value>) -> Option<BlindZoneSummary> {
    let (files, languages) = sfc_counts_from_triage(triage)?;
    Some(zone(
        "sfc-scan-gap",
        BlindZoneSeverity::ScanGap,
        "Vue/Svelte/Astro single-file components were counted by triage but are not included in the symbol graph; do not make repo-wide absence claims for SFC-owned imports, exports, or template reachability.",
        Some(json!({
            "files": files,
            "languages": languages,
            "reason": "sfc-extractor-not-registered",
        })),
    ))
}

pub(crate) fn detect_shape_zones(
    triage: Option<&Value>,
    support: LanguageSupportState<'_>,
    rust_analysis_complete: bool,
) -> Vec<BlindZoneSummary> {
    let shape = triage.and_then(|triage| triage.get("shape"));
    let Some(total_files) = shape
        .and_then(|shape| shape.get("totalFiles"))
        .and_then(number_u64)
    else {
        return Vec::new();
    };

    let mut zones = Vec::new();
    let known = shape_count(shape, "tsFiles")
        + shape_count(shape, "jsFiles")
        + shape_count(shape, "pyFiles")
        + shape_count(shape, "goFiles")
        + shape_count_opt(shape, "rustFiles")
            .or_else(|| shape_count_opt(shape, "rsFiles"))
            .unwrap_or(0)
        + shape_count(shape, "sfcFiles");
    let unknown = total_files.saturating_sub(known);
    if total_files > 0
        && unknown > 0
        && (unknown as f64 / total_files as f64) >= SHAPE_UNKNOWN_FILE_SHARE
    {
        zones.push(zone(
            "unclassified-files",
            BlindZoneSeverity::ScanGap,
            format!(
                "Do not make repo-wide absence claims; {unknown} file(s) are not in a language with a registered extractor (could be Rust, Kotlin, Swift, etc. — or non-source)."
            ),
            Some(json!({ "unknownFiles": unknown, "totalFiles": total_files })),
        ));
    }

    if let Some(files) = shape_count_opt(shape, "pyFiles").filter(|count| *count > 0) {
        zones.push(python_zone(files, support));
    }
    if let Some(files) = shape_count_opt(shape, "goFiles").filter(|count| *count > 0) {
        zones.push(go_zone(files, support));
    }
    let rust_files = shape_count_opt(shape, "rustFiles")
        .or_else(|| shape_count_opt(shape, "rsFiles"))
        .unwrap_or(0);
    if !rust_analysis_complete && rust_files > 0 {
        zones.push(rust_zone(rust_files));
    }
    zones
}

pub(crate) fn detect_by_language_zones(
    triage: Option<&Value>,
    support: LanguageSupportState<'_>,
    rust_analysis_complete: bool,
    existing_zones: &[BlindZoneSummary],
) -> Vec<BlindZoneSummary> {
    let Some(by_lang) = by_language(triage).and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut zones = Vec::new();
    for (lang, count_value) in by_lang {
        let Some(count) = language_count(Some(count_value)).filter(|count| *count > 0) else {
            continue;
        };
        if SFC_FAMILY_LANGS.contains(&lang.as_str()) {
            continue;
        }
        let lang_str = lang.as_str();
        if lang_str == "rs" {
            if !rust_analysis_complete && !has_area_many(existing_zones, &zones, &["rs"]) {
                zones.push(rust_zone(count));
            }
            continue;
        }
        if !is_supported_lang(lang_str)
            && !has_area_many(existing_zones, &zones, &["unclassified-files", lang_str])
        {
            zones.push(zone(
                lang,
                BlindZoneSeverity::ScanGap,
                format!(
                    "Do not make repo-wide absence claims; {count} {lang} file(s) not analyzed."
                ),
                Some(json!({ "files": count, "reason": "extractor-not-registered" })),
            ));
        }
        if lang_str == "py"
            && !has_area_many(
                existing_zones,
                &zones,
                &["python-method-resolution", "python-scan-gap"],
            )
        {
            zones.push(python_zone(count, support));
        }
        if lang_str == "go"
            && !has_area_many(
                existing_zones,
                &zones,
                &["go-method-resolution", "go-scan-gap"],
            )
        {
            zones.push(go_zone(count, support));
        }
    }
    zones
}

fn language_enabled_or_default(language_support: Option<&Value>, key: &str) -> bool {
    match language_support.and_then(|support| support.get(key)) {
        Some(value) => value.get("enabled").and_then(Value::as_bool) == Some(true),
        None => true,
    }
}

fn python_zone(files: u64, support: LanguageSupportState<'_>) -> BlindZoneSummary {
    if support.python_enabled {
        return zone(
            "python-method-resolution",
            BlindZoneSeverity::PrecisionGap,
            "Method-level dead-code claims are degraded. __getattr__ / lazy export maps not detected.",
            Some(json!({ "files": files })),
        );
    }
    zone(
        "python-scan-gap",
        BlindZoneSeverity::ScanGap,
        "Python files were counted by triage but were not included in the symbol graph; do not make Python absence claims.",
        Some(json!({
            "files": files,
            "reason": language_support_reason(support, "python", "python extractor unavailable"),
        })),
    )
}

fn go_zone(files: u64, support: LanguageSupportState<'_>) -> BlindZoneSummary {
    if support.go_enabled {
        return zone(
            "go-method-resolution",
            BlindZoneSeverity::PrecisionGap,
            "Method-level and interface-dispatch claims are degraded.",
            Some(json!({ "files": files })),
        );
    }
    zone(
        "go-scan-gap",
        BlindZoneSeverity::ScanGap,
        "Go files were counted by triage but were not included in the symbol graph; do not make Go absence claims.",
        Some(json!({
            "files": files,
            "reason": language_support_reason(support, "go", "tree-sitter unavailable"),
        })),
    )
}

fn language_support_reason(support: LanguageSupportState<'_>, key: &str, fallback: &str) -> String {
    support
        .language_support
        .and_then(|value| value.get(key))
        .and_then(|value| value.get("reason"))
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}

fn rust_zone(files: u64) -> BlindZoneSummary {
    zone(
        "rs",
        BlindZoneSeverity::ScanGap,
        "Rust files were counted by triage, but the JS/TS symbol graph does not own Rust absence claims; read the lumin-rust-analyzer artifact before making Rust findings.",
        Some(json!({
            "files": files,
            "reason": "rust-owned-analysis-not-registered-in-this-audit-run",
        })),
    )
}

fn sfc_counts_from_triage(triage: Option<&Value>) -> Option<(u64, BTreeMap<String, u64>)> {
    let mut languages = BTreeMap::new();
    if let Some(by_lang) = by_language(triage).and_then(Value::as_object) {
        for lang in SFC_FAMILY_LANGS {
            if let Some(count) = language_count(by_lang.get(*lang)).filter(|count| *count > 0) {
                languages.insert((*lang).to_string(), count);
            }
        }
    }
    let explicit_total = languages.values().sum::<u64>();
    let shape_total = triage
        .and_then(|triage| get_path(triage, &["shape", "sfcFiles"]))
        .and_then(number_u64)
        .unwrap_or(0);
    let total = explicit_total.max(shape_total);
    (total > 0).then_some((total, languages))
}

fn shape_count(shape: Option<&Value>, key: &str) -> u64 {
    shape_count_opt(shape, key).unwrap_or(0)
}

fn shape_count_opt(shape: Option<&Value>, key: &str) -> Option<u64> {
    shape.and_then(|shape| shape.get(key)).and_then(number_u64)
}

fn is_supported_lang(lang: &str) -> bool {
    JS_FAMILY_LANGS.contains(&lang) || lang == "py" || lang == "go"
}
