use serde_json::{json, Value};

const TOOL_INFO_URI: &str = "https://github.com/annyeong844/lumin-repo-lens-lab";
const HELP_URI: &str = "https://github.com/annyeong844/lumin-repo-lens-lab#readme";

pub(super) fn tool_info_uri() -> &'static str {
    TOOL_INFO_URI
}

pub(super) fn sarif_rules() -> Value {
    json!([
        {
            "id": "GA001",
            "name": "dead-export",
            "shortDescription": { "text": "Exported symbol has no consumers." },
            "fullDescription": {
                "text": "Symbol is exported but no import or re-export references it across the scanned file set. Confidence is upgraded when fused with runtime coverage (merge-runtime-evidence) and git staleness (measure-staleness)."
            },
            "defaultConfiguration": { "level": "warning" },
            "helpUri": HELP_URI
        },
        {
            "id": "GA002",
            "name": "cyclic-dependency",
            "shortDescription": { "text": "File participates in an import cycle." },
            "fullDescription": {
                "text": "File-level strongly-connected component detected via Tarjan SCC on non-type-only import edges."
            },
            "defaultConfiguration": { "level": "warning" },
            "helpUri": HELP_URI
        },
        {
            "id": "GA003",
            "name": "escape-hatch",
            "shortDescription": { "text": "Type-safety or discipline escape hatch." },
            "fullDescription": {
                "text": "Use of `: any`, `as any`, `@ts-ignore`, `@ts-nocheck`, `eslint-disable`, `new Function(...)`, or similar mechanisms that bypass static checks."
            },
            "defaultConfiguration": { "level": "note" },
            "helpUri": HELP_URI
        },
        {
            "id": "GA004",
            "name": "god-module",
            "shortDescription": { "text": "File exceeds size threshold." },
            "fullDescription": {
                "text": "File has 1000+ lines of code — candidate for splitting into smaller modules."
            },
            "defaultConfiguration": { "level": "note" },
            "helpUri": HELP_URI
        },
        {
            "id": "GA005",
            "name": "cross-submodule-hotspot",
            "shortDescription": { "text": "Heavy cross-submodule coupling." },
            "fullDescription": {
                "text": "High count of imports crossing top-level submodule boundaries — potential architectural layering violation."
            },
            "defaultConfiguration": { "level": "note" },
            "helpUri": HELP_URI
        },
        {
            "id": "GA006",
            "name": "barrel-discipline",
            "shortDescription": { "text": "Import bypasses the package barrel." },
            "fullDescription": {
                "text": "Root-level (non-subpath) import of a workspace package — consumer should use the public subpath export instead of pulling through the barrel."
            },
            "defaultConfiguration": { "level": "warning" },
            "helpUri": HELP_URI
        }
    ])
}

pub(super) fn rule_index(rule_id: &str) -> usize {
    match rule_id {
        "GA001" => 0,
        "GA002" => 1,
        "GA003" => 2,
        "GA004" => 3,
        "GA005" => 4,
        "GA006" => 5,
        _ => 0,
    }
}
