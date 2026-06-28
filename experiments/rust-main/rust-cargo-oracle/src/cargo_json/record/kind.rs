use crate::rustc_diagnostic::RustcDiagnostic;

use super::super::target::{CargoJsonTarget, CargoJsonTargetData};
use super::raw::RawCargoJsonRecord;
use super::reason::CargoJsonReason;

#[derive(Debug, Clone)]
pub(in crate::cargo_json) enum CargoJsonRecord {
    CompilerMessage(CargoCompilerMessage),
    CompilerArtifact(CargoCompilerArtifact),
    BuildFinished(CargoBuildFinishedRecord),
    Other,
}

impl CargoJsonRecord {
    pub(super) fn from_raw(raw: RawCargoJsonRecord) -> Self {
        match raw.reason() {
            Some(CargoJsonReason::CompilerMessage) => {
                Self::CompilerMessage(CargoCompilerMessage::from_raw(raw))
            }
            Some(CargoJsonReason::CompilerArtifact) => {
                Self::CompilerArtifact(CargoCompilerArtifact::from_raw(raw))
            }
            Some(CargoJsonReason::BuildFinished) => {
                Self::BuildFinished(CargoBuildFinishedRecord::from_raw(raw))
            }
            None => Self::Other,
        }
    }

    pub(in crate::cargo_json) fn reason(&self) -> Option<CargoJsonReason> {
        match self {
            Self::CompilerMessage(_) => Some(CargoJsonReason::CompilerMessage),
            Self::CompilerArtifact(_) => Some(CargoJsonReason::CompilerArtifact),
            Self::BuildFinished(_) => Some(CargoJsonReason::BuildFinished),
            Self::Other => None,
        }
    }

    pub(in crate::cargo_json) fn package_id(&self) -> Option<&str> {
        match self {
            Self::CompilerMessage(event) => event.package_id.as_deref(),
            Self::CompilerArtifact(event) => event.package_id.as_deref(),
            Self::BuildFinished(_) | Self::Other => None,
        }
    }

    pub(in crate::cargo_json) fn rustc_diagnostic(&self) -> Option<&RustcDiagnostic> {
        match self {
            Self::CompilerMessage(event) => event.message.as_ref(),
            Self::CompilerArtifact(_) | Self::BuildFinished(_) | Self::Other => None,
        }
    }

    pub(in crate::cargo_json) fn target(&self) -> Option<CargoJsonTarget<'_>> {
        match self {
            Self::CompilerMessage(event) => event.target.as_ref().map(CargoJsonTarget::new),
            Self::CompilerArtifact(event) => event.target.as_ref().map(CargoJsonTarget::new),
            Self::BuildFinished(_) | Self::Other => None,
        }
    }

    pub(in crate::cargo_json) fn build_finished_success(&self) -> Option<Option<bool>> {
        match self {
            Self::BuildFinished(event) => Some(event.success),
            Self::CompilerMessage(_) | Self::CompilerArtifact(_) | Self::Other => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::cargo_json) struct CargoCompilerMessage {
    package_id: Option<String>,
    message: Option<RustcDiagnostic>,
    target: Option<CargoJsonTargetData>,
}

impl CargoCompilerMessage {
    fn from_raw(raw: RawCargoJsonRecord) -> Self {
        let RawCargoJsonRecord {
            package_id,
            message,
            target,
            ..
        } = raw;
        let target = target.or_else(|| message.as_ref().and_then(|message| message.target.clone()));
        let message = message.map(|message| message.diagnostic);

        Self {
            package_id,
            message,
            target,
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::cargo_json) struct CargoCompilerArtifact {
    package_id: Option<String>,
    target: Option<CargoJsonTargetData>,
}

impl CargoCompilerArtifact {
    fn from_raw(raw: RawCargoJsonRecord) -> Self {
        Self {
            package_id: raw.package_id,
            target: raw.target,
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::cargo_json) struct CargoBuildFinishedRecord {
    success: Option<bool>,
}

impl CargoBuildFinishedRecord {
    fn from_raw(raw: RawCargoJsonRecord) -> Self {
        Self {
            success: raw.success,
        }
    }
}
