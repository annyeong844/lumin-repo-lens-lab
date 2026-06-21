use lumin_topology_scanner::protocol::ModuleEdge;
use lumin_topology_scanner::scan_file_text;

pub fn scan_ok(file: &str, source: &str, expected_edges: usize) -> Vec<ModuleEdge> {
    let result = scan_file_text(file, source);
    assert!(result.ok, "unexpected risks: {:?}", result.risk);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), expected_edges);
    result.edges
}
