use lumin_rust_source_health::protocol::{AstDefinition, AstDefinitionKind, FileHealth, Location};

pub(super) fn function_definitions(health: &FileHealth) -> Vec<&AstDefinition> {
    health
        .ast
        .definitions
        .iter()
        .filter(|definition| definition.kind == AstDefinitionKind::Function)
        .collect()
}

pub(super) fn nearest_function_container<'a>(
    child: &AstDefinition,
    functions: &[&'a AstDefinition],
) -> Option<&'a AstDefinition> {
    functions
        .iter()
        .copied()
        .filter(|candidate| location_contains(&candidate.location, &child.location))
        .min_by_key(|candidate| {
            candidate
                .location
                .byte_end
                .saturating_sub(candidate.location.byte_start)
        })
}

fn location_contains(container: &Location, child: &Location) -> bool {
    container.byte_start < child.byte_start && child.byte_end <= container.byte_end
}
