#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CargoJsonReason {
    BuildFinished,
    CompilerArtifact,
    CompilerMessage,
}

impl CargoJsonReason {
    pub(super) fn from_str(value: &str) -> Option<Self> {
        match value {
            "build-finished" => Some(Self::BuildFinished),
            "compiler-artifact" => Some(Self::CompilerArtifact),
            "compiler-message" => Some(Self::CompilerMessage),
            _ => None,
        }
    }
}
