use lumin_rust_cargo_oracle::CargoCheckMode;
use lumin_rust_source_health::protocol::{
    AstOpaqueReason, AstOpaqueSurface, AstOpaqueSurfaceVisibility,
};

use crate::syntax_phase::{SyntaxFile, SyntaxPhase};

pub(crate) fn targeted_oracle_paths(mode: CargoCheckMode, syntax: SyntaxPhase<'_>) -> Vec<String> {
    if mode != CargoCheckMode::TargetedCargoCheck {
        return Vec::new();
    }
    let mut paths = syntax
        .files()
        .filter(|(_, file)| syntax_file_needs_compiler_oracle(*file))
        .map(|(path, _)| path.to_string())
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

fn syntax_file_needs_compiler_oracle(file: SyntaxFile<'_>) -> bool {
    match file {
        SyntaxFile::Full(file) => file
            .ast
            .opaque_surfaces
            .iter()
            .any(opaque_surface_needs_compiler_oracle),
        SyntaxFile::Compact(file) => file.ast_summary.compiler_oracle_opaque_surfaces > 0,
    }
}

fn opaque_surface_needs_compiler_oracle(surface: &AstOpaqueSurface) -> bool {
    surface.visibility == AstOpaqueSurfaceVisibility::Review
        && matches!(
            surface.reason,
            AstOpaqueReason::MacroExpansionNotEvaluated | AstOpaqueReason::CfgConditionNotEvaluated
        )
}
