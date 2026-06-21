mod broad;
mod package;
mod two_package;

use anyhow::Result;
use std::path::Path;

pub fn write_broad_targeted_workspace(root: &Path, package_count: usize) -> Result<()> {
    broad::write_broad_targeted_workspace(root, package_count)
}

pub fn write_two_package_targeted_workspace(root: &Path) -> Result<()> {
    two_package::write_two_package_targeted_workspace(root)
}
