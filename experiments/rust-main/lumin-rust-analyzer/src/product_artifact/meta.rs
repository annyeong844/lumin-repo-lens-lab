use lumin_rust_cargo_oracle::{CargoCheckMode, CargoTargetDirMode};
use serde::Serialize;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub(super) enum ProductArtifactProducer {
    #[serde(rename = "lumin-rust-analyzer")]
    LuminRustAnalyzer,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ProductArtifactMode {
    RustMain,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ArtifactLane {
    RustSourceHealth,
    RustCargoOracle,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum EmbeddedLane {
    Brief,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProductArtifactMeta {
    pub(super) producer: ProductArtifactProducer,
    pub(super) mode: ProductArtifactMode,
    pub(super) generated: String,
    pub(super) input: ProductArtifactInput,
    pub(super) output: Option<String>,
    pub(super) phase_timings: ProductPhaseTimings,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProductArtifactInput {
    pub(super) root: String,
    pub(super) package_name: Option<String>,
    pub(super) features: Option<String>,
    pub(super) cargo_bin: String,
    pub(super) semantic_mode: CargoCheckMode,
    pub(super) cargo_target_dir_mode: CargoTargetDirMode,
    pub(super) cargo_target_dir: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProductPhaseTimings {
    pub(super) syntax_ms: u128,
    pub(super) semantic_ms: u128,
    pub(super) analyzer_ms: u128,
}
