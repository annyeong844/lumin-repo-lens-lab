#[path = "edge/assertions.rs"]
mod assertions;

#[allow(unused_imports)]
pub use assertions::{
    assert_edge, assert_edge_at, assert_first_dynamic_source, assert_first_source,
    assert_reexport_pair,
};
