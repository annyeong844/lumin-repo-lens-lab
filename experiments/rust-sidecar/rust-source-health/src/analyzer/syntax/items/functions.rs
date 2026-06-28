use crate::locations::LineIndex;
use crate::protocol::{AstCallableKind, AstDefinitionKind};
use ra_ap_syntax::{
    ast::{self, HasVisibility},
    AstNode, SyntaxKind, SyntaxNode,
};

use crate::analyzer::facts::{collect_definition, function_is_unsafe};
use crate::analyzer::location::line_span;
use crate::analyzer::syntax::FileSyntax;

use super::function_bodies::collect_function_body_fingerprint;

mod signature;

pub(in crate::analyzer::syntax::items) use self::signature::function_signature;

pub(in crate::analyzer::syntax) fn collect_function(
    node: &SyntaxNode,
    line_index: &LineIndex,
    syntax: &mut FileSyntax,
) {
    syntax.facts.items += 1;
    syntax.facts.functions += 1;
    if function_is_unsafe(node) {
        syntax.facts.unsafe_functions += 1;
    }
    let lines = line_span(line_index, node.text_range());
    syntax.facts.max_function_lines = syntax.facts.max_function_lines.max(lines);
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Function,
        ast::Fn::cast(node.clone()),
        line_index,
    );
    let Some(function) = ast::Fn::cast(node.clone()) else {
        return;
    };
    if function_is_nested_or_associated(node) {
        return;
    }
    if let Some(fingerprint) =
        collect_function_body_fingerprint(&function, AstCallableKind::Function, None, line_index)
    {
        syntax.ast.function_body_fingerprints.push(fingerprint);
    }
    if let Some(signature) = function_signature(
        &function,
        AstCallableKind::Function,
        None,
        function.visibility(),
        line_index,
    ) {
        syntax.ast.function_signatures.push(signature);
    }
}

fn function_is_nested_or_associated(node: &SyntaxNode) -> bool {
    node.ancestors().skip(1).any(|ancestor| {
        matches!(
            ancestor.kind(),
            SyntaxKind::FN | SyntaxKind::ASSOC_ITEM_LIST
        )
    })
}
