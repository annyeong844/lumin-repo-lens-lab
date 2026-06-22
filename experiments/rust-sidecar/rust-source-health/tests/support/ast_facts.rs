#![allow(dead_code, unused_imports)]

#[path = "ast_facts/projection.rs"]
mod projection;
#[path = "ast_facts/summary.rs"]
mod summary;

#[allow(unused_imports)]
pub use projection::assert_core_ast_fact_projection;
#[allow(unused_imports)]
pub use summary::{assert_ast_summary_counts, AstSummaryCounts};
