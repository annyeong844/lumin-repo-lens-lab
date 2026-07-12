use std::fs;
use std::io::{self, Read};
use std::path::Path;

use anyhow::Result;
use lumin_rust_common::usage_error;

mod input;
mod model;
mod normalize;

pub(super) use model::{
    IntentWarning, LoadedIntent, NameDeclaration, NormalizedIntent, RefactorSource, ShapeIntent,
};

pub(super) fn load(path: &Path) -> Result<LoadedIntent> {
    let bytes = if path == Path::new("-") {
        let mut bytes = Vec::new();
        io::stdin()
            .read_to_end(&mut bytes)
            .map_err(|error| usage_error(format!("invalid --intent -: failed to read: {error}")))?;
        bytes
    } else {
        fs::read(path).map_err(|error| {
            usage_error(format!(
                "invalid --intent {}: failed to read: {error}",
                path.display()
            ))
        })?
    };
    let raw: input::RawIntent = serde_json::from_slice(&bytes)
        .map_err(|error| usage_error(format!("invalid --intent {}: {error}", path.display())))?;
    normalize::normalize(raw)
}
