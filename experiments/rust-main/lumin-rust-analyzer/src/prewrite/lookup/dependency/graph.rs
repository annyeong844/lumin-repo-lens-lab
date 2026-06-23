use lumin_rust_source_health::protocol::HealthResponse;

mod completeness;
mod roots;

use completeness::partial_import_graph_reason;
use roots::rust_path_root;

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
