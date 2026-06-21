use lumin_topology_scanner::protocol::ModuleEdge;

pub fn assert_edge_at(
    edges: &[ModuleEdge],
    source: &str,
    line: usize,
    type_only: bool,
    re_export: bool,
    dynamic: bool,
) {
    assert!(
        edges.iter().any(|edge| edge.source == source
            && edge.line == line
            && edge.type_only == type_only
            && edge.re_export == re_export
            && edge.dynamic == dynamic),
        "missing edge {source:?} line={line} type_only={type_only} re_export={re_export} dynamic={dynamic}; edges={edges:?}"
    );
}
