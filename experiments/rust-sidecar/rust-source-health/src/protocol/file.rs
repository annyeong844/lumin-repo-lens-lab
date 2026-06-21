use serde::Serialize;

use super::{AstFacts, ParseStatus, PathMeta, Signal};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHealth {
    pub sha256: String,
    pub facts: Facts,
    pub ast: AstFacts,
    pub signals: Vec<Signal>,
    pub parse: ParseStatus,
    pub path: PathMeta,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Facts {
    pub items: usize,
    pub functions: usize,
    pub max_function_lines: usize,
    pub unsafe_blocks: usize,
    pub unsafe_functions: usize,
}
