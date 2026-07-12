use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

pub const TOPOLOGY_MERMAID_RENDER_REQUEST_SCHEMA_VERSION: &str =
    "lumin-topology-mermaid-render-request.v1";
pub const TOPOLOGY_MERMAID_RENDER_RESULT_SCHEMA_VERSION: &str =
    "lumin-topology-mermaid-render-result.v1";

const DEFAULT_EDGE_LIMIT: usize = 30;
const DEFAULT_CYCLE_LIMIT: usize = 5;
const DEFAULT_HUB_LIMIT: usize = 10;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyMermaidRenderRequest {
    pub schema_version: String,
    pub topology: Value,
    pub output_path: String,
    #[serde(default)]
    pub options: TopologyMermaidOptions,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyMermaidOptions {
    pub edge_limit: Option<Value>,
    pub cycle_limit: Option<Value>,
    pub hub_limit: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyMermaidRenderResult {
    pub schema_version: &'static str,
    pub path: String,
    pub bytes: usize,
}

#[derive(Debug, Clone)]
struct CrossEdge {
    from: String,
    to: String,
    count: f64,
}

#[derive(Debug, Clone)]
struct HubRow {
    file: String,
    count: f64,
}

pub fn render_topology_mermaid_request(
    request: &TopologyMermaidRenderRequest,
) -> Result<(String, TopologyMermaidRenderResult)> {
    if request.schema_version != TOPOLOGY_MERMAID_RENDER_REQUEST_SCHEMA_VERSION {
        bail!(
            "topology-mermaid-render: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    let markdown = render_topology_mermaid(&request.topology, &request.options);
    let result = TopologyMermaidRenderResult {
        schema_version: TOPOLOGY_MERMAID_RENDER_RESULT_SCHEMA_VERSION,
        path: request.output_path.clone(),
        bytes: markdown.len(),
    };
    Ok((markdown, result))
}

pub fn render_topology_mermaid(topology: &Value, options: &TopologyMermaidOptions) -> String {
    let edge_limit = limit(options.edge_limit.as_ref(), DEFAULT_EDGE_LIMIT);
    let cycle_limit = limit(options.cycle_limit.as_ref(), DEFAULT_CYCLE_LIMIT);
    let hub_limit = limit(options.hub_limit.as_ref(), DEFAULT_HUB_LIMIT);
    let generated = topology
        .pointer("/meta/generated")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let lens = topology
        .pointer("/summary/lens")
        .and_then(Value::as_str)
        .unwrap_or("runtime");

    let mut lines = vec![
        "# Topology Mermaid".to_string(),
        String::new(),
        "This document is a visual companion for `topology.json`, not citation authority."
            .to_string(),
        String::new(),
        format!("Generated: {generated}"),
        format!("Lens: {lens}"),
        String::new(),
        "## How To Read This".to_string(),
        String::new(),
        "- Use the Mermaid blocks to understand the shape of cross-submodule flow and runtime cycles."
            .to_string(),
        "- Use the hub lists to find high-degree files before opening raw JSON.".to_string(),
        "- For exact counts, complete lists, or grounded claims, cite `topology.json` path/value evidence."
            .to_string(),
        String::new(),
    ];
    lines.extend(render_cross_submodule_graph(topology, edge_limit));
    lines.extend(render_cycle_graph(topology, cycle_limit));
    lines.extend(render_hub_files(topology, hub_limit));
    lines.extend(render_limits(topology, edge_limit, cycle_limit, hub_limit));
    lines.extend([
        "## Citation Contract".to_string(),
        String::new(),
        "- This artifact is a visual companion, not citation authority.".to_string(),
        "- Cite `topology.json` for topology claims, including counts, absence claims, SCC membership, and complete edge lists."
            .to_string(),
        "- Mermaid blocks and hub lists are capped so large repositories stay readable.".to_string(),
        String::new(),
    ]);
    lines.join("\n")
}

fn arr(value: Option<&Value>) -> &[Value] {
    value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn number(value: Option<&Value>, fallback: f64) -> f64 {
    value.and_then(Value::as_f64).unwrap_or(fallback)
}

fn limit(value: Option<&Value>, fallback: usize) -> usize {
    number(value, fallback as f64).trunc().max(0.0) as usize
}

fn escape_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn display_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn normalize_cross_edge(edge: &Value) -> Option<CrossEdge> {
    let object = edge.as_object()?;
    let count = number(object.get("count"), 1.0);
    if let (Some(from), Some(to)) = (
        object.get("from").and_then(Value::as_str),
        object.get("to").and_then(Value::as_str),
    ) {
        return Some(CrossEdge {
            from: from.to_string(),
            to: to.to_string(),
            count,
        });
    }
    let edge_text = object.get("edge").and_then(Value::as_str)?;
    let (delimiter, width) = if edge_text.contains(" → ") {
        (" → ", " → ".len())
    } else {
        (" -> ", " -> ".len())
    };
    let split_at = edge_text.find(delimiter)?;
    Some(CrossEdge {
        from: edge_text[..split_at].to_string(),
        to: edge_text[split_at + width..].to_string(),
        count,
    })
}

fn cross_edge_source(topology: &Value) -> (&'static str, &[Value]) {
    if topology
        .get("crossSubmoduleEdges")
        .and_then(Value::as_array)
        .is_some()
    {
        (
            "topology.json.crossSubmoduleEdges",
            arr(topology.get("crossSubmoduleEdges")),
        )
    } else {
        (
            "topology.json.crossSubmoduleTop",
            arr(topology.get("crossSubmoduleTop")),
        )
    }
}

fn sorted_cross_edges(topology: &Value) -> (&'static str, Vec<CrossEdge>) {
    let (path, edges) = cross_edge_source(topology);
    let mut edges = edges
        .iter()
        .filter_map(normalize_cross_edge)
        .collect::<Vec<_>>();
    edges.sort_by(compare_cross_edges);
    (path, edges)
}

fn compare_cross_edges(left: &CrossEdge, right: &CrossEdge) -> Ordering {
    right
        .count
        .partial_cmp(&left.count)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.from.cmp(&right.from))
        .then_with(|| left.to.cmp(&right.to))
}

fn render_cross_submodule_graph(topology: &Value, limit: usize) -> Vec<String> {
    let (path, edges) = sorted_cross_edges(topology);
    let shown = edges.iter().take(limit).collect::<Vec<_>>();
    if edges.is_empty() {
        return vec![
            "## Cross-Submodule Edges".to_string(),
            String::new(),
            format!("- No cross-submodule edges were observed in `{path}`."),
            String::new(),
        ];
    }

    let mut ids = BTreeMap::<String, String>::new();
    let mut emitted = BTreeSet::<String>::new();
    let mut lines = vec![
        "## Cross-Submodule Edges".to_string(),
        String::new(),
        format!(
            "Showing {} of {} cross-submodule edge{} (cap: {}). Source: `{}`.",
            shown.len(),
            edges.len(),
            if edges.len() == 1 { "" } else { "s" },
            limit,
            path
        ),
        String::new(),
        "```mermaid".to_string(),
        "flowchart LR".to_string(),
    ];
    for edge in shown {
        let from = id_for(&mut ids, &edge.from, "sub");
        let to = id_for(&mut ids, &edge.to, "sub");
        emit_node(&mut lines, &mut emitted, &from, &edge.from);
        emit_node(&mut lines, &mut emitted, &to, &edge.to);
        lines.push(format!("  {from} -->|{}| {to}", display_number(edge.count)));
    }
    lines.extend(["```".to_string(), String::new()]);
    lines
}

fn id_for(ids: &mut BTreeMap<String, String>, name: &str, prefix: &str) -> String {
    if let Some(id) = ids.get(name) {
        return id.clone();
    }
    let id = format!("{prefix}{}", ids.len());
    ids.insert(name.to_string(), id.clone());
    id
}

fn emit_node(lines: &mut Vec<String>, emitted: &mut BTreeSet<String>, id: &str, label: &str) {
    if emitted.insert(id.to_string()) {
        lines.push(format!("  {id}[\"{}\"]", escape_label(label)));
    }
}

fn cycle_edges(topology: &Value, members: &[String]) -> Vec<(String, String)> {
    let member_set = members.iter().map(String::as_str).collect::<BTreeSet<_>>();
    arr(topology.get("edges"))
        .iter()
        .filter_map(|edge| {
            let object = edge.as_object()?;
            let from = object.get("from").and_then(Value::as_str)?;
            let to = object.get("to").and_then(Value::as_str)?;
            if !member_set.contains(from) || !member_set.contains(to) {
                return None;
            }
            if object.get("typeOnly").and_then(Value::as_bool) == Some(true) {
                return None;
            }
            Some((from.to_string(), to.to_string()))
        })
        .collect()
}

fn render_cycle_graph(topology: &Value, limit: usize) -> Vec<String> {
    let all_sccs = arr(topology.get("sccs"));
    let lens = topology
        .pointer("/summary/lens")
        .and_then(Value::as_str)
        .unwrap_or("runtime");
    if all_sccs.is_empty() {
        return vec![
            "## Runtime Cycles".to_string(),
            String::new(),
            format!("- No runtime cycles were observed in `topology.json.sccs[]` (lens: {lens})."),
            String::new(),
        ];
    }

    let mut lines = vec![
        "## Runtime Cycles".to_string(),
        String::new(),
        format!(
            "Showing {} of {} runtime cycle{} (cap: {}). Source: `topology.json.sccs[]` (lens: {}).",
            all_sccs.len().min(limit),
            all_sccs.len(),
            if all_sccs.len() == 1 { "" } else { "s" },
            limit,
            lens
        ),
        String::new(),
        "```mermaid".to_string(),
        "flowchart LR".to_string(),
    ];

    for (i, scc) in all_sccs.iter().take(limit).enumerate() {
        let members = arr(scc.get("members"))
            .iter()
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let ids = members
            .iter()
            .enumerate()
            .map(|(index, member)| (member.clone(), format!("scc{i}_{index}")))
            .collect::<BTreeMap<_, _>>();
        lines.push(format!(
            "  subgraph cluster{}[\"SCC {} ({} files)\"]",
            i,
            i + 1,
            members.len()
        ));
        for member in &members {
            if let Some(id) = ids.get(member) {
                lines.push(format!("    {id}[\"{}\"]", escape_label(member)));
            }
        }
        for (from, to) in cycle_edges(topology, &members) {
            if let (Some(from_id), Some(to_id)) = (ids.get(&from), ids.get(&to)) {
                lines.push(format!("    {from_id} --> {to_id}"));
            }
        }
        lines.push("  end".to_string());
    }
    lines.extend(["```".to_string(), String::new()]);
    lines
}

fn normalize_hub(row: &Value) -> Option<HubRow> {
    let object = row.as_object()?;
    Some(HubRow {
        file: object.get("file").and_then(Value::as_str)?.to_string(),
        count: number(object.get("count"), 0.0),
    })
}

fn sorted_hubs(rows: Option<&Value>) -> Vec<HubRow> {
    let mut rows = arr(rows)
        .iter()
        .filter_map(normalize_hub)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .count
            .partial_cmp(&left.count)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.file.cmp(&right.file))
    });
    rows
}

fn render_hub_files(topology: &Value, limit: usize) -> Vec<String> {
    let fan_in = sorted_hubs(topology.get("topFanIn"));
    let fan_out = sorted_hubs(topology.get("topFanOut"));
    let shown_in = fan_in.iter().take(limit).collect::<Vec<_>>();
    let shown_out = fan_out.iter().take(limit).collect::<Vec<_>>();
    let mut lines = vec!["## Hub Files".to_string(), String::new()];

    if fan_in.is_empty() && fan_out.is_empty() {
        lines.extend([
            "- No hub files were available from `topology.json.topFanIn` or `topology.json.topFanOut`."
                .to_string(),
            String::new(),
        ]);
        return lines;
    }

    lines.push(format!(
        "Showing {} of {} fan-in files from `topology.json.topFanIn` (cap: {}).",
        shown_in.len(),
        fan_in.len(),
        limit
    ));
    for row in shown_in {
        lines.push(format!(
            "- `{}` — {} inbound",
            row.file,
            display_number(row.count)
        ));
    }
    if fan_in.is_empty() {
        lines.push("- No fan-in rows were available from `topology.json.topFanIn`.".to_string());
    }
    lines.push(String::new());

    lines.push(format!(
        "Showing {} of {} fan-out files from `topology.json.topFanOut` (cap: {}).",
        shown_out.len(),
        fan_out.len(),
        limit
    ));
    for row in shown_out {
        lines.push(format!(
            "- `{}` — {} outbound",
            row.file,
            display_number(row.count)
        ));
    }
    if fan_out.is_empty() {
        lines.push("- No fan-out rows were available from `topology.json.topFanOut`.".to_string());
    }
    lines.push(String::new());
    lines
}

fn render_limits(
    topology: &Value,
    edge_limit: usize,
    cycle_limit: usize,
    hub_limit: usize,
) -> Vec<String> {
    let (path, cross_edges) = sorted_cross_edges(topology);
    let sccs = arr(topology.get("sccs"));
    let fan_in = sorted_hubs(topology.get("topFanIn"));
    let fan_out = sorted_hubs(topology.get("topFanOut"));
    vec![
        "## Omitted Detail / Limits".to_string(),
        String::new(),
        format!(
            "- Cross-submodule edges: showing {} of {}; cap {}; source `{}`.",
            edge_limit.min(cross_edges.len()),
            cross_edges.len(),
            edge_limit,
            path
        ),
        format!(
            "- Runtime cycles: showing {} of {}; cap {}; source `topology.json.sccs[]`.",
            cycle_limit.min(sccs.len()),
            sccs.len(),
            cycle_limit
        ),
        format!(
            "- Hub fan-in files: showing {} of {}; cap {}; source `topology.json.topFanIn`.",
            hub_limit.min(fan_in.len()),
            fan_in.len(),
            hub_limit
        ),
        format!(
            "- Hub fan-out files: showing {} of {}; cap {}; source `topology.json.topFanOut`.",
            hub_limit.min(fan_out.len()),
            fan_out.len(),
            hub_limit
        ),
        String::new(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn renders_stable_sections_and_graphs() {
        let markdown = render_topology_mermaid(
            &json!({
                "meta": { "generated": "2026-05-01T00:00:00.000Z" },
                "summary": { "lens": "runtime", "sccCount": 1 },
                "crossSubmoduleEdges": [
                    { "from": "apps/web", "to": "packages/ui", "count": 4 },
                    { "from": "apps/web", "to": "packages/api", "count": 2 }
                ],
                "topFanIn": [{ "file": "packages/ui/src/button.ts", "count": 8 }],
                "topFanOut": [{ "file": "apps/web/src/app.ts", "count": 5 }],
                "sccs": [{ "size": 2, "members": ["src/a.ts", "src/b.ts"] }],
                "edges": [
                    { "from": "src/a.ts", "to": "src/b.ts", "typeOnly": false },
                    { "from": "src/b.ts", "to": "src/a.ts", "typeOnly": false }
                ]
            }),
            &TopologyMermaidOptions::default(),
        );

        assert!(markdown.starts_with("# Topology Mermaid"));
        assert!(markdown.contains("```mermaid"));
        for section in [
            "## How To Read This",
            "## Cross-Submodule Edges",
            "## Runtime Cycles",
            "## Hub Files",
            "## Omitted Detail / Limits",
            "## Citation Contract",
        ] {
            assert!(markdown.contains(section));
        }
        assert!(markdown.contains("flowchart LR"));
        assert!(markdown.contains("sub0[\"apps/web\"]"));
        assert!(markdown.contains("sub1[\"packages/ui\"]"));
        assert!(markdown.contains("sub0 -->|4| sub1"));
        assert!(markdown.contains("scc0_0[\"src/a.ts\"]"));
        assert!(markdown.contains("scc0_1[\"src/b.ts\"]"));
        assert!(markdown.contains("scc0_0 --> scc0_1"));
        assert!(markdown.contains("scc0_1 --> scc0_0"));
        assert!(markdown.contains("packages/ui/src/button.ts"));
        assert!(markdown.contains("8 inbound"));
        assert!(markdown.contains("apps/web/src/app.ts"));
        assert!(markdown.contains("5 outbound"));
        assert!(markdown.contains("not citation authority"));
        assert!(markdown.contains("cite `topology.json`"));
    }

    #[test]
    fn renders_empty_states_and_escapes_labels() {
        let empty = render_topology_mermaid(
            &json!({
                "summary": { "lens": "runtime", "sccCount": 0 },
                "crossSubmoduleEdges": [],
                "sccs": [],
                "edges": []
            }),
            &TopologyMermaidOptions::default(),
        );
        assert!(empty.contains("No cross-submodule edges were observed"));
        assert!(empty.contains("No runtime cycles were observed"));
        assert!(empty.contains("No hub files were available"));

        let escaped = render_topology_mermaid(
            &json!({
                "summary": { "lens": "runtime", "sccCount": 0 },
                "crossSubmoduleEdges": [{ "from": "a\"b", "to": "x[y]", "count": 1 }],
                "sccs": [],
                "edges": []
            }),
            &TopologyMermaidOptions::default(),
        );
        assert!(escaped.contains("sub0[\"a\\\"b\"]"));
        assert!(escaped.contains("sub1[\"x[y]\"]"));
    }

    #[test]
    fn honors_caps_and_avoids_dangling_cycle_ids() {
        let edges = (0..31)
            .map(|i| json!({ "from": format!("pkg{i}"), "to": "core", "count": i + 1 }))
            .collect::<Vec<_>>();
        let markdown = render_topology_mermaid(
            &json!({
                "summary": { "lens": "runtime", "sccCount": 1 },
                "crossSubmoduleEdges": edges,
                "sccs": [
                    { "size": 2, "members": ["src/a.ts", "src/b.ts"] },
                    { "size": 2, "members": ["src/c.ts", "src/d.ts"] }
                ],
                "edges": [
                    { "from": "src/a.ts", "to": "src/missing.ts", "typeOnly": false },
                    { "from": "src/a.ts", "to": "src/b.ts", "typeOnly": true }
                ]
            }),
            &TopologyMermaidOptions {
                edge_limit: Some(json!(3)),
                cycle_limit: Some(json!(1)),
                hub_limit: None,
            },
        );

        assert!(markdown.contains("Showing 3 of 31 cross-submodule edges (cap: 3)."));
        assert!(markdown.contains("pkg30"));
        assert!(markdown.contains("|31|"));
        assert!(!markdown.contains("pkg0[\"pkg0\"]"));
        assert!(markdown.contains("Showing 1 of 2 runtime cycles (cap: 1)."));
        assert!(markdown.contains("SCC 1"));
        assert!(!markdown.contains("SCC 2"));
        assert!(!markdown.contains("undefined"));
        assert!(!markdown.contains("--> undefined"));
    }

    #[test]
    fn reads_legacy_cross_submodule_top_source() {
        let markdown = render_topology_mermaid(
            &json!({
                "summary": { "lens": "runtime" },
                "crossSubmoduleTop": [{ "edge": "a → b", "count": 2 }]
            }),
            &TopologyMermaidOptions::default(),
        );
        assert!(markdown.contains("Source: `topology.json.crossSubmoduleTop`."));
        assert!(markdown.contains("sub0[\"a\"]"));
        assert!(markdown.contains("sub1[\"b\"]"));
        assert!(markdown.contains("sub0 -->|2| sub1"));
    }
}
