mod normalize;

use crate::analyzer::location::ast_location;
use crate::analyzer::syntax::FileSyntax;
use crate::locations::LineIndex;
use crate::protocol::{
    AstInlinePattern, AstInlinePatternKind, RUST_INLINE_PATTERN_MAX_STATEMENTS,
    RUST_INLINE_PATTERN_NORMALIZED_VERSION,
};
use lumin_rust_common::sha256_text;
use ra_ap_syntax::{
    ast::{self, HasName},
    AstNode, SyntaxKind, SyntaxNode,
};

use normalize::normalize_statement;

pub(in crate::analyzer::syntax) fn collect_inline_patterns(
    node: &SyntaxNode,
    line_index: &LineIndex,
    syntax: &mut FileSyntax,
) {
    let Some(stmt_list) = ast::StmtList::cast(node.clone()) else {
        return;
    };
    if stmt_list.tail_expr().is_some() || has_attribute_or_cfg_surface(stmt_list.syntax()) {
        return;
    }
    let statements = stmt_list.statements().collect::<Vec<_>>();
    if statements.is_empty() || statements.len() > RUST_INLINE_PATTERN_MAX_STATEMENTS {
        return;
    }
    let mut normalized_statements = Vec::new();
    for statement in statements {
        let Some(normalized) = normalize_statement(&statement) else {
            return;
        };
        normalized_statements.push(normalized);
    }
    let normalized_pattern = format!("block {{ {} }}", normalized_statements.join(" "));
    syntax.ast.inline_patterns.push(AstInlinePattern {
        kind: AstInlinePatternKind::StatementSequence,
        pattern_hash: sha256_text(&normalized_pattern),
        normalized_pattern,
        normalized_version: RUST_INLINE_PATTERN_NORMALIZED_VERSION,
        statement_count: normalized_statements.len(),
        enclosing_function: enclosing_function_name(stmt_list.syntax()),
        location: ast_location(line_index, stmt_list.syntax().text_range()),
    });
}

fn enclosing_function_name(node: &SyntaxNode) -> String {
    node.ancestors()
        .skip(1)
        .find_map(|ancestor| ast::Fn::cast(ancestor).and_then(|function| function.name()))
        .map(|name| name.text().to_string())
        .unwrap_or_else(|| "<top-level>".to_string())
}

fn has_attribute_or_cfg_surface(node: &SyntaxNode) -> bool {
    node.descendants()
        .any(|descendant| descendant.kind() == SyntaxKind::ATTR)
}
