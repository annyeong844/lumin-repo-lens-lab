use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageError {
    message: String,
}

impl UsageError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for UsageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl Error for UsageError {}

pub fn usage_error(message: impl Into<String>) -> anyhow::Error {
    UsageError::new(message).into()
}

pub fn is_usage_error(error: &anyhow::Error) -> bool {
    error.downcast_ref::<UsageError>().is_some()
        || error
            .chain()
            .any(|cause| cause.downcast_ref::<UsageError>().is_some())
}
