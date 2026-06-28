use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub(super) struct ChildEntry {
    path: PathBuf,
    is_dir: bool,
}

impl ChildEntry {
    pub(super) fn file_name(&self) -> Option<&std::ffi::OsStr> {
        self.path.file_name()
    }

    pub(super) fn is_dir(&self) -> bool {
        self.is_dir
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

pub(super) fn child_entries(parent: &Path) -> Result<Vec<ChildEntry>> {
    if !parent.is_dir() {
        return Ok(Vec::new());
    }
    let mut children = Vec::new();
    for entry in read_workspace_member_directory(parent)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        children.push(ChildEntry {
            path: entry.path(),
            is_dir: file_type.is_dir(),
        });
    }
    children.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(children)
}

pub(super) fn child_directories(parent: &Path) -> Result<Vec<PathBuf>> {
    if !parent.is_dir() {
        return Ok(Vec::new());
    }
    let mut children = Vec::new();
    for entry in read_workspace_member_directory(parent)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            children.push(entry.path());
        }
    }
    children.sort();
    Ok(children)
}

fn read_workspace_member_directory(parent: &Path) -> Result<fs::ReadDir> {
    fs::read_dir(parent).with_context(|| {
        format!(
            "blocked-prewrite-dependency-manifest: failed to read workspace member directory {}",
            parent.display()
        )
    })
}
