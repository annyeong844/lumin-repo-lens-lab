use lumin_rust_cargo_oracle::CargoCheckMode;
use lumin_rust_source_health::protocol::{
    AstOpaqueReason, AstOpaqueSurface, AstOpaqueSurfaceVisibility, HealthResponse,
};

pub(crate) fn targeted_oracle_paths(mode: CargoCheckMode, syntax: &HealthResponse) -> Vec<String> {
    if mode != CargoCheckMode::TargetedCargoCheck {
        return Vec::new();
    }
    let mut paths = syntax
        .files
        .iter()
        .filter(|(_, file)| {
            file.ast
                .opaque_surfaces
                .iter()
                .any(opaque_surface_needs_compiler_oracle)
        })
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

fn opaque_surface_needs_compiler_oracle(surface: &AstOpaqueSurface) -> bool {
    surface.visibility == AstOpaqueSurfaceVisibility::Review
        && matches!(
            surface.reason,
            AstOpaqueReason::MacroExpansionNotEvaluated | AstOpaqueReason::CfgConditionNotEvaluated
        )
}
