use crate::locations::LineIndex;
use crate::protocol::{
    AstDefinition, AstDefinitionKind, AstFacts, AstUseTree, AstVisibility, Facts,
};

use super::location::ast_location;
use ra_ap_syntax::{
    ast::{self, HasName, HasVisibility},
    AstNode, SyntaxKind, SyntaxNode,
};

pub(super) fn counted_item_cast<T>(
    node: &SyntaxNode,
    facts: &mut Facts,
    cast: impl FnOnce(SyntaxNode) -> Option<T>,
) -> Option<T> {
    facts.items += 1;
    cast(node.clone())
}

pub(super) fn collect_definition<T>(
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

pub(super) fn collect_use_tree_facts(
    use_trees: &mut Vec<AstUseTree>,
    use_tree: &ast::UseTree,
    visibility: AstVisibility,
    line_index: &LineIndex,
) {
    use_trees.push(AstUseTree {
        tree: syntax_text(use_tree.syntax()),
        path: use_tree.path().map(|path| syntax_text(path.syntax())),
        glob: use_tree.star_token().is_some(),
        visibility,
        location: ast_location(line_index, use_tree.syntax().text_range()),
    });

    if let Some(list) = use_tree.use_tree_list() {
        for child in list.use_trees() {
            collect_use_tree_facts(use_trees, &child, visibility, line_index);
        }
    }
}

pub(super) fn sort_ast_facts(facts: &mut AstFacts) {
    facts.definitions.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.kind.cmp(&right.kind))
            .then(left.name.cmp(&right.name))
    });
    for shape in &mut facts.shape_hashes {
        shape.fields.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then(left.kind.cmp(&right.kind))
                .then(left.type_text.cmp(&right.type_text))
                .then(left.visibility.cmp(&right.visibility))
        });
    }
    facts.shape_hashes.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.name.cmp(&right.name))
            .then(left.hash.cmp(&right.hash))
    });
    for impl_block in &mut facts.impls {
        impl_block.methods.sort_by(|left, right| {
            left.location
                .byte_start
                .cmp(&right.location.byte_start)
                .then(left.name.cmp(&right.name))
        });
    }
    facts.impls.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.target.cmp(&right.target))
            .then(left.trait_path.cmp(&right.trait_path))
    });
    facts.use_trees.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.tree.cmp(&right.tree))
    });
    facts.path_refs.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.path.cmp(&right.path))
    });
    facts.method_calls.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.method.cmp(&right.method))
    });
    facts.macro_calls.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.path.cmp(&right.path))
    });
    facts.cfg_gates.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.expr.cmp(&right.expr))
    });
    facts.opaque_surfaces.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.kind.cmp(&right.kind))
            .then(left.detail.cmp(&right.detail))
    });
}

pub(super) fn is_qualified_path_ref(path: &str) -> bool {
    path.contains("::")
}

pub(super) fn is_review_method_call(method: &str) -> bool {
    matches!(method, "unwrap" | "expect" | "clone")
}

pub(super) fn visibility_for(visibility: Option<ast::Visibility>) -> AstVisibility {
    let Some(visibility) = visibility else {
        return AstVisibility::Private;
    };
    let text = visibility.syntax().text().to_string();
    match text.as_str() {
        "pub" => AstVisibility::Public,
        "pub(crate)" => AstVisibility::Crate,
        value if value.starts_with("pub(") => AstVisibility::Restricted,
        _ => AstVisibility::Unknown,
    }
}

pub(super) fn macro_path_and_name(call: &ast::MacroCall) -> (String, String) {
    let path = call.path();
    let path_text = path
        .as_ref()
        .map(|path| syntax_text(path.syntax()))
        .unwrap_or_else(|| "<unknown>".to_string());
    let name = path
        .and_then(|path| path.segment())
        .and_then(|segment| segment.name_ref())
        .map(|name_ref| name_ref.text().to_string())
        .unwrap_or_else(|| path_text.clone());
    (path_text, name)
}

pub(super) fn path_terminal_name(path: &ast::Path) -> String {
    path.segment()
        .and_then(|segment| segment.name_ref())
        .map(|name_ref| name_ref.text().to_string())
        .unwrap_or_else(|| syntax_text(path.syntax()))
}

pub(super) fn syntax_text(node: &SyntaxNode) -> String {
    node.text().to_string()
}

pub(super) fn function_is_unsafe(node: &SyntaxNode) -> bool {
    node.children_with_tokens()
        .any(|child| child.kind() == SyntaxKind::UNSAFE_KW)
}

pub(super) fn is_unsafe_block_expr(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::BLOCK_EXPR
        && node
            .children_with_tokens()
            .any(|child| child.kind() == SyntaxKind::UNSAFE_KW)
}
