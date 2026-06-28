use std::collections::BTreeSet;

use toml::Value as TomlValue;

use super::roots::{manifest_key_candidates, rust_code_root_candidates};
use super::scope::CargoManifestScope;
use table::{dependency_tables, DependencyTable};
use value::{manifest_dependency_value, manifest_package_name};

mod table;
mod value;

#[derive(Clone)]
pub(super) struct CargoDependencyDeclaration {
    pub(super) section: String,
    pub(super) manifest_path: String,
    manifest_key_roots: BTreeSet<String>,
    pub(super) manifest_key: String,
    pub(super) display_value: String,
}

impl CargoDependencyDeclaration {
    pub(super) fn matches_manifest_key_root(&self, root: &str) -> bool {
        self.manifest_key_roots.contains(root)
    }
}

pub(super) fn find_declarations_in_scopes(
    scopes: &[CargoManifestScope],
    workspace_dependencies: Option<&toml::map::Map<String, TomlValue>>,
    dependency_root: &str,
) -> Vec<CargoDependencyDeclaration> {
    let candidates = manifest_key_candidates(dependency_root);
    scopes
        .iter()
        .flat_map(|scope| {
            dependency_tables(&scope.value)
                .into_iter()
                .filter_map(|table| {
                    find_dependency_in_table(scope, table, &candidates, workspace_dependencies)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn find_dependency_in_table(
    scope: &CargoManifestScope,
    table: DependencyTable<'_>,
    candidates: &[String],
    workspace_dependencies: Option<&toml::map::Map<String, TomlValue>>,
) -> Option<CargoDependencyDeclaration> {
    table.entries.iter().find_map(|(key, value)| {
        let package_name = manifest_package_name(key, value, workspace_dependencies);
        let declared = candidates.iter().any(|candidate| candidate == key)
            || package_name
                .as_deref()
                .is_some_and(|package| candidates.iter().any(|candidate| candidate == package));
        declared.then(|| CargoDependencyDeclaration {
            section: table.section.clone(),
            manifest_path: scope.manifest_path.clone(),
            manifest_key_roots: rust_code_root_candidates(key),
            manifest_key: key.clone(),
            display_value: manifest_dependency_value(value),
        })
    })
}
