mod function_bodies;
mod functions;
mod impls;
mod inline_patterns;
mod normalize;
mod shapes;

use crate::locations::LineIndex;
use crate::protocol::{AstDefinitionKind, SignalKind};
use ra_ap_syntax::{ast, AstNode, SyntaxNode};

use super::FileSyntax;
use crate::analyzer::facts::{collect_definition, counted_item_cast, is_unsafe_block_expr};
use crate::analyzer::signal_policy::contextual_review_signal;

pub(super) use functions::collect_function;
pub(super) use impls::collect_impl;
pub(super) use inline_patterns::collect_inline_patterns;
pub(super) use shapes::collect_struct;

pub(super) fn collect_enum(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Enum,
        counted_item_cast(node, &mut syntax.facts, ast::Enum::cast),
        line_index,
    );
}

pub(super) fn collect_trait(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Trait,
        counted_item_cast(node, &mut syntax.facts, ast::Trait::cast),
        line_index,
    );
}

pub(super) fn collect_module(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Module,
        counted_item_cast(node, &mut syntax.facts, ast::Module::cast),
        line_index,
    );
}

pub(super) fn collect_const(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Const,
        counted_item_cast(node, &mut syntax.facts, ast::Const::cast),
        line_index,
    );
}

pub(super) fn collect_static(node: &SyntaxNode, line_index: &LineIndex, syntax: &mut FileSyntax) {
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::Static,
        counted_item_cast(node, &mut syntax.facts, ast::Static::cast),
        line_index,
    );
}

pub(super) fn collect_type_alias(
    node: &SyntaxNode,
    line_index: &LineIndex,
    syntax: &mut FileSyntax,
) {
    collect_definition(
        &mut syntax.ast.definitions,
        AstDefinitionKind::TypeAlias,
        counted_item_cast(node, &mut syntax.facts, ast::TypeAlias::cast),
        line_index,
    );
}

pub(super) fn collect_unsafe_block(
    node: &SyntaxNode,
    line_index: &LineIndex,
    syntax: &mut FileSyntax,
) -> bool {
    if !is_unsafe_block_expr(node) {
        return false;
    }
    syntax.facts.unsafe_blocks += 1;
    syntax.signals.push(contextual_review_signal(
        SignalKind::UnsafeBlock,
        line_index,
        node,
    ));
    true
}
