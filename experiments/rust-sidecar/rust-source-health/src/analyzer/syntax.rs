mod items;
mod opaque_surfaces;
mod refs;
mod visit;

use crate::locations::LineIndex;
use crate::protocol::{AstFacts, Facts, PathClassification, Signal, Thresholds};
use ra_ap_syntax::SyntaxNode;

use super::facts::sort_ast_facts;
use visit::collect_syntax_node;

#[derive(Default)]
pub(super) struct FileSyntax {
    pub(super) facts: Facts,
    pub(super) ast: AstFacts,
    pub(super) signals: Vec<Signal>,
}

pub(super) fn collect_file_syntax(
    root: &SyntaxNode,
    line_index: &LineIndex,
    thresholds: &Thresholds,
    classifications: &[PathClassification],
) -> FileSyntax {
    let mut syntax = FileSyntax::default();

    for node in root.descendants() {
        collect_syntax_node(&node, line_index, thresholds, classifications, &mut syntax);
    }

    sort_ast_facts(&mut syntax.ast);
    syntax
}
