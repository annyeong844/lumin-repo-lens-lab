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
            raw_artifact: RawArtifactReference::syntax_full_lane(),
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
            raw_artifact: RawArtifactReference::semantic_full_lane(),
            default_mode: CargoCheckMode::MetadataOnly,
            cargo_check_mode: SemanticCargoCheckModeFlag::SemanticModeCargoCheck,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct RawArtifactReference {
    status: RawArtifactStatus,
    cli: RawArtifactCli,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact_profile: Option<RawArtifactProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cargo_check_mode: Option<RawArtifactCargoCheckMode>,
}

impl RawArtifactReference {
    fn syntax_full_lane() -> Self {
        Self {
            status: RawArtifactStatus::AvailableViaCompatibilityCli,
            cli: RawArtifactCli::LuminRustSourceHealth,
            artifact_profile: Some(RawArtifactProfile::Full),
            cargo_check_mode: None,
        }
    }

    fn semantic_full_lane() -> Self {
        Self {
            status: RawArtifactStatus::AvailableViaCompatibilityCli,
            cli: RawArtifactCli::LuminRustCargoOracle,
            artifact_profile: None,
            cargo_check_mode: Some(RawArtifactCargoCheckMode::CargoCheck),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum RawArtifactStatus {
    AvailableViaCompatibilityCli,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum RawArtifactCli {
    LuminRustSourceHealth,
    LuminRustCargoOracle,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum RawArtifactProfile {
    Full,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum RawArtifactCargoCheckMode {
    CargoCheck,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
enum SemanticCargoCheckModeFlag {
    #[serde(rename = "--semantic-mode cargo-check")]
    SemanticModeCargoCheck,
}
