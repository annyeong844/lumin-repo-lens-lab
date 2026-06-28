mod semantic;
mod syntax;

use serde::Serialize;

pub(super) use semantic::{semantic_phase_brief, SemanticPhaseBrief, SemanticPhaseCounts};
pub(super) use syntax::{syntax_phase_brief, SyntaxPhaseBrief};

#[derive(Debug, Serialize)]
pub(super) struct PhaseBriefs<'a> {
    pub(super) syntax: SyntaxPhaseBrief<'a>,
    pub(super) semantic: SemanticPhaseBrief<'a>,
}
