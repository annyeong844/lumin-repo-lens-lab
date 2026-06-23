use crate::analyzer::location::ast_location;
use crate::locations::LineIndex;
use crate::protocol::{AstUseTree, AstVisibility};

use super::syntax_text;
use ra_ap_syntax::{
    ast::{self, HasName},
    AstNode,
};

pub(in crate::analyzer) fn collect_use_tree_facts(
    use_trees: &mut Vec<AstUseTree>,
    use_tree: &ast::UseTree,
    visibility: AstVisibility,
    line_index: &LineIndex,
) {
    collect_use_tree_facts_with_prefix(use_trees, use_tree, visibility, line_index, None);
}

fn collect_use_tree_facts_with_prefix(
    use_trees: &mut Vec<AstUseTree>,
    use_tree: &ast::UseTree,
    visibility: AstVisibility,
    line_index: &LineIndex,
    parent_path: Option<&str>,
) {
    let path = use_tree.path();
    let use_tree_list = use_tree.use_tree_list();
    let glob = use_tree.star_token().is_some();
    let path_text = path.as_ref().map(|path| syntax_text(path.syntax()));
    let full_path = full_use_tree_path(parent_path, path_text.as_deref());
    let anonymous_rename = use_tree
        .rename()
        .is_some_and(|rename| rename.name().is_none());
    let terminal_name = (!glob && use_tree_list.is_none() && !anonymous_rename)
        .then(|| full_path.as_deref().and_then(path_terminal_text))
        .flatten();
    let alias = (!glob && use_tree_list.is_none() && !anonymous_rename)
        .then(|| {
            use_tree
                .rename()
                .and_then(|rename| rename.name())
                .map(|name| name.text().to_string())
        })
        .flatten();
    use_trees.push(AstUseTree {
        tree: syntax_text(use_tree.syntax()),
        path: full_path.clone(),
        name: terminal_name,
        alias,
        glob,
        visibility,
        location: ast_location(line_index, use_tree.syntax().text_range()),
    });

    if let Some(list) = use_tree_list {
        for child in list.use_trees() {
            collect_use_tree_facts_with_prefix(
                use_trees,
                &child,
                visibility,
                line_index,
                full_path.as_deref().or(parent_path),
            );
        }
    }
}

fn full_use_tree_path(parent_path: Option<&str>, path: Option<&str>) -> Option<String> {
    match (parent_path, path) {
        (_, None) => parent_path.map(str::to_string),
        (None, Some(path)) => Some(path.to_string()),
        (Some(parent), Some("self")) => Some(parent.to_string()),
        (Some(parent), Some(path)) => Some(format!("{parent}::{path}")),
    }
}

fn path_terminal_text(path: &str) -> Option<String> {
    path.rsplit("::")
        .next()
        .filter(|segment| !segment.is_empty() && *segment != "self")
        .map(str::to_string)
}
