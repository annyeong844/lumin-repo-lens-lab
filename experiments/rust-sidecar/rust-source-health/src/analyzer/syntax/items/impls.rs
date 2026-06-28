use crate::locations::LineIndex;
use crate::protocol::{
    AstCallableKind, AstFunctionBodyFingerprint, AstFunctionOwner, AstFunctionSignature,
    AstImplBlock, AstImplMethod,
};
use ra_ap_syntax::{
    ast::{self, HasName, HasVisibility},
    AstNode, SyntaxNode,
};

use super::function_bodies::collect_function_body_fingerprint;
use super::functions::function_signature;
use crate::analyzer::facts::{syntax_text, visibility_for};
use crate::analyzer::location::ast_location;
use crate::analyzer::syntax::FileSyntax;

pub(in crate::analyzer::syntax) fn collect_impl(
    node: &SyntaxNode,
    line_index: &LineIndex,
    syntax: &mut FileSyntax,
) {
    syntax.facts.items += 1;
    let Some(impl_block) = ast::Impl::cast(node.clone()) else {
        return;
    };
    let Some(target) = impl_block.self_ty() else {
        return;
    };
    let owner = AstFunctionOwner {
        target: syntax_text(target.syntax()),
        trait_path: impl_block
            .trait_()
            .map(|trait_path| syntax_text(trait_path.syntax())),
    };
    let (methods, signatures, fingerprints) =
        impl_methods_and_signatures(&impl_block, &owner, line_index);
    syntax.ast.function_signatures.extend(signatures);
    syntax.ast.function_body_fingerprints.extend(fingerprints);
    syntax.ast.impls.push(AstImplBlock {
        target: owner.target,
        trait_path: owner.trait_path,
        methods,
        location: ast_location(line_index, impl_block.syntax().text_range()),
    });
}

fn impl_methods_and_signatures(
    impl_block: &ast::Impl,
    owner: &AstFunctionOwner,
    line_index: &LineIndex,
) -> (
    Vec<AstImplMethod>,
    Vec<AstFunctionSignature>,
    Vec<AstFunctionBodyFingerprint>,
) {
    let Some(items) = impl_block.assoc_item_list() else {
        return (Vec::new(), Vec::new(), Vec::new());
    };
    let mut methods = Vec::new();
    let mut signatures = Vec::new();
    let mut fingerprints = Vec::new();
    for item in items.assoc_items() {
        match item {
            ast::AssocItem::Fn(function) => {
                let Some(name) = function.name() else {
                    continue;
                };
                if let Some(signature) = function_signature(
                    &function,
                    AstCallableKind::ImplMethod,
                    Some(owner.clone()),
                    function.visibility(),
                    line_index,
                ) {
                    signatures.push(signature);
                }
                if let Some(fingerprint) = collect_function_body_fingerprint(
                    &function,
                    AstCallableKind::ImplMethod,
                    Some(owner.clone()),
                    line_index,
                ) {
                    fingerprints.push(fingerprint);
                }
                methods.push(AstImplMethod {
                    name: name.text().to_string(),
                    visibility: visibility_for(function.visibility()),
                    has_receiver: function
                        .param_list()
                        .and_then(|params| params.self_param())
                        .is_some(),
                    location: ast_location(line_index, function.syntax().text_range()),
                });
            }
            ast::AssocItem::Const(_)
            | ast::AssocItem::MacroCall(_)
            | ast::AssocItem::TypeAlias(_) => {}
        }
    }
    (methods, signatures, fingerprints)
}
