use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use toml::Value as TomlValue;

use super::declarations::find_declarations_in_scopes;
pub(super) use super::declarations::CargoDependencyDeclaration;
use super::graph::{DependencyImportGraph, DependencyImportObservation};
use super::targets::{is_package_manifest, manifest_target_files};
use super::workspace::workspace_member_manifest_paths;

pub(super) struct CargoManifest {
    scopes: Vec<CargoManifestScope>,
    workspace_dependencies: Option<toml::map::Map<String, TomlValue>>,
}

pub(super) struct CargoManifestScope {
    pub(super) manifest_path: String,
    scope_root: String,
    is_package: bool,
    target_files: BTreeSet<String>,
    pub(super) value: TomlValue,
}

impl CargoManifest {
    pub(super) fn read(root: &Path) -> Result<Self> {
        let path = root.join("Cargo.toml");
        let value = parse_manifest(&path)?;
        let workspace_dependencies = value
            .get("workspace")
            .and_then(|workspace| workspace.get("dependencies"))
            .and_then(TomlValue::as_table)
            .cloned();
        let mut scopes = vec![CargoManifestScope {
            manifest_path: "Cargo.toml".to_string(),
            scope_root: String::new(),
            is_package: is_package_manifest(&value),
            target_files: manifest_target_files(root, &path, &value),
            value: value.clone(),
        }];
        for member_manifest in workspace_member_manifest_paths(root, &value)? {
            if member_manifest == path {
                continue;
            }
            let manifest_path = relative_manifest_path(root, &member_manifest);
            let value = parse_manifest(&member_manifest)?;
            scopes.push(CargoManifestScope {
                scope_root: manifest_scope_root(&manifest_path),
                manifest_path,
                is_package: is_package_manifest(&value),
                target_files: manifest_target_files(root, &member_manifest, &value),
                value,
            });
        }
        Ok(Self {
            scopes,
            workspace_dependencies,
        })
    }

    pub(super) fn find_declarations(
        &self,
        dependency_root: &str,
    ) -> Vec<CargoDependencyDeclaration> {
        find_declarations_in_scopes(
            &self.scopes,
            self.workspace_dependencies.as_ref(),
            dependency_root,
        )
    }

    pub(super) fn observations_for_dependency<'a>(
        &self,
        graph: &'a DependencyImportGraph,
        requested_root: &str,
        declarations: &[CargoDependencyDeclaration],
    ) -> (Vec<&'a DependencyImportObservation>, Vec<String>) {
        let requested_roots = rust_code_root_candidates(requested_root);
        let mut observations = Vec::new();
        let mut misses = BTreeSet::new();
        for observation in graph.observations() {
            let scope = self.scope_for_file(&observation.file);
            if requested_roots.contains(&observation.root) {
                if declarations.is_empty() {
                    observations.push(observation);
                    continue;
                }
                let Some(scope) = scope else {
                    continue;
                };
                if declarations
                    .iter()
                    .any(|declaration| declaration.manifest_path == scope.manifest_path)
                {
                    observations.push(observation);
                } else {
                    misses.insert(scope.manifest_path.clone());
                    observations.push(observation);
                }
                continue;
            }
            if let Some(scope) = scope {
                let alias_declared_in_scope = declarations.iter().any(|declaration| {
                    declaration.manifest_path == scope.manifest_path
                        && declaration.matches_manifest_key_root(&observation.root)
                });
                if alias_declared_in_scope {
                    observations.push(observation);
                }
            }
        }
        (observations, misses.into_iter().collect())
    }

    pub(super) fn declaration_for_observations(
        &self,
        observations: &[&DependencyImportObservation],
        declarations: &[CargoDependencyDeclaration],
    ) -> Option<CargoDependencyDeclaration> {
        observations.iter().find_map(|observation| {
            let scope = self.scope_for_file(&observation.file)?;
            declarations
                .iter()
                .find(|declaration| declaration.manifest_path == scope.manifest_path)
                .cloned()
        })
    }

    fn scope_for_file(&self, file: &str) -> Option<&CargoManifestScope> {
        self.scopes
            .iter()
            .filter(|scope| scope.file_is_in_scope(file))
            .max_by_key(|scope| scope.scope_root.len())
    }
}

impl CargoManifestScope {
    fn file_is_in_scope(&self, file: &str) -> bool {
        self.target_files.contains(file)
            || (self.is_package && file_is_in_scope(file, &self.scope_root))
    }
}

fn parse_manifest(path: &Path) -> Result<TomlValue> {
    let content = fs::read_to_string(path).with_context(|| {
        format!(
            "blocked-prewrite-dependency-manifest: failed to read {}",
            path.display()
        )
    })?;
    content.parse::<TomlValue>().with_context(|| {
        format!(
            "blocked-prewrite-dependency-manifest: failed to parse {}",
            path.display()
        )
    })
}

fn relative_manifest_path(root: &Path, manifest: &Path) -> String {
    manifest
        .strip_prefix(root)
        .unwrap_or(manifest)
        .to_string_lossy()
        .replace('\\', "/")
}

fn manifest_scope_root(manifest_path: &str) -> String {
    manifest_path
        .strip_suffix("/Cargo.toml")
        .unwrap_or("")
        .to_string()
}

fn file_is_in_scope(file: &str, scope_root: &str) -> bool {
    scope_root.is_empty()
        || file == scope_root
        || file
            .strip_prefix(scope_root)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn rust_code_root_candidates(root: &str) -> BTreeSet<String> {
    BTreeSet::from([root.to_string(), root.replace('-', "_")])
}
