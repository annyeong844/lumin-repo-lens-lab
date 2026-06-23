use crate::protocol::AstFacts;

pub(in crate::analyzer) fn sort_ast_facts(facts: &mut AstFacts) {
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
    facts.function_signatures.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.callable_kind.cmp(&right.callable_kind))
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
