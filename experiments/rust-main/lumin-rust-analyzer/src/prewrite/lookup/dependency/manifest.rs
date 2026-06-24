use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use toml::Value as TomlValue;

mod observations;

use super::declarations::find_declarations_in_scopes;
pub(super) use super::declarations::CargoDependencyDeclaration;
use super::graph::{DependencyImportGraph, DependencyImportObservation};
use super::scope::CargoManifestScope;
use super::workspace::workspace_member_manifest_paths;
use observations::{declaration_for_observations, observations_for_dependency};

pub(super) struct CargoManifest {
    scopes: Vec<CargoManifestScope>,
    workspace_dependencies: Option<toml::map::Map<String, TomlValue>>,
}

impl CargoManifest {
    pub(super) fn read(root: &Path) -> Result<Self> {
        let path = root.join("Cargo.toml");
        let value = parse_manifest(&path)?;
        let workspace_dependencies = workspace_dependencies(&value)?;
        let mut scopes = vec![CargoManifestScope::root(root, &path, &value)];
        for member_manifest in workspace_member_manifest_paths(root, &value)? {
            if member_manifest == path {
                continue;
            }
            let value = parse_manifest(&member_manifest)?;
            scopes.push(CargoManifestScope::member(root, &member_manifest, value));
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
        observations_for_dependency(&self.scopes, graph, requested_root, declarations)
    }

    pub(super) fn declaration_for_observations(
        &self,
        observations: &[&DependencyImportObservation],
        declarations: &[CargoDependencyDeclaration],
    ) -> Option<CargoDependencyDeclaration> {
        declaration_for_observations(&self.scopes, observations, declarations)
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

fn workspace_dependencies(value: &TomlValue) -> Result<Option<toml::map::Map<String, TomlValue>>> {
    let Some(workspace) = value.get("workspace") else {
        return Ok(None);
    };
    let Some(workspace) = workspace.as_table() else {
        bail!("blocked-prewrite-dependency-manifest: workspace must be a table");
    };
    let Some(dependencies) = workspace.get("dependencies") else {
        return Ok(None);
    };
    let Some(table) = dependencies.as_table() else {
        bail!("blocked-prewrite-dependency-manifest: workspace.dependencies must be a table");
    };
    Ok(Some(table.clone()))
}
