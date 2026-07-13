use super::{as_usize, round3, unavailable, value_at};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
struct CrossEdge {
    from: String,
    to: String,
    count: usize,
}

pub(super) fn decoupling_ratio(topology: Option<&Value>) -> Value {
    let Some(topology) = topology else {
        return unavailable("topology.json missing — run measure-topology.mjs first");
    };
    if !topology.get("summary").is_some_and(Value::is_object) {
        return unavailable("topology.json missing — run measure-topology.mjs first");
    }

    let total = value_at(topology, &["summary", "internalEdges"])
        .and_then(as_usize)
        .unwrap_or(0);
    let (source, mut edges) = normalize_cross_submodule_edges(topology);
    let cross_sum: usize = edges.iter().map(|edge| edge.count).sum();
    let layered_sum: usize = edges
        .iter()
        .filter(|edge| is_healthy_layered_cross_edge(edge))
        .map(|edge| edge.count)
        .sum();
    let reviewed_sum = cross_sum.saturating_sub(layered_sum);
    let ratio = if total > 0 {
        cross_sum as f64 / total as f64
    } else {
        0.0
    };
    let raw_gate = if ratio > 0.5 {
        "fix"
    } else if ratio > 0.3 {
        "watch"
    } else {
        "ok"
    };
    let gate = if raw_gate != "ok" && cross_sum > 0 && reviewed_sum == 0 {
        "ok"
    } else {
        raw_gate
    };
    edges.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.from.cmp(&b.from))
            .then_with(|| a.to.cmp(&b.to))
    });
    let note = if source == "full-list" {
        "ratio is exact from topology.json.crossSubmoduleEdges. Healthy layered flows (root/scripts/tests → _lib, tests → production) are visible but do not trip the gate by themselves."
    } else {
        "ratio is a LOWER bound from topology.json.crossSubmoduleTop; the true ratio may be slightly higher."
    };

    json!({
        "gate": gate,
        "rawGate": raw_gate,
        "crossSubmoduleEdgeSource": source,
        "crossSubmoduleEdgesSum": cross_sum,
        "crossSubmoduleEdgesTop30Sum": if source == "top-30" { json!(cross_sum) } else { Value::Null },
        "healthyLayeredEdgesSum": layered_sum,
        "reviewedEdgesSum": reviewed_sum,
        "totalInternalEdges": total,
        "ratioLowerBound": round3(ratio),
        "topCrossSubmoduleEdges": edges.iter().take(10).map(cross_edge_value).collect::<Vec<_>>(),
        "note": note,
    })
}

pub(super) fn cycles(topology: Option<&Value>) -> Value {
    let Some(topology) = topology else {
        return unavailable("topology.json missing");
    };
    let Some(sccs) = topology.get("sccs").and_then(Value::as_array) else {
        return unavailable("topology.json missing");
    };
    let nontrivial = sccs
        .iter()
        .filter(|scc| scc.get("size").and_then(as_usize).unwrap_or(0) >= 2)
        .cloned()
        .collect::<Vec<_>>();
    json!({
        "gate": if nontrivial.is_empty() { "ok" } else { "fix" },
        "sccCount": value_at(topology, &["summary", "sccCount"]).and_then(as_usize).unwrap_or(nontrivial.len()),
        "maxSccSize": value_at(topology, &["summary", "maxSccSize"]).and_then(as_usize).unwrap_or(0),
        "lens": value_at(topology, &["summary", "lens"]).and_then(Value::as_str).unwrap_or("unknown"),
        "topSccs": nontrivial.into_iter().take(5).collect::<Vec<_>>(),
    })
}

fn normalize_cross_submodule_edges(topology: &Value) -> (&'static str, Vec<CrossEdge>) {
    if let Some(edges) = topology
        .get("crossSubmoduleEdges")
        .and_then(Value::as_array)
    {
        return (
            "full-list",
            edges
                .iter()
                .filter_map(|edge| {
                    Some(CrossEdge {
                        from: edge.get("from")?.as_str()?.to_string(),
                        to: edge.get("to")?.as_str()?.to_string(),
                        count: edge.get("count").and_then(as_usize).unwrap_or(0),
                    })
                })
                .collect(),
        );
    }
    if let Some(edges) = topology.get("crossSubmoduleTop").and_then(Value::as_array) {
        return (
            "top-30",
            edges
                .iter()
                .filter_map(|edge| {
                    let text = edge.get("edge")?.as_str()?;
                    let (from, to) = text.split_once(" → ")?;
                    Some(CrossEdge {
                        from: from.to_string(),
                        to: to.to_string(),
                        count: edge.get("count").and_then(as_usize).unwrap_or(0),
                    })
                })
                .collect(),
        );
    }
    ("absent", Vec::new())
}

fn is_healthy_layered_cross_edge(edge: &CrossEdge) -> bool {
    (edge.to == "_lib" && matches!(edge.from.as_str(), "root" | "scripts" | "tests"))
        || (edge.from == "tests" && edge.to != "tests" && edge.to != "root")
}

fn cross_edge_value(edge: &CrossEdge) -> Value {
    json!({
        "from": edge.from,
        "to": edge.to,
        "count": edge.count,
    })
}
