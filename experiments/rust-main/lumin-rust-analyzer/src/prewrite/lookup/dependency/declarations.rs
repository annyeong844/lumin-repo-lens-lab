use std::collections::BTreeSet;

use toml::Value as TomlValue;

use super::scope::CargoManifestScope;

const DEPENDENCY_SECTIONS: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

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
                .filter_map(|(section, table)| {
                    find_dependency_in_table(
                        scope,
                        section,
                        table,
                        &candidates,
                        workspace_dependencies,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn dependency_tables(value: &TomlValue) -> Vec<(String, &toml::map::Map<String, TomlValue>)> {
    let mut tables = Vec::new();
    for section in DEPENDENCY_SECTIONS {
        if let Some(table) = value.get(*section).and_then(TomlValue::as_table) {
            tables.push(((*section).to_string(), table));
        }
    }
    if let Some(targets) = value.get("target").and_then(TomlValue::as_table) {
        for (target, target_value) in targets {
            for section in DEPENDENCY_SECTIONS {
                if let Some(table) = target_value.get(*section).and_then(TomlValue::as_table) {
                    tables.push((format!("target.{target}.{section}"), table));
                }
            }
        }
    }
    tables
}

fn find_dependency_in_table(
    scope: &CargoManifestScope,
    section: String,
    table: &toml::map::Map<String, TomlValue>,
    candidates: &[String],
    workspace_dependencies: Option<&toml::map::Map<String, TomlValue>>,
) -> Option<CargoDependencyDeclaration> {
    table.iter().find_map(|(key, value)| {
        let package_name = manifest_package_name(key, value, workspace_dependencies);
        let declared = candidates.iter().any(|candidate| candidate == key)
            || package_name
                .as_deref()
                .is_some_and(|package| candidates.iter().any(|candidate| candidate == package));
        declared.then(|| CargoDependencyDeclaration {
            section: section.clone(),
            manifest_path: scope.manifest_path.clone(),
            manifest_key_roots: rust_code_root_candidates(key),
            manifest_key: key.clone(),
            display_value: manifest_dependency_value(value),
        })
    })
}

fn manifest_package_name(
    manifest_key: &str,
    value: &TomlValue,
    workspace_dependencies: Option<&toml::map::Map<String, TomlValue>>,
) -> Option<String> {
    if let Some(package) = value
        .as_table()
        .and_then(|table| table.get("package"))
        .and_then(TomlValue::as_str)
    {
        return Some(package.to_string());
    }
    if value
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(TomlValue::as_bool)
        == Some(true)
    {
        return workspace_dependencies
            .and_then(|dependencies| dependencies.get(manifest_key))
            .and_then(|workspace_value| {
                workspace_value
                    .as_table()
                    .and_then(|table| table.get("package"))
                    .and_then(TomlValue::as_str)
            })
            .map(str::to_string);
    }
    None
}

fn manifest_key_candidates(root: &str) -> Vec<String> {
    dedupe_candidates([
        root.to_string(),
        root.replace('_', "-"),
        root.replace('-', "_"),
    ])
}

fn rust_code_root_candidates(root: &str) -> BTreeSet<String> {
    BTreeSet::from([root.to_string(), root.replace('-', "_")])
}

fn dedupe_candidates<const N: usize>(candidates: [String; N]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter(|candidate| seen.insert(candidate.clone()))
        .collect()
}

fn manifest_dependency_value(value: &TomlValue) -> String {
    match value {
        TomlValue::String(version) => version.clone(),
        TomlValue::Table(table) => {
            if table.get("workspace").and_then(TomlValue::as_bool) == Some(true) {
                "workspace = true".to_string()
            } else if let Some(version) = table.get("version").and_then(TomlValue::as_str) {
                version.to_string()
            } else if let Some(path) = table.get("path").and_then(TomlValue::as_str) {
                format!("path = {path}")
            } else {
                "inline table".to_string()
            }
        }
        _ => "nonstandard value".to_string(),
    }
}
