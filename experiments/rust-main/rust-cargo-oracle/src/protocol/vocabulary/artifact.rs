use serde::Serialize;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum SemanticArtifactProducer {
    #[serde(rename = "rust-cargo-oracle")]
    RustCargoOracle,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum RustcCommandSource {
    #[serde(rename = "env:CARGO_BUILD_RUSTC")]
    CargoBuildRustc,
    #[serde(rename = "env:RUSTC")]
    RustcEnv,
    #[serde(rename = "default:rustc")]
    DefaultRustc,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactProfile {
    Dev,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticArtifactMode {
    SemanticOracle,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MissingInfluenceKind {
    BuildScriptRuntimeInputs,
    ProcMacroRuntimeInputs,
    IncludeStrNonRustFiles,
    GeneratedFiles,
    TargetSpecificCargoConfigExpanded,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheReusePolicy {
    NoReuseUnlessCompleteInfluenceSetIsCaptured,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheReuseReason {
    AnalysisInputSetIncompleteForCacheReuse,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheReuseSummaryStatus {
    NotReusable,
}
