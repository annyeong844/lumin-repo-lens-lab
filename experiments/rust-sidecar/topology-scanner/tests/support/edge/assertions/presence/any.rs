#![allow(dead_code)]

use lumin_topology_scanner::protocol::ModuleEdge;

pub fn assert_edge(
    edges: &[ModuleEdge],
    source: &str,
    type_only: bool,
    re_export: bool,
    dynamic: bool,
) {
    assert!(
        edges.iter().any(|edge| edge.source == source
            && edge.type_only == type_only
            && edge.re_export == re_export
            && edge.dynamic == dynamic),
        "missing edge {source:?} type_only={type_only} re_export={re_export} dynamic={dynamic}; edges={edges:?}"
    );
}

pub fn assert_reexport_pair(edges: &[ModuleEdge], source: &str) {
    assert_edge(edges, source, false, true, false);
    assert_edge(edges, source, true, true, false);
}
