use crate::protocol::Facts;

use ra_ap_syntax::{SyntaxKind, SyntaxNode};

pub(in crate::analyzer) fn counted_item_cast<T>(
    node: &SyntaxNode,
    facts: &mut Facts,
    cast: impl FnOnce(SyntaxNode) -> Option<T>,
) -> Option<T> {
    facts.items += 1;
    cast(node.clone())
}

pub(in crate::analyzer) fn is_review_method_call(method: &str) -> bool {
    matches!(method, "unwrap" | "expect" | "clone")
}

pub(in crate::analyzer) fn function_is_unsafe(node: &SyntaxNode) -> bool {
    node.children_with_tokens()
        .any(|child| child.kind() == SyntaxKind::UNSAFE_KW)
}

pub(in crate::analyzer) fn is_unsafe_block_expr(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::BLOCK_EXPR
        && node
            .children_with_tokens()
            .any(|child| child.kind() == SyntaxKind::UNSAFE_KW)
}
