mod expansion;
mod fields;
mod lossy;
mod span;

use serde::Deserialize;

use super::model::RustcSpan;
use span::RawRustcSpan;

impl<'de> Deserialize<'de> for RustcSpan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        RawRustcSpan::deserialize(deserializer).map(Into::into)
    }
}
