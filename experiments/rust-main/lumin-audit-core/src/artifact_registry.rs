use anyhow::Result;
use std::path::Path;

pub fn collect_produced_artifacts(
    _out_dir: &Path,
    _rust_analysis_usable: bool,
) -> Result<Vec<String>> {
    Ok(Vec::new())
}
