use serde::Serialize;

use lumin_rust_cargo_oracle::CargoCheckMode;

use super::meta::ArtifactLane;
use crate::policy::RawLaneOmitted;

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ArtifactRefs {
    syntax: ArtifactRefSyntax,
    semantic: ArtifactRefSemantic,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactRefSyntax {
    artifact: ArtifactLane,
    raw_embedded: RawLaneOmitted,
    raw_artifact: RawArtifactReference,
}

impl Default for ArtifactRefSyntax {
    fn default() -> Self {
        Self {
            artifact: ArtifactLane::RustSourceHealth,
            raw_embedded: RawLaneOmitted,
            raw_artifact: RawArtifactReference::RustSourceHealthCompatibilityCli,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactRefSemantic {
    artifact: ArtifactLane,
    raw_embedded: RawLaneOmitted,
    raw_artifact: RawArtifactReference,
    default_mode: CargoCheckMode,
    cargo_check_mode: SemanticCargoCheckModeFlag,
}

impl Default for ArtifactRefSemantic {
    fn default() -> Self {
        Self {
            artifact: ArtifactLane::RustCargoOracle,
            raw_embedded: RawLaneOmitted,
            raw_artifact: RawArtifactReference::RustCargoOracleCompatibilityCli,
            default_mode: CargoCheckMode::MetadataOnly,
            cargo_check_mode: SemanticCargoCheckModeFlag::SemanticModeCargoCheck,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
enum RawArtifactReference {
    #[serde(rename = "run rust-source-health compatibility CLI for full syntax lane evidence")]
    RustSourceHealthCompatibilityCli,
    #[serde(rename = "run rust-cargo-oracle compatibility CLI for full semantic lane evidence")]
    RustCargoOracleCompatibilityCli,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
enum SemanticCargoCheckModeFlag {
    #[serde(rename = "--semantic-mode cargo-check")]
    SemanticModeCargoCheck,
}
