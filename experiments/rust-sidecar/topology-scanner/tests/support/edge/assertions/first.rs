#![allow(dead_code)]

use lumin_topology_scanner::protocol::ModuleEdge;

pub fn assert_first_source(edges: &[ModuleEdge], source: &str) {
    let Some(edge) = edges.first() else {
        panic!("expected first edge");
    };
    assert_eq!(edge.source, source);
}

pub fn assert_first_dynamic_source(edges: &[ModuleEdge], source: &str) {
    let Some(edge) = edges.first() else {
        panic!("expected first edge");
    };
    assert_eq!(edge.source, source);
    assert!(edge.dynamic);
}
