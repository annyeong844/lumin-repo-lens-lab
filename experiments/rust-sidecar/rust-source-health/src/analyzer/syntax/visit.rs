use crate::locations::LineIndex;
use crate::protocol::PathClassification;
use ra_ap_syntax::{SyntaxKind, SyntaxNode};

use super::{items, opaque_surfaces, refs, FileSyntax};

pub(super) fn collect_syntax_node(
    node: &SyntaxNode,
    line_index: &LineIndex,
    classifications: &[PathClassification],
    syntax: &mut FileSyntax,
) {
    match node.kind() {
        SyntaxKind::FN => items::collect_function(node, line_index, syntax),
        SyntaxKind::STRUCT => items::collect_struct(node, line_index, syntax),
        SyntaxKind::ENUM => items::collect_enum(node, line_index, syntax),
        SyntaxKind::TRAIT => items::collect_trait(node, line_index, syntax),
        SyntaxKind::IMPL => items::collect_impl(node, line_index, syntax),
        SyntaxKind::MODULE => items::collect_module(node, line_index, syntax),
        SyntaxKind::CONST => items::collect_const(node, line_index, syntax),
        SyntaxKind::STATIC => items::collect_static(node, line_index, syntax),
        SyntaxKind::TYPE_ALIAS => items::collect_type_alias(node, line_index, syntax),
        SyntaxKind::STMT_LIST => items::collect_inline_patterns(node, line_index, syntax),
        SyntaxKind::USE => refs::collect_use_tree(node, line_index, syntax),
        SyntaxKind::PATH_EXPR => refs::collect_path_ref(node, line_index, syntax),
        SyntaxKind::PATH_TYPE => refs::collect_type_path_ref(node, line_index, syntax),
        SyntaxKind::METHOD_CALL_EXPR => refs::collect_method_call(node, line_index, syntax),
        SyntaxKind::MACRO_CALL => {
            opaque_surfaces::collect_macro_call(node, line_index, classifications, syntax)
        }
        SyntaxKind::ATTR => {
            opaque_surfaces::collect_attr(node, line_index, classifications, syntax)
        }
        _ if items::collect_unsafe_block(node, line_index, syntax) => {}
        _ => {}
    }
}
