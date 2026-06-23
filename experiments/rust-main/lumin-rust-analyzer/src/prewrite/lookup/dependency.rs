use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use lumin_rust_source_health::protocol::HealthResponse;
use serde::Serialize;
use toml::Value as TomlValue;

use super::super::intent::NormalizedIntent;

pub(in crate::prewrite) const DEPENDENCY_EXAMPLE_LIMIT: usize = 5;
pub(in crate::prewrite) const DEPENDENCY_WATCH_FOR_THRESHOLD: usize = 10;
const DEPENDENCY_SECTIONS: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];
const LOCAL_RUST_PATH_ROOTS: &[&str] = &["Self", "crate", "self", "super", "std", "core", "alloc"];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct DependencyLookup {
    kind: DependencyLookupKind,
    pub(in crate::prewrite) dep_name: String,
    declared_in: Option<String>,
    result: DependencyLookupResult,
    existing_imports: ExistingImports,
    citations: Vec<String>,
}

impl DependencyLookup {
    pub(in crate::prewrite) fn is_watch_for_eligible(&self) -> bool {
        self.existing_imports.count_confidence == ImportCountConfidence::Grounded
            && self
                .existing_imports
                .observed_import_count
                .is_some_and(|count| count >= DEPENDENCY_WATCH_FOR_THRESHOLD)
    }

    pub(in crate::prewrite) fn observed_import_count(&self) -> Option<usize> {
        self.existing_imports.observed_import_count
    }

    pub(in crate::prewrite) fn result(&self) -> DependencyLookupResult {
        self.result
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
enum DependencyLookupKind {
    #[serde(rename = "dependency")]
    Dependency,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(in crate::prewrite) enum DependencyLookupResult {
    #[serde(rename = "DEPENDENCY_AVAILABLE")]
    Available,
    #[serde(rename = "DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS")]
    AvailableNoObservedImports,
    #[serde(rename = "DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE")]
    AvailableImportGraphUnavailable,
    #[serde(rename = "NEW_PACKAGE")]
    NewPackage,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExistingImports {
    examples: Vec<DependencyImportExample>,
    observed_import_count: Option<usize>,
    count_confidence: ImportCountConfidence,
    unavailable_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DependencyImportExample {
    file: String,
    from_spec: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ImportCountConfidence {
    Grounded,
    SampleOnly,
    Unavailable,
}

struct CargoManifest {
    scopes: Vec<CargoManifestScope>,
    workspace_dependencies: Option<toml::map::Map<String, TomlValue>>,
}

struct CargoManifestScope {
    manifest_path: String,
    value: TomlValue,
}

struct CargoDependencyDeclaration {
    section: String,
    manifest_path: String,
    manifest_key: String,
    display_value: String,
}

#[derive(Default)]
struct DependencyImportGraph {
    examples_by_root: Vec<DependencyImportObservation>,
    complete: bool,
    partial_reason: Option<String>,
}

struct DependencyImportObservation {
    root: String,
    file: String,
    from_spec: String,
}

pub(in crate::prewrite) fn lookup_dependencies(
    intent: &NormalizedIntent,
    syntax: &HealthResponse,
    root: &Path,
) -> Result<Vec<DependencyLookup>> {
    if intent.dependencies.is_empty() {
        return Ok(Vec::new());
    }
    let manifest = CargoManifest::read(root)?;
    let graph = DependencyImportGraph::from_syntax(syntax);
    Ok(intent
        .dependencies
        .iter()
        .map(|dependency| lookup_dependency(dependency, &manifest, &graph))
        .collect())
}

fn lookup_dependency(
    dependency: &str,
    manifest: &CargoManifest,
    graph: &DependencyImportGraph,
) -> DependencyLookup {
    let requested_root = dependency_root(dependency);
    let declaration = manifest.find_dependency(&requested_root);
    let code_roots = code_root_candidates(&requested_root, declaration.as_ref());
    let observations = graph.observations_for(&code_roots);
    let observed_import_count = observations.len();
    let examples = observations
        .iter()
        .take(DEPENDENCY_EXAMPLE_LIMIT)
        .map(|observation| DependencyImportExample {
            file: observation.file.clone(),
            from_spec: observation.from_spec.clone(),
        })
        .collect::<Vec<_>>();

    let mut citations = Vec::new();
    if let Some(declaration) = &declaration {
        citations.push(format!(
            "[grounded, {}.{}['{}'] declares {dependency} as {}]",
            declaration.manifest_path,
            declaration.section,
            declaration.manifest_key,
            declaration.display_value
        ));
    } else {
        citations.push(format!(
            "[grounded, Cargo manifest scope does not declare '{dependency}' in dependency tables]"
        ));
    }

    let result = if declaration.is_none() {
        DependencyLookupResult::NewPackage
    } else if observed_import_count > 0 {
        citations.push(format!(
            "[grounded, Rust AST static import graph observed {observed_import_count} consumer(s) for '{dependency}']"
        ));
        DependencyLookupResult::Available
    } else if graph.zero_observed_is_grounded() {
        citations.push(format!(
            "[확인 불가, scan range: Rust AST import graph only; '{dependency}' may still be consumed by build scripts, cfg-gated code, generated code, runtime plugins, examples, or external cargo commands]"
        ));
        DependencyLookupResult::AvailableNoObservedImports
    } else {
        let reason = graph.zero_observed_unavailable_reason();
        citations.push(format!(
            "[확인 불가, reason: {reason}; zero observed Rust path consumers is not a grounded absence claim]"
        ));
        DependencyLookupResult::AvailableImportGraphUnavailable
    };

    let count_reason = if result == DependencyLookupResult::AvailableImportGraphUnavailable {
        Some(graph.zero_observed_unavailable_reason())
    } else {
        graph.partial_reason.as_deref()
    };
    let (observed_import_count, count_confidence, unavailable_reason) =
        existing_import_count_fields(result, observed_import_count, count_reason);

    DependencyLookup {
        kind: DependencyLookupKind::Dependency,
        dep_name: dependency.to_string(),
        declared_in: declaration
            .as_ref()
            .map(|declaration| declaration.section.clone()),
        result,
        existing_imports: ExistingImports {
            examples,
            observed_import_count,
            count_confidence,
            unavailable_reason: unavailable_reason.map(str::to_string),
        },
        citations,
    }
}

fn existing_import_count_fields(
    result: DependencyLookupResult,
    count: usize,
    partial_reason: Option<&str>,
) -> (Option<usize>, ImportCountConfidence, Option<&str>) {
    match result {
        DependencyLookupResult::Available => {
            if partial_reason.is_some() {
                (
                    Some(count),
                    ImportCountConfidence::SampleOnly,
                    partial_reason,
                )
            } else {
                (Some(count), ImportCountConfidence::Grounded, None)
            }
        }
        DependencyLookupResult::AvailableNoObservedImports => {
            (Some(0), ImportCountConfidence::Grounded, None)
        }
        DependencyLookupResult::AvailableImportGraphUnavailable => {
            (None, ImportCountConfidence::Unavailable, partial_reason)
        }
        DependencyLookupResult::NewPackage => {
            if partial_reason.is_some() {
                (
                    Some(count),
                    ImportCountConfidence::SampleOnly,
                    partial_reason,
                )
            } else {
                (Some(count), ImportCountConfidence::Grounded, None)
            }
        }
    }
}

impl CargoManifest {
    fn read(root: &Path) -> Result<Self> {
        let path = root.join("Cargo.toml");
        let value = parse_manifest(&path)?;
        let workspace_dependencies = value
            .get("workspace")
            .and_then(|workspace| workspace.get("dependencies"))
            .and_then(TomlValue::as_table)
            .cloned();
        let mut scopes = vec![CargoManifestScope {
            manifest_path: "Cargo.toml".to_string(),
            value: value.clone(),
        }];
        for member_manifest in workspace_member_manifest_paths(root, &value)? {
            if member_manifest == path {
                continue;
            }
            scopes.push(CargoManifestScope {
                manifest_path: relative_manifest_path(root, &member_manifest),
                value: parse_manifest(&member_manifest)?,
            });
        }
        Ok(Self {
            scopes,
            workspace_dependencies,
        })
    }

    fn find_dependency(&self, dependency_root: &str) -> Option<CargoDependencyDeclaration> {
        let candidates = manifest_key_candidates(dependency_root);
        self.scopes.iter().find_map(|scope| {
            dependency_tables(&scope.value)
                .into_iter()
                .find_map(|(section, table)| {
                    find_dependency_in_table(
                        &scope.manifest_path,
                        section,
                        table,
                        &candidates,
                        self.workspace_dependencies.as_ref(),
                    )
                })
        })
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

fn workspace_member_manifest_paths(root: &Path, value: &TomlValue) -> Result<Vec<PathBuf>> {
    let Some(members) = value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(TomlValue::as_array)
    else {
        return Ok(Vec::new());
    };
    let mut paths = Vec::new();
    for member in members.iter().filter_map(TomlValue::as_str) {
        if let Some(prefix) = member.strip_suffix("/*") {
            let parent = root.join(prefix);
            if !parent.is_dir() {
                continue;
            }
            for entry in fs::read_dir(&parent).with_context(|| {
                format!(
                    "blocked-prewrite-dependency-manifest: failed to read workspace member directory {}",
                    parent.display()
                )
            })? {
                let entry = entry?;
                let manifest = entry.path().join("Cargo.toml");
                if manifest.is_file() {
                    paths.push(manifest);
                }
            }
            continue;
        }
        let manifest = root.join(member).join("Cargo.toml");
        if manifest.is_file() {
            paths.push(manifest);
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn relative_manifest_path(root: &Path, manifest: &Path) -> String {
    manifest
        .strip_prefix(root)
        .unwrap_or(manifest)
        .to_string_lossy()
        .replace('\\', "/")
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
    manifest_path: &str,
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
            manifest_path: manifest_path.to_string(),
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

impl DependencyImportGraph {
    fn from_syntax(syntax: &HealthResponse) -> Self {
        let mut graph = Self {
            complete: syntax.summary.parse_error_files == 0 && syntax.skipped_files.is_empty(),
            partial_reason: partial_import_graph_reason(syntax),
            examples_by_root: Vec::new(),
        };
        for (file, health) in &syntax.files {
            for use_tree in &health.ast.use_trees {
                let path = use_tree.path.as_deref().unwrap_or(&use_tree.tree);
                graph.push(file, path);
            }
            for path_ref in &health.ast.path_refs {
                graph.push(file, &path_ref.path);
            }
            for macro_call in &health.ast.macro_calls {
                graph.push(file, &macro_call.path);
            }
            for surface in &health.ast.opaque_surfaces {
                graph.push(file, &surface.detail);
            }
        }
        graph
    }

    fn zero_observed_is_grounded(&self) -> bool {
        self.complete && self.partial_reason.is_none()
    }

    fn zero_observed_unavailable_reason(&self) -> &str {
        self.partial_reason
            .as_deref()
            .unwrap_or("rust-source-health import graph is incomplete")
    }

    fn push(&mut self, file: &str, path: &str) {
        let Some(root) = rust_path_root(path) else {
            return;
        };
        self.examples_by_root.push(DependencyImportObservation {
            root,
            file: file.to_string(),
            from_spec: path.to_string(),
        });
    }

    fn observations_for(&self, roots: &BTreeSet<String>) -> Vec<&DependencyImportObservation> {
        self.examples_by_root
            .iter()
            .filter(|observation| roots.contains(&observation.root))
            .collect()
    }
}

fn dependency_root(specifier: &str) -> String {
    specifier
        .split("::")
        .next()
        .unwrap_or(specifier)
        .split('/')
        .next()
        .unwrap_or(specifier)
        .to_string()
}

fn manifest_key_candidates(root: &str) -> Vec<String> {
    dedupe_candidates([
        root.to_string(),
        root.replace('_', "-"),
        root.replace('-', "_"),
    ])
}

fn code_root_candidates(
    requested_root: &str,
    declaration: Option<&CargoDependencyDeclaration>,
) -> BTreeSet<String> {
    let mut roots = BTreeSet::new();
    roots.insert(requested_root.to_string());
    roots.insert(requested_root.replace('-', "_"));
    if let Some(declaration) = declaration {
        roots.insert(declaration.manifest_key.clone());
        roots.insert(declaration.manifest_key.replace('-', "_"));
    }
    roots
}

fn rust_path_root(path: &str) -> Option<String> {
    let normalized = path.trim_start_matches("::");
    let root = normalized.split("::").next().unwrap_or(normalized);
    if root.is_empty() || LOCAL_RUST_PATH_ROOTS.contains(&root) {
        None
    } else {
        Some(root.to_string())
    }
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

fn partial_import_graph_reason(syntax: &HealthResponse) -> Option<String> {
    let parse_error_files = syntax.summary.parse_error_files;
    let skipped_files = syntax.skipped_files.len();
    match (parse_error_files, skipped_files) {
        (0, 0) => None,
        (parse_error_files, 0) => Some(format!(
            "rust-source-health import graph is partial: {parse_error_files} parse-error file(s)"
        )),
        (0, skipped_files) => Some(format!(
            "rust-source-health import graph is partial: {skipped_files} skipped file(s)"
        )),
        (parse_error_files, skipped_files) => Some(format!(
            "rust-source-health import graph is partial: {parse_error_files} parse-error file(s), {skipped_files} skipped file(s)"
        )),
    }
}
