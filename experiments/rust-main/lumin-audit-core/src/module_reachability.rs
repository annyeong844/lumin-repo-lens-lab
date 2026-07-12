use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const MODULE_REACHABILITY_SCHEMA_VERSION: &str = "module-reachability.v1";
pub const MODULE_REACHABILITY_REQUEST_SCHEMA_VERSION: &str =
    "lumin-module-reachability-producer-request.v1";
pub const DEFAULT_MAX_FILES_VISITED: usize = 200_000;
pub const DEFAULT_MAX_EDGES_VISITED: usize = 400_000;

const TOOL_NAME: &str = "build-module-reachability.mjs";
const ENTRY_SURFACE_FILE: &str = "entry-surface.json";
const MODE_FULL_BFS: &str = "full-bfs";
const SCC_NOTE: &str =
    "Files import each other, but none are reachable from the recorded entry surface.";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleReachabilityRequest {
    pub schema_version: String,
    pub root: String,
    #[serde(default)]
    pub generated: Option<String>,
    pub symbols: SymbolsInput,
    pub entry_surface: EntrySurfaceInput,
    #[serde(default = "default_max_files_visited")]
    pub max_files_visited: usize,
    #[serde(default = "default_max_edges_visited")]
    pub max_edges_visited: usize,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolsInput {
    #[serde(default)]
    pub def_index: BTreeMap<String, Value>,
    #[serde(default)]
    pub re_exports_by_file: BTreeMap<String, Value>,
    #[serde(default)]
    pub resolved_internal_edges: Vec<ResolvedInternalEdge>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedInternalEdge {
    #[serde(default)]
    pub from: Value,
    #[serde(default)]
    pub to: Value,
    #[serde(default)]
    pub type_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrySurfaceInput {
    #[serde(default)]
    pub entry_files: Vec<String>,
    #[serde(default = "default_global_completeness")]
    pub global_completeness: String,
    #[serde(default)]
    pub completeness_by_submodule: BTreeMap<String, Value>,
}

impl Default for EntrySurfaceInput {
    fn default() -> Self {
        Self {
            entry_files: Vec::new(),
            global_completeness: default_global_completeness(),
            completeness_by_submodule: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleReachabilityArtifact {
    pub meta: ModuleReachabilityMeta,
    pub runtime_reachable_files: Vec<String>,
    pub type_reachable_files: Vec<String>,
    pub reachable_files: Vec<String>,
    pub bounded_out_files: Vec<String>,
    pub unreachable_files: Vec<String>,
    pub unreachable_strongly_connected_components: Vec<UnreachableStronglyConnectedComponent>,
    pub summary: ModuleReachabilitySummary,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleReachabilityMeta {
    pub tool: &'static str,
    pub generated: String,
    pub root: String,
    pub schema_version: &'static str,
    pub mode: &'static str,
    pub entry_surface_file: &'static str,
    pub global_completeness: String,
    pub completeness_by_submodule: BTreeMap<String, Value>,
    pub max_files_visited: usize,
    pub max_edges_visited: usize,
    pub bounded_out_reason: Option<String>,
    pub supports: ModuleReachabilitySupports,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleReachabilitySupports {
    pub runtime_reachable_files: bool,
    pub type_reachable_files: bool,
    pub bounded_out_files: bool,
    pub unreachable_strongly_connected_components: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnreachableStronglyConnectedComponent {
    pub kind: &'static str,
    pub graph: &'static str,
    pub size: usize,
    pub files: Vec<String>,
    pub note: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleReachabilitySummary {
    pub runtime_reachable: usize,
    pub type_reachable: usize,
    pub reachable: usize,
    pub bounded_out: usize,
    pub unreachable: usize,
    pub unreachable_strongly_connected_components: usize,
    pub unreachable_strongly_connected_files: usize,
    pub known_files: usize,
}

#[derive(Debug)]
struct BfsResult {
    visited: BTreeSet<String>,
    bounded_out_reason: Option<String>,
}

pub fn build_module_reachability_artifact(
    request: ModuleReachabilityRequest,
) -> Result<ModuleReachabilityArtifact> {
    if request.schema_version != MODULE_REACHABILITY_REQUEST_SCHEMA_VERSION {
        bail!(
            "module-reachability-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if request.max_files_visited == 0 {
        bail!("module-reachability-artifact: maxFilesVisited must be a positive integer");
    }
    if request.max_edges_visited == 0 {
        bail!("module-reachability-artifact: maxEdgesVisited must be a positive integer");
    }

    let known_files = collect_known_files(&request.symbols, &request.entry_surface);
    let entry_files = ordered_entry_files(&request.entry_surface.entry_files);
    let runtime_graph = build_adjacency(&request.symbols.resolved_internal_edges, false);
    let all_graph = build_adjacency(&request.symbols.resolved_internal_edges, true);

    let runtime = bfs_reachable(
        &entry_files,
        &runtime_graph,
        request.max_files_visited,
        request.max_edges_visited,
    );
    let type_reachability = bfs_reachable(
        &entry_files,
        &all_graph,
        request.max_files_visited,
        request.max_edges_visited,
    );

    let bounded_out_reason = runtime
        .bounded_out_reason
        .clone()
        .or_else(|| type_reachability.bounded_out_reason.clone());
    let reachable_files = runtime
        .visited
        .union(&type_reachability.visited)
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut bounded_out_files = BTreeSet::new();
    let mut unreachable_files = BTreeSet::new();
    for file in &known_files {
        if reachable_files.contains(file) {
            continue;
        }
        if bounded_out_reason.is_some() {
            bounded_out_files.insert(file.clone());
        } else {
            unreachable_files.insert(file.clone());
        }
    }

    let unreachable_strongly_connected_components = unreachable_runtime_sccs(
        &known_files,
        &runtime_graph,
        &unreachable_files,
        bounded_out_reason.as_deref(),
    );
    let unreachable_strongly_connected_files = unreachable_strongly_connected_components
        .iter()
        .map(|component| component.size)
        .sum();

    let summary = ModuleReachabilitySummary {
        runtime_reachable: runtime.visited.len(),
        type_reachable: type_reachability.visited.len(),
        reachable: reachable_files.len(),
        bounded_out: bounded_out_files.len(),
        unreachable: unreachable_files.len(),
        unreachable_strongly_connected_components: unreachable_strongly_connected_components.len(),
        unreachable_strongly_connected_files,
        known_files: known_files.len(),
    };

    Ok(ModuleReachabilityArtifact {
        meta: ModuleReachabilityMeta {
            tool: TOOL_NAME,
            generated: request.generated.unwrap_or_else(|| "unknown".to_string()),
            root: request.root,
            schema_version: MODULE_REACHABILITY_SCHEMA_VERSION,
            mode: MODE_FULL_BFS,
            entry_surface_file: ENTRY_SURFACE_FILE,
            global_completeness: request.entry_surface.global_completeness,
            completeness_by_submodule: request.entry_surface.completeness_by_submodule,
            max_files_visited: request.max_files_visited,
            max_edges_visited: request.max_edges_visited,
            bounded_out_reason,
            supports: ModuleReachabilitySupports {
                runtime_reachable_files: true,
                type_reachable_files: true,
                bounded_out_files: true,
                unreachable_strongly_connected_components: true,
            },
        },
        runtime_reachable_files: sorted_set(&runtime.visited),
        type_reachable_files: sorted_set(&type_reachability.visited),
        reachable_files: sorted_set(&reachable_files),
        bounded_out_files: sorted_set(&bounded_out_files),
        unreachable_files: sorted_set(&unreachable_files),
        unreachable_strongly_connected_components,
        summary,
    })
}

fn default_max_files_visited() -> usize {
    DEFAULT_MAX_FILES_VISITED
}

fn default_max_edges_visited() -> usize {
    DEFAULT_MAX_EDGES_VISITED
}

fn default_global_completeness() -> String {
    "low".to_string()
}

fn normalize_rel(value: &str) -> String {
    value.replace('\\', "/")
}

fn value_to_path(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => normalize_rel(value),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Array(_) | Value::Object(_) => String::new(),
    }
}

fn collect_known_files(
    symbols: &SymbolsInput,
    entry_surface: &EntrySurfaceInput,
) -> BTreeSet<String> {
    let mut files = BTreeSet::new();
    for file in symbols.def_index.keys() {
        files.insert(normalize_rel(file));
    }
    for file in symbols.re_exports_by_file.keys() {
        files.insert(normalize_rel(file));
    }
    for edge in &symbols.resolved_internal_edges {
        let from = value_to_path(&edge.from);
        if !from.is_empty() {
            files.insert(from);
        }
        let to = value_to_path(&edge.to);
        if !to.is_empty() {
            files.insert(to);
        }
    }
    for file in &entry_surface.entry_files {
        files.insert(normalize_rel(file));
    }
    files
}

fn ordered_entry_files(entry_files: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for file in entry_files {
        let normalized = normalize_rel(file);
        if normalized.is_empty() || !seen.insert(normalized.clone()) {
            continue;
        }
        out.push(normalized);
    }
    out
}

fn build_adjacency(
    edges: &[ResolvedInternalEdge],
    include_type_only: bool,
) -> BTreeMap<String, Vec<String>> {
    let mut buckets: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for edge in edges {
        if !include_type_only && edge.type_only == Some(true) {
            continue;
        }
        let from = value_to_path(&edge.from);
        let to = value_to_path(&edge.to);
        if from.is_empty() || to.is_empty() {
            continue;
        }
        buckets.entry(from).or_default().insert(to);
    }
    buckets
        .into_iter()
        .map(|(from, targets)| (from, targets.into_iter().collect()))
        .collect()
}

fn bfs_reachable(
    seeds: &[String],
    adjacency: &BTreeMap<String, Vec<String>>,
    max_files_visited: usize,
    max_edges_visited: usize,
) -> BfsResult {
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    let mut edges_visited = 0usize;
    let mut bounded_out_reason = None;

    for seed in seeds {
        if seed.is_empty() || visited.contains(seed) {
            continue;
        }
        if visited.len() >= max_files_visited {
            bounded_out_reason = Some("max-files-visited".to_string());
            break;
        }
        visited.insert(seed.clone());
        queue.push_back(seed.clone());
    }

    while bounded_out_reason.is_none() {
        let Some(current) = queue.pop_front() else {
            break;
        };
        let Some(targets) = adjacency.get(&current) else {
            continue;
        };
        for next in targets {
            edges_visited += 1;
            if edges_visited > max_edges_visited {
                bounded_out_reason = Some("max-edges-visited".to_string());
                break;
            }
            if visited.contains(next) {
                continue;
            }
            if visited.len() >= max_files_visited {
                bounded_out_reason = Some("max-files-visited".to_string());
                break;
            }
            visited.insert(next.clone());
            queue.push_back(next.clone());
        }
    }

    BfsResult {
        visited,
        bounded_out_reason,
    }
}

fn unreachable_runtime_sccs(
    known_files: &BTreeSet<String>,
    runtime_graph: &BTreeMap<String, Vec<String>>,
    unreachable_files: &BTreeSet<String>,
    bounded_out_reason: Option<&str>,
) -> Vec<UnreachableStronglyConnectedComponent> {
    if bounded_out_reason.is_some() {
        return Vec::new();
    }
    strongly_connected_components(known_files, runtime_graph)
        .into_iter()
        .filter(|files| {
            files.len() > 1 && files.iter().all(|file| unreachable_files.contains(file))
        })
        .map(|files| UnreachableStronglyConnectedComponent {
            kind: "entry-unreachable-scc",
            graph: "runtime",
            size: files.len(),
            files,
            note: SCC_NOTE,
        })
        .collect()
}

fn strongly_connected_components(
    nodes: &BTreeSet<String>,
    adjacency: &BTreeMap<String, Vec<String>>,
) -> Vec<Vec<String>> {
    let normalized_adjacency = normalize_adjacency_for_nodes(nodes, adjacency);
    let reverse = build_reverse_adjacency(nodes, &normalized_adjacency);
    let order = finish_order(nodes, &normalized_adjacency);
    let mut assigned = BTreeSet::new();
    let mut components = Vec::new();

    for start in order.iter().rev() {
        if assigned.contains(start) {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![start.clone()];
        assigned.insert(start.clone());

        while let Some(node) = stack.pop() {
            component.push(node.clone());
            for next in reverse.get(&node).into_iter().flatten() {
                if assigned.contains(next) {
                    continue;
                }
                assigned.insert(next.clone());
                stack.push(next.clone());
            }
        }

        component.sort();
        components.push(component);
    }

    components.sort_by(|left, right| {
        right
            .len()
            .cmp(&left.len())
            .then_with(|| first_or_empty(left).cmp(first_or_empty(right)))
    });
    components
}

fn normalize_adjacency_for_nodes(
    nodes: &BTreeSet<String>,
    adjacency: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, Vec<String>> {
    nodes
        .iter()
        .map(|node| {
            let targets = adjacency
                .get(node)
                .into_iter()
                .flatten()
                .filter(|target| nodes.contains(*target))
                .cloned()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            (node.clone(), targets)
        })
        .collect()
}

fn build_reverse_adjacency(
    nodes: &BTreeSet<String>,
    adjacency: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, Vec<String>> {
    let mut reverse = nodes
        .iter()
        .map(|node| (node.clone(), BTreeSet::<String>::new()))
        .collect::<BTreeMap<_, _>>();
    for from in nodes {
        for to in adjacency.get(from).into_iter().flatten() {
            if let Some(targets) = reverse.get_mut(to) {
                targets.insert(from.clone());
            }
        }
    }
    reverse
        .into_iter()
        .map(|(node, targets)| (node, targets.into_iter().collect()))
        .collect()
}

fn finish_order(
    nodes: &BTreeSet<String>,
    adjacency: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut visited = BTreeSet::new();
    let mut order = Vec::new();

    for start in nodes {
        if visited.contains(start) {
            continue;
        }
        visited.insert(start.clone());
        let mut stack = vec![FinishFrame {
            node: start.clone(),
            next_index: 0,
        }];

        while let Some(frame) = stack.last_mut() {
            let targets = adjacency.get(&frame.node).map(Vec::as_slice).unwrap_or(&[]);
            if frame.next_index >= targets.len() {
                let node = frame.node.clone();
                stack.pop();
                order.push(node);
                continue;
            }

            let next = targets[frame.next_index].clone();
            frame.next_index += 1;
            if visited.contains(&next) {
                continue;
            }
            visited.insert(next.clone());
            stack.push(FinishFrame {
                node: next,
                next_index: 0,
            });
        }
    }

    order
}

#[derive(Debug)]
struct FinishFrame {
    node: String,
    next_index: usize,
}

fn sorted_set(set: &BTreeSet<String>) -> Vec<String> {
    set.iter().cloned().collect()
}

fn first_or_empty(values: &[String]) -> &str {
    values.first().map(String::as_str).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use serde_json::json;

    fn request(edges: Vec<Value>, entry_files: Vec<&str>) -> Result<ModuleReachabilityRequest> {
        Ok(serde_json::from_value(json!({
            "schemaVersion": MODULE_REACHABILITY_REQUEST_SCHEMA_VERSION,
            "root": "/repo",
            "generated": "2026-07-03T00:00:00.000Z",
            "symbols": {
                "defIndex": {
                    "src/index.ts": {},
                    "src/isolated.ts": {}
                },
                "reExportsByFile": {},
                "resolvedInternalEdges": edges
            },
            "entrySurface": {
                "entryFiles": entry_files,
                "globalCompleteness": "high",
                "completenessBySubmodule": { "src": "high" }
            }
        }))?)
    }

    fn artifact(edges: Vec<Value>, entry_files: Vec<&str>) -> Result<ModuleReachabilityArtifact> {
        build_module_reachability_artifact(request(edges, entry_files)?)
    }

    #[test]
    fn runtime_and_type_reachability_match_checked_js_semantics() -> Result<()> {
        let artifact = artifact(
            vec![
                json!({ "from": "src/index.ts", "to": "src/runtime.ts" }),
                json!({ "from": "src/index.ts", "to": "src/types.ts", "typeOnly": true }),
                json!({ "from": "src/runtime.ts", "to": "src/deep.ts" }),
            ],
            vec!["src/index.ts"],
        )?;

        assert_eq!(
            artifact.runtime_reachable_files,
            vec!["src/deep.ts", "src/index.ts", "src/runtime.ts"]
        );
        assert_eq!(
            artifact.type_reachable_files,
            vec![
                "src/deep.ts",
                "src/index.ts",
                "src/runtime.ts",
                "src/types.ts"
            ]
        );
        assert_eq!(artifact.summary.unreachable, 1);
        assert!(artifact
            .unreachable_files
            .contains(&"src/isolated.ts".to_string()));
        Ok(())
    }

    #[test]
    fn bounded_file_limit_records_bounded_out_not_unreachable_or_scc() -> Result<()> {
        let mut request = request(
            vec![
                json!({ "from": "src/index.ts", "to": "src/a.ts" }),
                json!({ "from": "src/a.ts", "to": "src/b.ts" }),
                json!({ "from": "src/b.ts", "to": "src/a.ts" }),
            ],
            vec!["src/index.ts"],
        )?;
        request.max_files_visited = 1;

        let artifact = build_module_reachability_artifact(request)?;
        assert_eq!(
            artifact.meta.bounded_out_reason.as_deref(),
            Some("max-files-visited")
        );
        assert!(artifact.unreachable_files.is_empty());
        assert!(artifact.bounded_out_files.contains(&"src/a.ts".to_string()));
        assert!(artifact
            .unreachable_strongly_connected_components
            .is_empty());
        Ok(())
    }

    #[test]
    fn bounded_edge_limit_uses_js_off_by_one_and_deduped_edges() -> Result<()> {
        let mut request = request(
            vec![
                json!({ "from": "src/index.ts", "to": "src/a.ts" }),
                json!({ "from": "src/index.ts", "to": "src/a.ts" }),
                json!({ "from": "src/index.ts", "to": "src/b.ts" }),
            ],
            vec!["src/index.ts"],
        )?;
        request.max_edges_visited = 1;

        let artifact = build_module_reachability_artifact(request)?;
        assert_eq!(
            artifact.meta.bounded_out_reason.as_deref(),
            Some("max-edges-visited")
        );
        assert_eq!(
            artifact.runtime_reachable_files,
            vec!["src/a.ts", "src/index.ts"]
        );
        assert!(artifact.bounded_out_files.contains(&"src/b.ts".to_string()));
        Ok(())
    }

    #[test]
    fn re_exports_are_known_files_not_adjacency() -> Result<()> {
        let mut request = request(Vec::new(), vec!["src/index.ts"])?;
        request
            .symbols
            .re_exports_by_file
            .insert("src/barrel.ts".to_string(), json!({ "anything": true }));

        let artifact = build_module_reachability_artifact(request)?;
        assert!(artifact
            .unreachable_files
            .contains(&"src/barrel.ts".to_string()));
        assert!(!artifact
            .reachable_files
            .contains(&"src/barrel.ts".to_string()));
        Ok(())
    }

    #[test]
    fn path_normalization_and_empty_edges_match_js_helper_scope() -> Result<()> {
        let artifact = artifact(
            vec![
                json!({ "from": "src\\index.ts", "to": "C:/repo/src\\windows.ts" }),
                json!({ "from": "", "to": "src/ignored.ts" }),
                json!({ "from": "src/windows.ts", "to": "" }),
            ],
            vec!["src\\index.ts", "src\\index.ts"],
        )?;

        assert!(artifact
            .runtime_reachable_files
            .contains(&"src/index.ts".to_string()));
        assert!(artifact
            .runtime_reachable_files
            .contains(&"C:/repo/src/windows.ts".to_string()));
        assert!(!artifact
            .reachable_files
            .contains(&"src/ignored.ts".to_string()));
        Ok(())
    }

    #[test]
    fn entry_files_absent_from_symbols_are_still_known_reachable_seeds() -> Result<()> {
        let artifact = artifact(Vec::new(), vec!["src/entry-only.ts"])?;
        assert!(artifact
            .runtime_reachable_files
            .contains(&"src/entry-only.ts".to_string()));
        assert!(artifact
            .reachable_files
            .contains(&"src/entry-only.ts".to_string()));
        Ok(())
    }

    #[test]
    fn unreachable_runtime_sccs_are_review_evidence_and_self_loops_are_omitted() -> Result<()> {
        let artifact = artifact(
            vec![
                json!({ "from": "src/a.ts", "to": "src/b.ts" }),
                json!({ "from": "src/b.ts", "to": "src/a.ts" }),
                json!({ "from": "src/self.ts", "to": "src/self.ts" }),
            ],
            vec!["src/index.ts"],
        )?;

        assert_eq!(artifact.unreachable_strongly_connected_components.len(), 1);
        let component = artifact
            .unreachable_strongly_connected_components
            .first()
            .context("expected unreachable SCC")?;
        assert_eq!(component.files, vec!["src/a.ts", "src/b.ts"]);
        assert_eq!(component.kind, "entry-unreachable-scc");
        assert_eq!(component.graph, "runtime");
        assert_eq!(component.note, SCC_NOTE);
        assert!(!artifact
            .unreachable_strongly_connected_components
            .iter()
            .any(|component| component.files == vec!["src/self.ts"]));
        Ok(())
    }

    #[test]
    fn rejects_bad_schema_and_zero_limits() -> Result<()> {
        let mut bad_schema = request(Vec::new(), vec!["src/index.ts"])?;
        bad_schema.schema_version = "wrong".to_string();
        let err = build_module_reachability_artifact(bad_schema)
            .err()
            .context("wrong schema should fail")?;
        assert!(err.to_string().contains("unsupported schemaVersion"));

        let mut zero_limit = request(Vec::new(), vec!["src/index.ts"])?;
        zero_limit.max_files_visited = 0;
        let err = build_module_reachability_artifact(zero_limit)
            .err()
            .context("zero limit should fail")?;
        assert!(err.to_string().contains("maxFilesVisited"));
        Ok(())
    }
}
