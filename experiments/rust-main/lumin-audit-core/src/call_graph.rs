use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap};

pub const CALL_GRAPH_REQUEST_SCHEMA_VERSION: &str = "lumin-call-graph-producer-request.v1";

const TOOL_NAME: &str = "build-call-graph.mjs";
const TOP_CALLEES_DISPLAY_SLICE: usize = 100;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallGraphRequest {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    pub file_count: usize,
    pub parse_errors: usize,
    #[serde(default)]
    pub parse_error_details: Vec<Value>,
    #[serde(default)]
    pub total_call_expressions: usize,
    #[serde(default)]
    pub total_direct_calls: usize,
    #[serde(default)]
    pub resolved_direct_calls: usize,
    #[serde(default)]
    pub type_only_resolved: usize,
    #[serde(default)]
    pub call_edges: Vec<CallEdge>,
    #[serde(default)]
    pub export_alias_map: BTreeMap<String, String>,
    #[serde(default)]
    pub bounded_out_member_calls_by_file: BTreeMap<String, usize>,
    #[serde(default)]
    pub member_calls_by_file: BTreeMap<String, usize>,
    #[serde(default)]
    pub semi_dead_list: Vec<Value>,
    #[serde(default)]
    pub semi_dead_react_filtered: usize,
    #[serde(default)]
    pub prototype_calls: Vec<PrototypeCall>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallEdge {
    pub from: String,
    pub to: String,
    pub callee: String,
    #[serde(default = "one")]
    pub count: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrototypeCall {
    pub owner: String,
    pub method: String,
}

#[derive(Debug)]
struct TopCallee {
    file: String,
    name: String,
    count: usize,
    first_seen: usize,
}

fn one() -> usize {
    1
}

pub fn build_call_graph_artifact(request: CallGraphRequest) -> Result<Value> {
    if request.schema_version != CALL_GRAPH_REQUEST_SCHEMA_VERSION {
        bail!(
            "call-graph-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let edge_records = request
        .call_edges
        .iter()
        .map(|edge| ResolvedEdge {
            from_module: module_of(&request.root, &edge.from),
            to_module: module_of(&request.root, &edge.to),
            target_identity: format!("{}::{}", rel_path(&request.root, &edge.to), edge.callee),
            count: edge.count,
        })
        .collect::<Vec<_>>();

    let mut call_fan_in_by_identity = request
        .export_alias_map
        .keys()
        .map(|identity| (identity.clone(), 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut call_fan_in_by_definition_id = request
        .export_alias_map
        .values()
        .map(|definition_id| (definition_id.clone(), 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut call_site_fan_in_by_definition_id = call_fan_in_by_definition_id.clone();

    for edge in &edge_records {
        *call_fan_in_by_identity
            .entry(edge.target_identity.clone())
            .or_insert(0) += edge.count;
        if let Some(definition_id) = request.export_alias_map.get(&edge.target_identity) {
            *call_fan_in_by_definition_id
                .entry(definition_id.clone())
                .or_insert(0) += edge.count;
            *call_site_fan_in_by_definition_id
                .entry(definition_id.clone())
                .or_insert(0) += edge.count;
        }
    }

    let top_callees = top_callees(&edge_records);
    let module_call_count = module_call_count(&edge_records);
    let prototype_owners = prototype_owners(&request.prototype_calls);
    let total_prototype_calls = request.prototype_calls.len();
    let bounded_out_member_calls: usize = request.bounded_out_member_calls_by_file.values().sum();
    let complete = request.parse_errors == 0;

    Ok(json!({
        "meta": {
            "generated": request.generated,
            "root": request.root,
            "tool": TOOL_NAME,
            "complete": complete,
            "parseErrors": request.parse_errors,
            "filesWithParseErrors": request.parse_error_details,
            "warnings": call_graph_warnings(&request.parse_error_details),
            "supports": {
                "callFanInByDefinitionId": true,
                "callFanInByIdentity": true,
                "callSiteFanInByDefinitionId": true,
                "exportAliasMap": true,
                "boundedMemberCallResolution": true,
                "topCalleesDisplaySlice": TOP_CALLEES_DISPLAY_SLICE,
                "truncationFix": true,
            },
        },
        "summary": {
            "files": request.file_count,
            "totalCallExpressions": request.total_call_expressions,
            "totalDirectCalls": request.total_direct_calls,
            "resolvedCrossFileCalls": request.resolved_direct_calls,
            "typeOnlySkipped": request.type_only_resolved,
            "callEdges": request.call_edges.len(),
            "boundedOutMemberCalls": bounded_out_member_calls,
            "semiDead": request.semi_dead_list.len(),
            "semiDeadReactFiltered": request.semi_dead_react_filtered,
            "totalPrototypeCalls": total_prototype_calls,
        },
        "topCallees": top_callees.iter().take(TOP_CALLEES_DISPLAY_SLICE).map(top_callee_json).collect::<Vec<_>>(),
        "callFanInByDefinitionId": call_fan_in_by_definition_id,
        "callFanInByIdentity": call_fan_in_by_identity,
        "callSiteFanInByDefinitionId": call_site_fan_in_by_definition_id,
        "exportAliasMap": request.export_alias_map,
        "boundedOutMemberCallsByFile": request.bounded_out_member_calls_by_file,
        "memberCallsByFile": request.member_calls_by_file,
        "moduleCallCount": module_call_count,
        "semiDeadList": request.semi_dead_list,
        "prototypeOwners": prototype_owners,
    }))
}

#[derive(Debug)]
struct ResolvedEdge {
    from_module: String,
    to_module: String,
    target_identity: String,
    count: usize,
}

fn call_graph_warnings(parse_error_details: &[Value]) -> Vec<Value> {
    if parse_error_details.is_empty() {
        return Vec::new();
    }
    vec![json!({
        "kind": "parse-errors",
        "code": "call-graph-parse-errors",
        "count": parse_error_details.len(),
        "message": format!("{} file(s) failed to parse; call graph is partial", parse_error_details.len()),
        "files": parse_error_details,
    })]
}

fn top_callees(edges: &[ResolvedEdge]) -> Vec<TopCallee> {
    let mut counts = HashMap::<String, (usize, usize)>::new();
    for (index, edge) in edges.iter().enumerate() {
        let entry = counts
            .entry(edge.target_identity.clone())
            .or_insert((0, index));
        entry.0 += edge.count;
    }
    let mut entries = counts
        .into_iter()
        .map(|(identity, (count, first_seen))| {
            let (file, name) = identity
                .split_once("::")
                .map(|(file, name)| (file.to_string(), name.to_string()))
                .unwrap_or_else(|| (identity, String::new()));
            TopCallee {
                file,
                name,
                count,
                first_seen,
            }
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.first_seen.cmp(&right.first_seen))
    });
    entries
}

fn top_callee_json(callee: &TopCallee) -> Value {
    json!({
        "file": callee.file,
        "name": callee.name,
        "count": callee.count,
    })
}

fn module_call_count(edges: &[ResolvedEdge]) -> Vec<Value> {
    let mut counts = HashMap::<String, (usize, usize)>::new();
    for (index, edge) in edges.iter().enumerate() {
        if edge.from_module == edge.to_module {
            continue;
        }
        let key = format!("{} → {}", edge.from_module, edge.to_module);
        let entry = counts.entry(key).or_insert((0, index));
        entry.0 += edge.count;
    }
    let mut entries = counts
        .into_iter()
        .map(|(edge, (count, first_seen))| (edge, count, first_seen))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.2.cmp(&right.2)));
    entries.truncate(50);
    entries
        .into_iter()
        .map(|(edge, count, _)| json!({ "edge": edge, "count": count }))
        .collect()
}

fn prototype_owners(calls: &[PrototypeCall]) -> Vec<Value> {
    let mut owner_index = HashMap::<String, usize>::new();
    let mut owners = Vec::<(String, Vec<(String, usize)>, usize)>::new();
    for call in calls {
        let index = if let Some(index) = owner_index.get(&call.owner) {
            *index
        } else {
            let index = owners.len();
            owner_index.insert(call.owner.clone(), index);
            owners.push((call.owner.clone(), Vec::new(), 0));
            index
        };
        let (_, methods, total) = &mut owners[index];
        *total += 1;
        if let Some((_, count)) = methods
            .iter_mut()
            .find(|(method, _)| method == &call.method)
        {
            *count += 1;
        } else {
            methods.push((call.method.clone(), 1));
        }
    }
    let mut projected = owners
        .into_iter()
        .enumerate()
        .map(|(first_seen, (owner, methods, total))| {
            let method_object = methods
                .into_iter()
                .map(|(method, count)| (method, json!(count)))
                .collect::<Map<_, _>>();
            (
                first_seen,
                total,
                json!({
                    "owner": owner,
                    "methods": method_object,
                    "total": total,
                }),
            )
        })
        .collect::<Vec<_>>();
    projected.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    projected.into_iter().map(|(_, _, value)| value).collect()
}

fn module_of(root: &str, file: &str) -> String {
    let rel = rel_path(root, file);
    let parts = rel.split('/').collect::<Vec<_>>();
    if parts.len() <= 3 {
        return parts.iter().take(2).copied().collect::<Vec<_>>().join("/");
    }
    parts[..parts.len() - 1].join("/")
}

fn rel_path(root: &str, file: &str) -> String {
    let root = normalize_slashes(root).trim_end_matches('/').to_string();
    let file = normalize_slashes(file);
    let root_prefix = format!("{root}/");
    if let Some(stripped) = file.strip_prefix(&root_prefix) {
        return stripped.to_string();
    }
    file
}

fn normalize_slashes(value: &str) -> String {
    value.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_call_graph_projection_from_js_facts() -> Result<()> {
        let artifact = build_call_graph_artifact(CallGraphRequest {
            schema_version: CALL_GRAPH_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            file_count: 2,
            parse_errors: 0,
            parse_error_details: vec![],
            total_call_expressions: 2,
            total_direct_calls: 1,
            resolved_direct_calls: 2,
            type_only_resolved: 0,
            call_edges: vec![CallEdge {
                from: "C:/repo/src/b.ts".to_string(),
                to: "C:/repo/src/a.ts".to_string(),
                callee: "alpha".to_string(),
                count: 2,
            }],
            export_alias_map: BTreeMap::from([(
                "src/a.ts::alpha".to_string(),
                "src/a.ts#FunctionDeclaration:7-37".to_string(),
            )]),
            bounded_out_member_calls_by_file: BTreeMap::from([
                ("src/a.ts".to_string(), 0),
                ("src/b.ts".to_string(), 0),
            ]),
            member_calls_by_file: BTreeMap::from([
                ("src/a.ts".to_string(), 0),
                ("src/b.ts".to_string(), 1),
            ]),
            semi_dead_list: vec![],
            semi_dead_react_filtered: 0,
            prototype_calls: vec![PrototypeCall {
                owner: "Thing".to_string(),
                method: "patch".to_string(),
            }],
        })?;

        assert_eq!(artifact["meta"]["tool"], TOOL_NAME);
        assert_eq!(artifact["summary"]["resolvedCrossFileCalls"], 2);
        assert_eq!(artifact["summary"]["totalPrototypeCalls"], 1);
        assert_eq!(artifact["topCallees"][0]["file"], "src/a.ts");
        assert_eq!(artifact["topCallees"][0]["count"], 2);
        assert_eq!(
            artifact["callFanInByDefinitionId"]["src/a.ts#FunctionDeclaration:7-37"],
            2
        );
        assert_eq!(artifact["prototypeOwners"][0]["owner"], "Thing");
        Ok(())
    }

    #[test]
    fn parse_errors_make_artifact_incomplete_with_warning() -> Result<()> {
        let artifact = build_call_graph_artifact(CallGraphRequest {
            schema_version: CALL_GRAPH_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            file_count: 1,
            parse_errors: 1,
            parse_error_details: vec![json!({"file": "bad.ts", "message": "bad syntax"})],
            total_call_expressions: 0,
            total_direct_calls: 0,
            resolved_direct_calls: 0,
            type_only_resolved: 0,
            call_edges: vec![],
            export_alias_map: BTreeMap::new(),
            bounded_out_member_calls_by_file: BTreeMap::new(),
            member_calls_by_file: BTreeMap::new(),
            semi_dead_list: vec![],
            semi_dead_react_filtered: 0,
            prototype_calls: vec![],
        })?;

        assert_eq!(artifact["meta"]["complete"], false);
        assert_eq!(
            artifact["meta"]["warnings"][0]["code"],
            "call-graph-parse-errors"
        );
        Ok(())
    }

    #[test]
    fn rejects_unknown_schema() {
        let error = match build_call_graph_artifact(CallGraphRequest {
            schema_version: "lumin-call-graph.future".to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            file_count: 0,
            parse_errors: 0,
            parse_error_details: vec![],
            total_call_expressions: 0,
            total_direct_calls: 0,
            resolved_direct_calls: 0,
            type_only_resolved: 0,
            call_edges: vec![],
            export_alias_map: BTreeMap::new(),
            bounded_out_member_calls_by_file: BTreeMap::new(),
            member_calls_by_file: BTreeMap::new(),
            semi_dead_list: vec![],
            semi_dead_react_filtered: 0,
            prototype_calls: vec![],
        }) {
            Ok(_) => panic!("schema should reject"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("unsupported schemaVersion"));
    }
}
