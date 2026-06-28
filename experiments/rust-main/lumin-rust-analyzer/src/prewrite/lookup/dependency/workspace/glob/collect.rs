use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

mod entries;

use super::pattern::{component_contains_glob, glob_component_matches};
use super::workspace_member_root_is_excluded;
use entries::{child_directories, child_entries};

pub(super) fn collect_glob_member_manifests(
    root: &Path,
    components: &[String],
    excludes: &[Vec<String>],
    matched_member_roots: &mut usize,
    paths: &mut Vec<PathBuf>,
) -> Result<()> {
    GlobMemberCollector {
        root,
        components,
        excludes,
        matched_member_roots,
        paths,
    }
    .collect_at(0, root)
}

struct GlobMemberCollector<'a> {
    root: &'a Path,
    components: &'a [String],
    excludes: &'a [Vec<String>],
    matched_member_roots: &'a mut usize,
    paths: &'a mut Vec<PathBuf>,
}

impl GlobMemberCollector<'_> {
    fn collect_at(&mut self, index: usize, current: &Path) -> Result<()> {
        if index == self.components.len() {
            self.collect_member_root(current)?;
            return Ok(());
        }

        let component = &self.components[index];
        if component == "**" {
            if index + 1 == self.components.len() {
                self.collect_recursive_member_manifests(current)?;
                return Ok(());
            }
            self.collect_at(index + 1, current)?;
            for child in child_directories(current)? {
                self.collect_at(index, &child)?;
            }
            return Ok(());
        }

        if component_contains_glob(component) {
            for child in child_entries(current)? {
                let Some(name) = child.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                if glob_component_matches(component, name) {
                    if child.is_dir() {
                        self.collect_at(index + 1, child.path())?;
                    } else if index + 1 == self.components.len() {
                        *self.matched_member_roots += 1;
                    }
                }
            }
            return Ok(());
        }

        let next = current.join(component);
        if next.is_dir() {
            self.collect_at(index + 1, &next)?;
        }
        Ok(())
    }

    fn collect_recursive_member_manifests(&mut self, current: &Path) -> Result<()> {
        if self.collect_excluded_member_root(current) {
            return Ok(());
        }
        for child in child_directories(current)? {
            self.collect_recursive_member_candidate(&child)?;
        }
        Ok(())
    }

    fn collect_recursive_member_candidate(&mut self, current: &Path) -> Result<()> {
        if self.collect_excluded_member_root(current) {
            return Ok(());
        }
        self.collect_member_root(current)?;
        for child in child_directories(current)? {
            self.collect_recursive_member_candidate(&child)?;
        }
        Ok(())
    }

    fn collect_member_root(&mut self, current: &Path) -> Result<()> {
        *self.matched_member_roots += 1;
        if self.member_root_is_excluded(current) {
            return Ok(());
        }
        let manifest = current.join("Cargo.toml");
        if manifest.is_file() {
            self.paths.push(manifest);
            return Ok(());
        }
        bail!(
            "blocked-prewrite-dependency-manifest: workspace member directory {} does not contain Cargo.toml",
            current.display()
        );
    }

    fn member_root_is_excluded(&self, current: &Path) -> bool {
        self.excludes
            .iter()
            .any(|exclude| workspace_member_root_is_excluded(self.root, current, exclude))
    }

    fn collect_excluded_member_root(&mut self, current: &Path) -> bool {
        if !self.member_root_is_excluded(current) {
            return false;
        }
        *self.matched_member_roots += 1;
        true
    }
}
