use super::findings::{finding_identity, normalize_path_text};
use super::protocol::RankFixesRequest;
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub(super) fn with_evidence_support(mut finding: Value, request: &RankFixesRequest) -> Value {
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
