use lumin_rust_source_health::protocol::HealthResponse;

const LOCAL_RUST_PATH_ROOTS: &[&str] = &["Self", "crate", "self", "super", "std", "core", "alloc"];

#[derive(Default)]
pub(super) struct DependencyImportGraph {
    examples_by_root: Vec<DependencyImportObservation>,
    complete: bool,
    pub(super) partial_reason: Option<String>,
}

pub(super) struct DependencyImportObservation {
    pub(super) root: String,
    pub(super) file: String,
    pub(super) from_spec: String,
}

impl DependencyImportGraph {
    pub(super) fn from_syntax(syntax: &HealthResponse) -> Self {
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

    pub(super) fn zero_observed_is_grounded(&self) -> bool {
        self.complete && self.partial_reason.is_none()
    }

    pub(super) fn zero_observed_unavailable_reason(&self) -> &str {
        self.partial_reason
            .as_deref()
            .unwrap_or("rust-source-health import graph is incomplete")
    }

    pub(super) fn observations(&self) -> impl Iterator<Item = &DependencyImportObservation> {
        self.examples_by_root.iter()
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
