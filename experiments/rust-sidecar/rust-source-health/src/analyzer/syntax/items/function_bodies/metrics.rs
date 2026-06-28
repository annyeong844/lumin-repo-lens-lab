use ra_ap_syntax::{ast, SyntaxKind, SyntaxNode};

pub(super) fn body_statement_count(body: &ast::BlockExpr) -> usize {
    body.stmt_list()
        .map(|statements| {
            statements.statements().count() + usize::from(statements.tail_expr().is_some())
        })
        .unwrap_or(0)
}

pub(super) fn function_has_token(function: &SyntaxNode, needle: SyntaxKind) -> bool {
    function
        .children_with_tokens()
        .any(|child| child.kind() == needle)
}
