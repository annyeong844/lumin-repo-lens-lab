use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

pub const TOPOLOGY_REQUEST_SCHEMA_VERSION: &str = "lumin-topology-producer-request.v1";

const TOOL_NAME: &str = "m2s1-topology.mjs";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyRequest {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    pub mode: String,
    #[serde(default)]
    pub root_pkg_name: Option<Value>,
    #[serde(default)]
    pub include_type_edges: bool,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub source_entries: BTreeMap<String, TopologySourceEntry>,
    #[serde(default)]
    pub submodule_by_file: BTreeMap<String, String>,
    #[serde(default)]
    pub performance: Value,
    #[serde(default)]
    pub rust_metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologySourceEntry {
    pub loc: Option<usize>,
    #[serde(default)]
    pub edges: Vec<TopologySourceEdge>,
    #[serde(default)]
    pub external_count: usize,
    #[serde(default)]
    pub unresolved_count: usize,
    #[serde(default)]
    pub parse_error: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologySourceEdge {
    pub to: String,
    #[serde(default)]
    pub type_only: bool,
}

#[derive(Debug, Clone)]
struct NodeInfo {
    file: String,
    loc: usize,
}

#[derive(Debug, Clone)]
struct InternalEdge {
    from: String,
    to: String,
    type_only: bool,
}

pub fn build_topology_artifact(request: TopologyRequest) -> Result<Value> {
    if request.schema_version != TOPOLOGY_REQUEST_SCHEMA_VERSION {
        bail!(
            "topology-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let mut nodes = BTreeMap::<String, NodeInfo>::new();
    let mut node_order = Vec::<String>::new();
    let mut edges = Vec::<InternalEdge>::new();
    let mut total_loc = 0usize;
    let mut parse_errors = 0usize;
    let mut external_edges = 0usize;
    let mut unresolved_edges = 0usize;

    for file in &request.files {
        let Some(entry) = request.source_entries.get(file) else {
            parse_errors += 1;
            continue;
        };
        let Some(loc) = entry.loc else {
            parse_errors += 1;
            continue;
        };
        nodes.insert(
            file.clone(),
            NodeInfo {
                file: file.clone(),
                loc,
            },
        );
        node_order.push(file.clone());
        total_loc += loc;
        external_edges += entry.external_count;
        unresolved_edges += entry.unresolved_count;
        if entry.parse_error {
            parse_errors += 1;
        }
        for edge in &entry.edges {
            edges.push(InternalEdge {
                from: file.clone(),
                to: edge.to.clone(),
                type_only: edge.type_only,
            });
        }
    }

    let fan_in = fan_counts(edges.iter().map(|edge| edge.to.as_str()));
    let fan_out = fan_counts(edges.iter().map(|edge| edge.from.as_str()));
    let sccs = strongly_connected_components(&node_order, &edges, request.include_type_edges);
    let cross_submodule_edges = cross_submodule_edges(&request, &edges);
    let largest_files = largest_files(&request.root, nodes.values());

    let mut meta = Map::new();
    meta.insert("tool".to_string(), json!(TOOL_NAME));
    meta.insert("generated".to_string(), json!(request.generated));
    meta.insert("root".to_string(), json!(request.root));
    meta.insert("mode".to_string(), json!(request.mode));
    if let Some(root_pkg_name) = request.root_pkg_name {
        meta.insert("rootPkgName".to_string(), root_pkg_name);
    }
    for (key, value) in request.rust_metadata {
        meta.insert(key, value);
    }
    meta.insert("complete".to_string(), json!(true));

    Ok(json!({
        "meta": Value::Object(meta),
        "summary": {
            "files": request.files.len(),
            "totalLoc": total_loc,
            "meanLocPerFile": mean_loc(total_loc, request.files.len()),
            "parseErrors": parse_errors,
            "internalEdges": edges.len(),
            "externalEdges": external_edges,
            "unresolvedEdges": unresolved_edges,
            "lens": if request.include_type_edges { "static" } else { "runtime" },
            "sccCount": sccs.len(),
            "maxSccSize": sccs.iter().map(Vec::len).max().unwrap_or(0),
            "typeOnlyEdges": edges.iter().filter(|edge| edge.type_only).count(),
            "bigFiles": largest_files.len(),
            "oneThousandPlusFiles": largest_files.iter()
                .filter(|file| file.get("loc").and_then(Value::as_u64).unwrap_or(0) >= 1000)
                .count(),
            "performance": request.performance,
        },
        "nodes": nodes_object(&request.root, nodes.values()),
        "edges": edges_array(&request.root, &edges),
        "topFanIn": top_fan(&request.root, fan_in),
        "topFanOut": top_fan(&request.root, fan_out),
        "sccs": sccs_array(&request.root, &sccs),
        "crossSubmoduleEdges": cross_submodule_edges,
        "crossSubmoduleTop": cross_submodule_top(&cross_submodule_edges),
        "largestFiles": largest_files,
    }))
}

fn mean_loc(total_loc: usize, files: usize) -> usize {
    ((total_loc as f64) / (files.max(1) as f64)).round() as usize
}

fn normalize_slashes(value: &str) -> String {
    value.replace('\\', "/")
}

fn rel_path(root: &str, file: &str) -> String {
    if file.starts_with("external:") || file.starts_with("unresolved:") {
        return file.to_string();
    }
    let root = normalize_slashes(root).trim_end_matches('/').to_string();
    let file = normalize_slashes(file);
    let prefix = format!("{root}/");
    if file == root {
        ".".to_string()
    } else if file.starts_with(&prefix) {
        file[prefix.len()..].to_string()
    } else {
        file
    }
}

fn nodes_object<'a>(root: &str, nodes: impl Iterator<Item = &'a NodeInfo>) -> Value {
    let mut out = Map::new();
    for node in nodes {
        out.insert(rel_path(root, &node.file), json!({ "loc": node.loc }));
    }
    Value::Object(out)
}

fn edges_array(root: &str, edges: &[InternalEdge]) -> Vec<Value> {
    edges
        .iter()
        .map(|edge| {
            json!({
                "from": rel_path(root, &edge.from),
                "to": rel_path(root, &edge.to),
                "typeOnly": edge.type_only,
            })
        })
        .collect()
}

fn fan_counts<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
    counts
}

fn top_fan(root: &str, counts: BTreeMap<String, usize>) -> Vec<Value> {
    let mut entries = counts.into_iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    entries
        .into_iter()
        .take(15)
        .map(|(file, count)| json!({ "file": rel_path(root, &file), "count": count }))
        .collect()
}

fn largest_files<'a>(root: &str, nodes: impl Iterator<Item = &'a NodeInfo>) -> Vec<Value> {
    let mut files = nodes
        .map(|node| (node.file.clone(), node.loc))
        .filter(|(_, loc)| *loc >= 400)
        .collect::<Vec<_>>();
    files.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    files
        .into_iter()
        .take(20)
        .map(|(file, loc)| json!({ "file": rel_path(root, &file), "loc": loc }))
        .collect()
}

fn strongly_connected_components(
    node_order: &[String],
    edges: &[InternalEdge],
    include_type_edges: bool,
) -> Vec<Vec<String>> {
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    for edge in edges {
        if edge.type_only && !include_type_edges {
            continue;
        }
        adjacency
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
    }

    let mut state = TarjanState {
        index: 0,
        indices: BTreeMap::new(),
        lows: BTreeMap::new(),
        on_stack: BTreeSet::new(),
        stack: Vec::new(),
        components: Vec::new(),
        adjacency,
    };

    for node in node_order {
        if !state.indices.contains_key(node) {
            state.visit(node.clone());
        }
    }

    let mut components = state.components;
    components.sort_by_key(|component| Reverse(component.len()));
    components.into_iter().take(10).collect()
}

struct TarjanState {
    index: usize,
    indices: BTreeMap<String, usize>,
    lows: BTreeMap<String, usize>,
    on_stack: BTreeSet<String>,
    stack: Vec<String>,
    components: Vec<Vec<String>>,
    adjacency: BTreeMap<String, Vec<String>>,
}

impl TarjanState {
    fn visit(&mut self, node: String) {
        let current_index = self.index;
        self.indices.insert(node.clone(), current_index);
        self.lows.insert(node.clone(), current_index);
        self.index += 1;
        self.stack.push(node.clone());
        self.on_stack.insert(node.clone());

        let targets = self.adjacency.get(&node).cloned().unwrap_or_default();
        for target in targets {
            if !self.indices.contains_key(&target) {
                self.visit(target.clone());
                let low = self.lows[&node].min(self.lows[&target]);
                self.lows.insert(node.clone(), low);
            } else if self.on_stack.contains(&target) {
                let low = self.lows[&node].min(self.indices[&target]);
                self.lows.insert(node.clone(), low);
            }
        }

        if self.lows[&node] == self.indices[&node] {
            let mut component = Vec::new();
            while let Some(top) = self.stack.pop() {
                self.on_stack.remove(&top);
                let done = top == node;
                component.push(top);
                if done {
                    break;
                }
            }
            if component.len() > 1 {
                self.components.push(component);
            }
        }
    }
}

fn sccs_array(root: &str, sccs: &[Vec<String>]) -> Vec<Value> {
    sccs.iter()
        .map(|members| {
            json!({
                "size": members.len(),
                "members": members.iter().map(|file| rel_path(root, file)).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn submodule_for(request: &TopologyRequest, file: &str) -> String {
    request
        .submodule_by_file
        .get(file)
        .or_else(|| {
            request
                .submodule_by_file
                .get(&rel_path(&request.root, file))
        })
        .cloned()
        .unwrap_or_else(|| "root".to_string())
}

fn cross_submodule_edges(request: &TopologyRequest, edges: &[InternalEdge]) -> Vec<Value> {
    let mut counts = BTreeMap::<(String, String), usize>::new();
    for edge in edges {
        let from = submodule_for(request, &edge.from);
        let to = submodule_for(request, &edge.to);
        if from == to {
            continue;
        }
        *counts.entry((from, to)).or_insert(0) += 1;
    }
    let mut rows = counts.into_iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.0 .0.cmp(&right.0 .0))
            .then_with(|| left.0 .1.cmp(&right.0 .1))
    });
    rows.into_iter()
        .map(|((from, to), count)| json!({ "from": from, "to": to, "count": count }))
        .collect()
}

fn cross_submodule_top(edges: &[Value]) -> Vec<Value> {
    edges
        .iter()
        .take(30)
        .map(|edge| {
            let from = edge.get("from").and_then(Value::as_str).unwrap_or("");
            let to = edge.get("to").and_then(Value::as_str).unwrap_or("");
            let count = edge.get("count").and_then(Value::as_u64).unwrap_or(0);
            json!({ "edge": format!("{from} → {to}"), "count": count })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    fn request() -> Result<TopologyRequest> {
        Ok(serde_json::from_value(json!({
            "schemaVersion": TOPOLOGY_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-04T00:00:00.000Z",
            "root": "C:/repo",
            "mode": "single-package",
            "rootPkgName": "demo",
            "includeTypeEdges": false,
            "files": [
                "C:/repo/src/index.ts",
                "C:/repo/src/runtime.ts",
                "C:/repo/src/types.ts",
                "C:/repo/src/a.ts",
                "C:/repo/src/b.ts"
            ],
            "sourceEntries": {
                "C:/repo/src/index.ts": {
                    "loc": 10,
                    "edges": [
                        { "to": "C:/repo/src/runtime.ts" },
                        { "to": "C:/repo/src/types.ts", "typeOnly": true }
                    ],
                    "externalCount": 1,
                    "unresolvedCount": 2
                },
                "C:/repo/src/runtime.ts": { "loc": 20, "edges": [] },
                "C:/repo/src/types.ts": { "loc": 30, "edges": [] },
                "C:/repo/src/a.ts": { "loc": 40, "edges": [{ "to": "C:/repo/src/b.ts" }] },
                "C:/repo/src/b.ts": { "loc": 50, "edges": [{ "to": "C:/repo/src/a.ts" }] }
            },
            "submoduleByFile": {
                "C:/repo/src/index.ts": "root",
                "C:/repo/src/runtime.ts": "root",
                "C:/repo/src/types.ts": "types",
                "C:/repo/src/a.ts": "cycle",
                "C:/repo/src/b.ts": "cycle"
            },
            "performance": { "filesCollected": 5, "scannerPolicyVersion": "test" },
            "rustMetadata": { "rustTopologyScanner": { "mode": "off" } }
        }))?)
    }

    #[test]
    fn builds_topology_artifact_from_precomputed_entries() -> Result<()> {
        let artifact = build_topology_artifact(request()?)?;

        assert_eq!(artifact["meta"]["tool"], "m2s1-topology.mjs");
        assert_eq!(artifact["meta"]["schemaVersion"], Value::Null);
        assert_eq!(artifact["meta"]["complete"], true);
        assert_eq!(artifact["summary"]["files"], 5);
        assert_eq!(artifact["summary"]["internalEdges"], 4);
        assert_eq!(artifact["summary"]["externalEdges"], 1);
        assert_eq!(artifact["summary"]["unresolvedEdges"], 2);
        assert_eq!(artifact["summary"]["typeOnlyEdges"], 1);
        assert_eq!(artifact["summary"]["sccCount"], 1);
        assert_eq!(artifact["summary"]["maxSccSize"], 2);
        assert_eq!(artifact["nodes"]["src/index.ts"]["loc"], 10);
        assert!(artifact["edges"]
            .as_array()
            .context("edges should be array")?
            .iter()
            .any(|edge| edge["to"] == "src/runtime.ts"));
        assert_eq!(artifact["sccs"][0]["size"], 2);
        Ok(())
    }

    #[test]
    fn static_lens_includes_type_only_cycles() -> Result<()> {
        let mut request = request()?;
        request.include_type_edges = true;
        request
            .source_entries
            .get_mut("C:/repo/src/types.ts")
            .context("types entry")?
            .edges
            .push(TopologySourceEdge {
                to: "C:/repo/src/index.ts".to_string(),
                type_only: true,
            });

        let artifact = build_topology_artifact(request)?;
        assert_eq!(artifact["summary"]["lens"], "static");
        assert_eq!(artifact["summary"]["sccCount"], 2);
        Ok(())
    }

    #[test]
    fn rejects_unknown_schema() -> Result<()> {
        let mut request = request()?;
        request.schema_version = "wrong".to_string();
        let err = build_topology_artifact(request)
            .err()
            .context("schema should fail")?;
        assert!(err.to_string().contains("unsupported schemaVersion"));
        Ok(())
    }
}
