use crate::analyzer::location::ast_location;
use crate::locations::LineIndex;
use crate::protocol::{AstDefinition, AstDefinitionKind};

use super::visibility_for;
use ra_ap_syntax::{
    ast::{HasName, HasVisibility},
    AstNode,
};

pub(in crate::analyzer) fn collect_definition<T>(
    definitions: &mut Vec<AstDefinition>,
    kind: AstDefinitionKind,
    item: Option<T>,
    line_index: &LineIndex,
) where
    T: AstNode + HasName + HasVisibility,
{
    let Some(item) = item else {
        return;
    };
    let Some(name) = item.name() else {
        return;
    };
    definitions.push(AstDefinition {
        kind,
        name: name.text().to_string(),
        visibility: visibility_for(item.visibility()),
        location: ast_location(line_index, item.syntax().text_range()),
    });
}
