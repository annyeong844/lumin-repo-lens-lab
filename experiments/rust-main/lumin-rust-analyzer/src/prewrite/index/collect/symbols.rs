use std::collections::BTreeSet;

use lumin_rust_source_health::protocol::{
    AstDefinition, AstDefinitionKind, AstFunctionSignature, AstVisibility, FileHealth,
};

use crate::prewrite::index::model::{Candidate, CandidateLane, ImplOwner};

pub(super) fn collect_definitions<'a>(
    file: &'a str,
    health: &'a FileHealth,
    impl_method_ranges: &BTreeSet<(usize, usize)>,
    nested_function_ranges: &BTreeSet<(usize, usize)>,
    candidates: &mut Vec<Candidate<'a>>,
) {
    candidates.extend(
        health
            .ast
            .definitions
            .iter()
            .filter(|definition| {
                if definition.kind != AstDefinitionKind::Function {
                    return true;
                }
                let range = (definition.location.byte_start, definition.location.byte_end);
                !impl_method_ranges.contains(&range) && !nested_function_ranges.contains(&range)
            })
            .map(|definition| Candidate {
                lane: CandidateLane::Definition,
                file,
                name: &definition.name,
                owner: None,
                definition_kind: Some(definition.kind),
                visibility: definition.visibility,
                location: &definition.location,
                path: &health.path,
                function_signature: signature_for_definition(health, definition),
            }),
    );
}

pub(super) fn collect_use_trees<'a>(
    file: &'a str,
    health: &'a FileHealth,
    candidates: &mut Vec<Candidate<'a>>,
) {
    candidates.extend(health.ast.use_trees.iter().filter_map(|use_tree| {
        if use_tree.glob || !is_reexport_visibility(use_tree.visibility) {
            return None;
        }
        let name = use_tree.alias.as_deref().or(use_tree.name.as_deref())?;
        Some(Candidate {
            lane: CandidateLane::UseTree,
            file,
            name,
            owner: None,
            definition_kind: None,
            visibility: use_tree.visibility,
            location: &use_tree.location,
            path: &health.path,
            function_signature: None,
        })
    }));
}

pub(super) fn collect_impl_methods<'a>(
    file: &'a str,
    health: &'a FileHealth,
    candidates: &mut Vec<Candidate<'a>>,
) {
    for impl_block in &health.ast.impls {
        let owner = ImplOwner {
            target: &impl_block.target,
            trait_path: impl_block.trait_path.as_deref(),
        };
        candidates.extend(impl_block.methods.iter().map(|method| Candidate {
            lane: CandidateLane::ImplMethod,
            file,
            name: &method.name,
            owner: Some(owner),
            definition_kind: None,
            visibility: method.visibility,
            location: &method.location,
            path: &health.path,
            function_signature: None,
        }));
    }
}

fn signature_for_definition<'a>(
    health: &'a FileHealth,
    definition: &AstDefinition,
) -> Option<&'a AstFunctionSignature> {
    if definition.kind != AstDefinitionKind::Function {
        return None;
    }
    health.ast.function_signatures.iter().find(|signature| {
        signature.name == definition.name
            && signature.location.byte_start == definition.location.byte_start
            && signature.location.byte_end == definition.location.byte_end
    })
}

fn is_reexport_visibility(visibility: AstVisibility) -> bool {
    matches!(
        visibility,
        AstVisibility::Public | AstVisibility::Crate | AstVisibility::Restricted
    )
}
