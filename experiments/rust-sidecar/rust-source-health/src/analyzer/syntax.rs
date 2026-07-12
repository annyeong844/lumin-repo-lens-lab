mod items;
mod opaque_surfaces;
mod refs;
mod visit;

use crate::locations::LineIndex;
use crate::protocol::{AstFacts, Facts, PathClassification, Signal};
use ra_ap_syntax::SyntaxNode;

use super::facts::sort_ast_facts;
use visit::collect_syntax_node;

#[derive(Default)]
pub(super) struct FileSyntax {
    pub(super) facts: Facts,
    pub(super) ast: AstFacts,
    pub(super) signals: Vec<Signal>,
    pub(super) retain_raw_name_refs: bool,
    pub(super) retain_raw_ast_lanes: bool,
}

pub(super) fn collect_file_syntax(
    root: &SyntaxNode,
    line_index: &LineIndex,
    classifications: &[PathClassification],
    retain_raw_name_refs: bool,
    retain_raw_ast_lanes: bool,
) -> FileSyntax {
    let mut syntax = FileSyntax {
        retain_raw_name_refs,
        retain_raw_ast_lanes,
        ..FileSyntax::default()
    };

    for node in root.descendants() {
        collect_syntax_node(&node, line_index, classifications, &mut syntax);
    }

    sort_ast_facts(&mut syntax.ast);
    syntax
}
