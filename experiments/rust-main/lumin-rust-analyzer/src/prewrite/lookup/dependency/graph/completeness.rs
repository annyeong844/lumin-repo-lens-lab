use lumin_rust_source_health::protocol::HealthResponse;

pub(super) fn partial_import_graph_reason(syntax: &HealthResponse) -> Option<String> {
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
