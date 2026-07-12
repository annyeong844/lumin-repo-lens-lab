use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub const SARIF_REQUEST_SCHEMA_VERSION: &str = "lumin-sarif-producer-request.v1";

const TOOL_VERSION: &str = "0.0.0-lab.0";
const TOOL_INFO_URI: &str = "https://github.com/annyeong844/lumin-repo-lens-lab";
const HELP_URI: &str = "https://github.com/annyeong844/lumin-repo-lens-lab#readme";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRequest {
    pub schema_version: String,
    pub root: String,
    #[serde(default)]
    pub generated: Option<String>,
    #[serde(default)]
    pub fix_plan: Option<Value>,
    #[serde(default)]
    pub runtime_evidence: Option<Value>,
    #[serde(default)]
    pub staleness: Option<Value>,
    #[serde(default)]
    pub dead_classify: Option<Value>,
    #[serde(default)]
    pub symbols: Option<Value>,
    #[serde(default)]
    pub topology: Option<Value>,
    #[serde(default)]
    pub discipline: Option<Value>,
    #[serde(default)]
    pub barrels: Option<Value>,
}

#[derive(Debug, Default)]
struct SarifState {
    results: Vec<Value>,
    artifacts_used: Vec<&'static str>,
}

pub fn build_sarif_artifact(request: SarifRequest) -> Result<Value> {
    if request.schema_version != SARIF_REQUEST_SCHEMA_VERSION {
        bail!(
            "sarif-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if request.root.trim().is_empty() {
        bail!("sarif-artifact: root must be non-empty");
    }

    let root = slash_path(&request.root);
    let generated = request
        .generated
        .unwrap_or_else(|| "1970-01-01T00:00:00.000Z".to_string());
    let rules = sarif_rules();
    let mut state = SarifState::default();

    let fix_plan = present_artifact(request.fix_plan.as_ref());
    let runtime_evidence = present_artifact(request.runtime_evidence.as_ref());
    let staleness = present_artifact(request.staleness.as_ref());
    let dead_classify = present_artifact(request.dead_classify.as_ref());
    let symbols = present_artifact(request.symbols.as_ref());
    let topology = present_artifact(request.topology.as_ref());
    let discipline = present_artifact(request.discipline.as_ref());
    let barrels = present_artifact(request.barrels.as_ref());

    collect_dead_export_results(
        &mut state,
        &root,
        fix_plan,
        runtime_evidence,
        staleness,
        dead_classify,
        symbols,
    );
    collect_topology_results(&mut state, &root, topology);
    collect_discipline_results(&mut state, &root, discipline);
    collect_barrel_results(&mut state, &root, barrels);

    let mut by_level = BTreeMap::from([
        ("error".to_string(), 0usize),
        ("warning".to_string(), 0usize),
        ("note".to_string(), 0usize),
    ]);
    for result in &state.results {
        if let Some(level) = string_field(result, "level") {
            *by_level.entry(level).or_default() += 1;
        }
    }

    Ok(json!({
        "$schema": "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "lumin-repo-lens-lab",
                    "version": TOOL_VERSION,
                    "informationUri": TOOL_INFO_URI,
                    "shortDescription": {
                        "text": "AST-based repository structural audit with layered evidence (AST + runtime + git history)."
                    },
                    "rules": rules
                }
            },
            "invocations": [{
                "executionSuccessful": true,
                "startTimeUtc": generated,
                "endTimeUtc": generated,
                "workingDirectory": { "uri": format!("file://{}", root) }
            }],
            "originalUriBaseIds": {
                "SRCROOT": { "uri": format!("file://{}/", root) }
            },
            "results": state.results,
            "properties": {
                "artifactsUsed": state.artifacts_used,
                "scanRoot": request.root,
                "generatedAt": generated,
                "totalFindings": by_level.values().sum::<usize>(),
                "upstreamWarnings": upstream_warnings(symbols, dead_classify, topology, discipline)
            }
        }]
    }))
}

fn collect_dead_export_results(
    state: &mut SarifState,
    root: &str,
    fix_plan: Option<&Value>,
    runtime_evidence: Option<&Value>,
    staleness: Option<&Value>,
    dead_classify: Option<&Value>,
    symbols: Option<&Value>,
) {
    if let Some(fix_plan) = fix_plan {
        state.artifacts_used.push("fix-plan.json");
        emit_fix_plan_entries(state, root, fix_plan, "safeFixes", "SAFE_FIX");
        emit_fix_plan_entries(state, root, fix_plan, "reviewFixes", "REVIEW_FIX");
        emit_fix_plan_entries(state, root, fix_plan, "degraded", "DEGRADED");
        return;
    }

    if let Some(runtime_evidence) = runtime_evidence {
        if !array_field(runtime_evidence, "merged").is_empty() {
            state.artifacts_used.push("runtime-evidence.json");
            let staleness_by = staleness.and_then(staleness_lookup);
            if staleness_by.is_some() {
                state.artifacts_used.push("staleness.json");
            }
            for merged in array_field(runtime_evidence, "merged") {
                if string_field(merged, "grounding").as_deref() == Some("blind") {
                    continue;
                }
                let key = finding_key(merged);
                let stale = key.as_ref().and_then(|key| {
                    staleness_by
                        .as_ref()
                        .and_then(|lookup| lookup.get(key).copied())
                });
                let symbol = string_field(merged, "symbol").unwrap_or_default();
                let kind = string_field(merged, "kind").unwrap_or_default();
                let runtime_status =
                    string_field(merged, "runtimeStatus").unwrap_or_else(|| "not-measured".into());
                let grounding =
                    string_field(merged, "grounding").unwrap_or_else(|| "grounded".into());
                let confidence =
                    string_field(merged, "confidence").unwrap_or_else(|| "medium".into());
                let mut parts = vec![
                    format!("Dead export `{symbol}` ({kind})"),
                    format!("runtime: {runtime_status}"),
                ];
                if let Some(stale) = stale {
                    if let Some(tier) = string_field(stale, "stalenessTier") {
                        parts.push(format!("staleness: {tier}"));
                    }
                }
                parts.push(format!("grounding: {grounding}/{confidence}"));

                let mut properties = Map::new();
                insert_string(&mut properties, "symbol", symbol);
                insert_string(&mut properties, "kind", kind);
                insert_string(&mut properties, "grounding", grounding.clone());
                insert_string(&mut properties, "confidence", confidence.clone());
                insert_string(&mut properties, "runtimeStatus", runtime_status.clone());
                insert_value(
                    &mut properties,
                    "hitsInSymbol",
                    merged
                        .get("hitsInSymbol")
                        .cloned()
                        .unwrap_or_else(|| Value::from(0)),
                );
                if let Some(note) = merged.get("note").cloned() {
                    insert_value(&mut properties, "note", note);
                }
                if let Some(stale) = stale {
                    copy_field(stale, &mut properties, "stalenessTier");
                    copy_field(stale, &mut properties, "lineLastTouchedDaysAgo");
                    copy_field(stale, &mut properties, "symbolMentionStatus");
                }
                state.results.push(make_result(
                    "GA001",
                    level_for_dead(&grounding, &confidence, &runtime_status),
                    parts.join(" | "),
                    &string_field(merged, "file").unwrap_or_default(),
                    number_field(merged, "line"),
                    properties,
                    root,
                ));
            }
            return;
        }
    }

    if let Some(dead_classify) = dead_classify {
        state.artifacts_used.push("dead-classify.json");
        emit_dead_classify_entries(
            state,
            root,
            dead_classify,
            "proposal_C_remove_symbol",
            "warning",
            "C",
        );
        emit_dead_classify_entries(
            state,
            root,
            dead_classify,
            "proposal_A_demote_to_internal",
            "warning",
            "A",
        );
        emit_dead_classify_entries(state, root, dead_classify, "proposal_B_review", "note", "B");
        emit_dead_classify_entries(
            state,
            root,
            dead_classify,
            "proposal_remove_export_specifier",
            "note",
            "specifier",
        );
        return;
    }

    if let Some(symbols) = symbols {
        let dead = array_field(symbols, "deadProdList");
        if dead.is_empty() {
            return;
        }
        state.artifacts_used.push("symbols.json");
        for finding in dead {
            let symbol = string_field(finding, "symbol").unwrap_or_default();
            let kind = string_field(finding, "kind").unwrap_or_default();
            let mut properties = Map::new();
            insert_string(&mut properties, "symbol", symbol.clone());
            insert_string(&mut properties, "kind", kind.clone());
            insert_string(&mut properties, "grounding", "grounded");
            insert_string(&mut properties, "confidence", "low");
            insert_string(&mut properties, "runtimeStatus", "not-measured");
            state.results.push(make_result(
                "GA001",
                "note",
                format!(
                    "Dead export `{symbol}` ({kind}) — pre-policy static AST; run classify-dead-exports.mjs for policy-filtered verdict."
                ),
                &string_field(finding, "file").unwrap_or_default(),
                number_field(finding, "line"),
                properties,
                root,
            ));
        }
    }
}

fn emit_fix_plan_entries(
    state: &mut SarifState,
    root: &str,
    fix_plan: &Value,
    field: &str,
    tier: &str,
) {
    let Some(level) = tier_to_sarif_level(tier) else {
        return;
    };
    for scored in array_field(fix_plan, field) {
        let finding = scored.get("finding").unwrap_or(&Value::Null);
        let evidence = scored.get("evidence").unwrap_or(&Value::Null);
        let runtime = evidence.get("runtime").unwrap_or(&Value::Null);
        let stale = evidence.get("staleness").unwrap_or(&Value::Null);
        let symbol = string_field(finding, "symbol").unwrap_or_default();
        let kind = string_field(finding, "kind").unwrap_or_default();
        let reason = string_field(scored, "reason").unwrap_or_default();
        let mut parts = vec![
            format!("Dead export `{symbol}` ({kind})"),
            format!("tier: {tier}"),
        ];
        if let Some(status) = string_field(runtime, "status") {
            parts.push(format!("runtime: {status}"));
        }
        if let Some(tier) = string_field(stale, "tier") {
            parts.push(format!("staleness: {tier}"));
        }
        parts.push(format!("({reason})"));

        let mut properties = Map::new();
        insert_string(&mut properties, "symbol", symbol);
        insert_string(&mut properties, "kind", kind);
        insert_string(&mut properties, "tier", tier);
        insert_string(&mut properties, "reason", reason);
        insert_optional_string(
            &mut properties,
            "proposalBucket",
            string_field(finding, "bucket"),
        );
        insert_string(
            &mut properties,
            "grounding",
            string_field(runtime, "grounding").unwrap_or_else(|| "grounded".to_string()),
        );
        insert_string(
            &mut properties,
            "confidence",
            string_field(runtime, "confidence").unwrap_or_else(|| "medium".to_string()),
        );
        insert_string(
            &mut properties,
            "runtimeStatus",
            string_field(runtime, "status").unwrap_or_else(|| "not-measured".to_string()),
        );
        insert_value(
            &mut properties,
            "hitsInSymbol",
            runtime
                .get("hitsInSymbol")
                .cloned()
                .unwrap_or_else(|| Value::from(0)),
        );
        if let Some(staleness_tier) = string_field(stale, "tier") {
            insert_string(&mut properties, "stalenessTier", staleness_tier);
        }
        copy_field(stale, &mut properties, "lineLastTouchedDaysAgo");
        copy_field(finding, &mut properties, "fileInternalUses");
        copy_field(finding, &mut properties, "predicatePartner");
        copy_field(finding, &mut properties, "localName");

        state.results.push(make_result(
            "GA001",
            level,
            parts.join(" | "),
            &string_field(finding, "file").unwrap_or_default(),
            number_field(finding, "line"),
            properties,
            root,
        ));
    }
}

fn emit_dead_classify_entries(
    state: &mut SarifState,
    root: &str,
    dead_classify: &Value,
    field: &str,
    level: &str,
    action_field: &str,
) {
    for proposal in array_field(dead_classify, field) {
        let symbol = string_field(proposal, "symbol").unwrap_or_default();
        let kind = string_field(proposal, "kind").unwrap_or_default();
        let action = string_field(proposal, "action").unwrap_or_default();
        let mut properties = Map::new();
        insert_string(&mut properties, "symbol", symbol.clone());
        insert_string(&mut properties, "kind", kind.clone());
        insert_string(&mut properties, "grounding", "grounded");
        insert_string(&mut properties, "confidence", "medium");
        insert_string(&mut properties, "runtimeStatus", "not-measured");
        insert_string(&mut properties, "proposalBucket", action_field);
        copy_field(proposal, &mut properties, "localName");
        copy_field(proposal, &mut properties, "fileInternalUses");
        copy_field(proposal, &mut properties, "predicatePartner");
        state.results.push(make_result(
            "GA001",
            level,
            format!("Dead export `{symbol}` ({kind}) — {action}"),
            &string_field(proposal, "file").unwrap_or_default(),
            number_field(proposal, "line"),
            properties,
            root,
        ));
    }
}

fn collect_topology_results(state: &mut SarifState, root: &str, topology: Option<&Value>) {
    let Some(topology) = topology else {
        return;
    };
    state.artifacts_used.push("topology.json");
    for scc in array_field(topology, "sccs") {
        let members = string_array(scc.get("members"));
        let mut preview = members
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(" → ");
        if members.len() > 3 {
            preview.push_str(" → …");
        }
        for member in &members {
            let mut properties = Map::new();
            insert_value(
                &mut properties,
                "sccSize",
                scc.get("size").cloned().unwrap_or_else(|| Value::from(0)),
            );
            insert_value(&mut properties, "sccMembers", Value::from(members.clone()));
            state.results.push(make_result(
                "GA002",
                "warning",
                format!(
                    "File participates in SCC of size {}. Cycle preview: {preview}",
                    scc.get("size").and_then(Value::as_i64).unwrap_or(0)
                ),
                member,
                Some(1),
                properties,
                root,
            ));
        }
    }

    for largest in array_field(topology, "largestFiles") {
        let loc = number_field(largest, "loc").unwrap_or(0);
        if loc < 1000 {
            continue;
        }
        let mut properties = Map::new();
        insert_value(&mut properties, "loc", Value::from(loc));
        state.results.push(make_result(
            "GA004",
            "note",
            format!("File has {loc} LOC (threshold: 1000). Consider splitting."),
            &string_field(largest, "file").unwrap_or_default(),
            Some(1),
            properties,
            root,
        ));
    }

    for hotspot in array_field(topology, "crossSubmoduleTop")
        .into_iter()
        .take(5)
    {
        let count = number_field(hotspot, "count").unwrap_or(0);
        if count < 20 {
            continue;
        }
        let edge = string_field(hotspot, "edge").unwrap_or_default();
        let mut properties = Map::new();
        insert_string(&mut properties, "edge", edge.clone());
        insert_value(&mut properties, "importCount", Value::from(count));
        state.results.push(make_result(
            "GA005",
            "note",
            format!("Cross-submodule hotspot: {edge} ({count} imports)."),
            ".",
            Some(1),
            properties,
            root,
        ));
    }
}

fn collect_discipline_results(state: &mut SarifState, root: &str, discipline: Option<&Value>) {
    let Some(discipline) = discipline else {
        return;
    };
    let offenders = array_field(discipline, "overallTopOffenders");
    if offenders.is_empty() {
        return;
    }
    state.artifacts_used.push("discipline.json");
    for offender in offenders {
        let file = string_field(offender, "file").unwrap_or_default();
        let Some(breakdown) = offender.get("breakdown").and_then(Value::as_object) else {
            continue;
        };
        for (pattern, count) in breakdown {
            let count = count.as_i64().unwrap_or(0);
            if count == 0 {
                continue;
            }
            let mut properties = Map::new();
            insert_string(&mut properties, "pattern", pattern.clone());
            insert_value(&mut properties, "count", Value::from(count));
            state.results.push(make_result(
                "GA003",
                "note",
                format!("Discipline: {count}× `{pattern}` in this file."),
                &file,
                Some(1),
                properties,
                root,
            ));
        }
    }
}

fn collect_barrel_results(state: &mut SarifState, root: &str, barrels: Option<&Value>) {
    let Some(barrels) = barrels else {
        return;
    };
    let Some(by_package) = barrels.get("byPackage").and_then(Value::as_object) else {
        return;
    };
    state.artifacts_used.push("barrels.json");
    for (package, info) in by_package {
        for import in array_field(info, "sampleRootImporters") {
            if import
                .get("eslintDisable")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                continue;
            }
            let mut properties = Map::new();
            insert_string(&mut properties, "package", package.clone());
            insert_value(
                &mut properties,
                "symbols",
                import
                    .get("symbols")
                    .cloned()
                    .unwrap_or_else(|| Value::Array(Vec::new())),
            );
            insert_value(
                &mut properties,
                "reExport",
                Value::from(
                    import
                        .get("reExport")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                ),
            );
            state.results.push(make_result(
                "GA006",
                "warning",
                format!("Root-level barrel import of `{package}`. Prefer subpath export."),
                &string_field(import, "file").unwrap_or_default(),
                number_field(import, "line"),
                properties,
                root,
            ));
        }
    }
}

fn make_result(
    rule_id: &str,
    level: &str,
    message: String,
    file: &str,
    line: Option<i64>,
    properties: Map<String, Value>,
    root: &str,
) -> Value {
    let mut result = json!({
        "ruleId": rule_id,
        "ruleIndex": rule_index(rule_id),
        "level": level,
        "message": { "text": message },
        "locations": [{
            "physicalLocation": {
                "artifactLocation": { "uri": uri_for(root, file) },
                "region": { "startLine": line.unwrap_or(1).max(1) }
            }
        }]
    });
    if !properties.is_empty() {
        result["properties"] = Value::Object(properties);
    }
    result
}

fn sarif_rules() -> Value {
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

fn upstream_warnings(
    symbols: Option<&Value>,
    dead_classify: Option<&Value>,
    topology: Option<&Value>,
    discipline: Option<&Value>,
) -> Value {
    let mut warnings = Vec::new();
    append_warnings(&mut warnings, "symbols.json", symbols);
    append_warnings(&mut warnings, "dead-classify.json", dead_classify);
    append_warnings(&mut warnings, "topology.json", topology);
    append_warnings(&mut warnings, "discipline.json", discipline);
    Value::Array(warnings)
}

fn append_warnings(warnings: &mut Vec<Value>, source: &str, artifact: Option<&Value>) {
    let Some(meta_warnings) = artifact
        .and_then(|artifact| artifact.get("meta"))
        .and_then(|meta| meta.get("warnings"))
        .and_then(Value::as_array)
    else {
        return;
    };
    for warning in meta_warnings {
        let mut object = Map::new();
        insert_string(&mut object, "source", source);
        if let Some(warning) = warning.as_object() {
            for (key, value) in warning {
                object.insert(key.clone(), value.clone());
            }
        }
        warnings.push(Value::Object(object));
    }
}

fn staleness_lookup(staleness: &Value) -> Option<BTreeMap<String, &Value>> {
    let enriched = staleness.get("enriched")?.as_array()?;
    let mut lookup = BTreeMap::new();
    for entry in enriched {
        if let Some(key) = finding_key(entry) {
            lookup.insert(key, entry);
        }
    }
    Some(lookup)
}

fn finding_key(value: &Value) -> Option<String> {
    Some(format!(
        "{}|{}|{}",
        string_field(value, "file")?,
        string_field(value, "symbol")?,
        number_field(value, "line")?
    ))
}

fn present_artifact(value: Option<&Value>) -> Option<&Value> {
    value.filter(|value| !value.is_null())
}

fn level_for_dead(grounding: &str, confidence: &str, runtime_status: &str) -> &'static str {
    if runtime_status == "executed" {
        return "note";
    }
    if grounding == "grounded" && confidence == "high" {
        return "warning";
    }
    if grounding == "grounded" {
        return "warning";
    }
    "note"
}

fn tier_to_sarif_level(tier: &str) -> Option<&'static str> {
    match tier {
        "SAFE_FIX" => Some("warning"),
        "REVIEW_FIX" | "DEGRADED" => Some("note"),
        "MUTED" => None,
        _ => Some("note"),
    }
}

fn rule_index(rule_id: &str) -> usize {
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

fn uri_for(root: &str, file: &str) -> String {
    if file.is_empty() {
        return ".".to_string();
    }
    let file = slash_path(file);
    let abs = if path_is_absolute(&file) {
        file
    } else {
        format!("{}/{}", root.trim_end_matches('/'), file)
    };
    if abs == root {
        return ".".to_string();
    }
    if let Some(rest) = abs.strip_prefix(&format!("{}/", root.trim_end_matches('/'))) {
        return rest.to_string();
    }
    abs
}

fn path_is_absolute(path: &str) -> bool {
    path.starts_with('/') || path.as_bytes().get(1).copied() == Some(b':')
}

fn slash_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn array_field<'a>(value: &'a Value, field: &str) -> Vec<&'a Value> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|values| values.iter().collect())
        .unwrap_or_default()
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn number_field(value: &Value, field: &str) -> Option<i64> {
    value.get(field).and_then(Value::as_i64)
}

fn insert_string(object: &mut Map<String, Value>, key: &str, value: impl Into<String>) {
    object.insert(key.to_string(), Value::String(value.into()));
}

fn insert_optional_string(object: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(value) = value {
        insert_string(object, key, value);
    }
}

fn insert_value(object: &mut Map<String, Value>, key: &str, value: Value) {
    object.insert(key.to_string(), value);
}

fn copy_field(source: &Value, target: &mut Map<String, Value>, field: &str) {
    if let Some(value) = source.get(field).cloned() {
        target.insert(field.to_string(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fix_plan_tiers_drive_ga001_sarif_levels_and_skip_muted() -> Result<()> {
        let artifact = build_sarif_artifact(SarifRequest {
            schema_version: SARIF_REQUEST_SCHEMA_VERSION.to_string(),
            root: "C:/repo".to_string(),
            generated: Some("2026-07-04T00:00:00.000Z".to_string()),
            fix_plan: Some(json!({
                "safeFixes": [{
                    "finding": { "file": "src/safe.ts", "line": 10, "symbol": "SafeSym", "kind": "FunctionDeclaration", "bucket": "C" },
                    "evidence": { "runtime": { "status": "dead-confirmed", "grounding": "grounded", "confidence": "high", "hitsInSymbol": 0 } },
                    "reason": "runtime-dead-confirmed"
                }],
                "reviewFixes": [{
                    "finding": { "file": "src/review.ts", "line": 20, "symbol": "ReviewSym", "kind": "FunctionDeclaration", "bucket": "A", "fileInternalUses": 2 },
                    "evidence": {},
                    "reason": "manual-review"
                }],
                "degraded": [{
                    "finding": { "file": "src/deg.ts", "line": 30, "symbol": "DegSym", "kind": "FunctionDeclaration", "bucket": "C" },
                    "evidence": { "runtime": { "status": "executed", "grounding": "grounded", "confidence": "high", "hitsInSymbol": 7 } },
                    "reason": "runtime-executed"
                }],
                "muted": [{
                    "finding": { "file": "eslint.config.mjs", "line": 1, "symbol": "default", "kind": "default" },
                    "reason": "policy-excluded"
                }]
            })),
            runtime_evidence: None,
            staleness: None,
            dead_classify: None,
            symbols: None,
            topology: None,
            discipline: None,
            barrels: None,
        })?;

        let results = artifact["runs"][0]["results"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("SARIF results must be an array"))?;
        assert_eq!(results.len(), 3);
        assert_eq!(results[0]["level"], "warning");
        assert_eq!(results[1]["level"], "note");
        assert_eq!(results[2]["properties"]["hitsInSymbol"], 7);
        assert_eq!(
            artifact["runs"][0]["properties"]["artifactsUsed"],
            json!(["fix-plan.json"])
        );
        Ok(())
    }

    #[test]
    fn topology_discipline_and_barrel_artifacts_project_secondary_rules() -> Result<()> {
        let artifact = build_sarif_artifact(SarifRequest {
            schema_version: SARIF_REQUEST_SCHEMA_VERSION.to_string(),
            root: "C:/repo".to_string(),
            generated: Some("2026-07-04T00:00:00.000Z".to_string()),
            fix_plan: None,
            runtime_evidence: None,
            staleness: None,
            dead_classify: None,
            symbols: None,
            topology: Some(json!({
                "sccs": [{ "size": 2, "members": ["src/a.ts", "src/b.ts"] }],
                "largestFiles": [{ "file": "src/huge.ts", "loc": 1200 }],
                "crossSubmoduleTop": [{ "edge": "a -> b", "count": 30 }]
            })),
            discipline: Some(json!({
                "overallTopOffenders": [{ "file": "src/a.ts", "breakdown": { "as any": 2, "ignored": 0 } }]
            })),
            barrels: Some(json!({
                "byPackage": {
                    "@scope/pkg": {
                        "sampleRootImporters": [
                            { "file": "src/c.ts", "line": 4, "symbols": ["x"], "reExport": false }
                        ]
                    }
                }
            })),
        })?;
        let results = artifact["runs"][0]["results"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("SARIF results must be an array"))?;
        let rule_ids: Vec<_> = results
            .iter()
            .filter_map(|result| result["ruleId"].as_str())
            .collect();
        assert!(rule_ids.contains(&"GA002"));
        assert!(rule_ids.contains(&"GA003"));
        assert!(rule_ids.contains(&"GA004"));
        assert!(rule_ids.contains(&"GA005"));
        assert!(rule_ids.contains(&"GA006"));
        Ok(())
    }

    #[test]
    fn rejects_bad_request_schema() {
        let result = build_sarif_artifact(SarifRequest {
            schema_version: "wrong".to_string(),
            root: "C:/repo".to_string(),
            generated: None,
            fix_plan: None,
            runtime_evidence: None,
            staleness: None,
            dead_classify: None,
            symbols: None,
            topology: None,
            discipline: None,
            barrels: None,
        });
        assert!(result.is_err());
    }
}
