use super::super::event::CargoJsonMessages;
use super::kind::CargoJsonRecord;
use super::raw::RawCargoJsonRecord;

#[derive(Debug, Clone, Default)]
pub(crate) struct CargoJsonStream {
    records: Vec<CargoJsonRecord>,
}

impl CargoJsonStream {
    pub(crate) fn empty() -> Self {
        Self::default()
    }

    pub(crate) fn push_json_line(&mut self, line: &str) -> serde_json::Result<()> {
        self.records
            .push(CargoJsonRecord::from_raw(serde_json::from_str::<
                RawCargoJsonRecord,
            >(line)?));
        Ok(())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub(crate) fn as_messages(&self) -> CargoJsonMessages<'_> {
        CargoJsonMessages::new(&self.records)
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.records.len()
    }
}
