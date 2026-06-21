use serde_json::Value;

use crate::cli::{run_sidecar, stdout_json};
use crate::request::{file, request};

pub fn analyze_file(path: &str, source: &str) -> Value {
    stdout_json(run_sidecar(request(vec![file(path, source)])))
}
