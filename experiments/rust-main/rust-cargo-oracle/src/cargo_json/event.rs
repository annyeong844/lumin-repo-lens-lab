use crate::rustc_diagnostic::RustcDiagnostic;

use super::record::{CargoJsonReason, CargoJsonRecord};
use super::target::CargoJsonTarget;

#[derive(Clone, Copy)]
pub(crate) struct CargoJsonMessages<'a> {
    records: &'a [CargoJsonRecord],
}

impl<'a> CargoJsonMessages<'a> {
    pub(super) fn new(records: &'a [CargoJsonRecord]) -> Self {
        Self { records }
    }

    pub(crate) fn compiler_messages(self) -> impl Iterator<Item = CargoJsonEvent<'a>> + 'a {
        self.records
            .iter()
            .map(CargoJsonEvent::new)
            .filter(|event| event.reason() == Some(CargoJsonReason::CompilerMessage))
    }

    pub(crate) fn compiler_target_events(self) -> impl Iterator<Item = CargoJsonEvent<'a>> + 'a {
        self.records
            .iter()
            .map(CargoJsonEvent::new)
            .filter(|event| {
                matches!(
                    event.reason(),
                    Some(CargoJsonReason::CompilerArtifact | CargoJsonReason::CompilerMessage)
                )
            })
    }

    pub(crate) fn build_finished(self) -> Option<CargoBuildFinished> {
        self.records
            .iter()
            .map(CargoJsonEvent::new)
            .find_map(CargoBuildFinished::from_event)
    }

    #[cfg(test)]
    pub(crate) fn contains_reason(self, expected: CargoJsonReason) -> bool {
        self.records
            .iter()
            .map(CargoJsonEvent::new)
            .any(|event| event.reason() == Some(expected))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CargoJsonEvent<'a> {
    record: &'a CargoJsonRecord,
}

impl<'a> CargoJsonEvent<'a> {
    fn new(record: &'a CargoJsonRecord) -> Self {
        Self { record }
    }

    pub(super) fn reason(self) -> Option<CargoJsonReason> {
        self.record.reason()
    }

    pub(crate) fn package_id(self) -> Option<&'a str> {
        self.record.package_id()
    }

    pub(crate) fn rustc_diagnostic(self) -> Option<&'a RustcDiagnostic> {
        self.record.rustc_diagnostic()
    }

    pub(crate) fn target(self) -> Option<CargoJsonTarget<'a>> {
        self.record.target()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CargoBuildFinished {
    success: Option<bool>,
}

impl CargoBuildFinished {
    fn from_event(event: CargoJsonEvent<'_>) -> Option<Self> {
        event
            .record
            .build_finished_success()
            .map(|success| Self { success })
    }

    pub(crate) fn success(self) -> Option<bool> {
        self.success
    }
}
