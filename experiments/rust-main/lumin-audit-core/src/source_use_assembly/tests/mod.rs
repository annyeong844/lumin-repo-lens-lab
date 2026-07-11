use super::*;
use serde_json::json;

fn must_request(value: serde_json::Value) -> SourceUseAssemblyRequest {
    match serde_json::from_value(value) {
        Ok(request) => request,
        Err(error) => panic!("test request must deserialize: {error}"),
    }
}

fn request(records: serde_json::Value) -> SourceUseAssemblyRequest {
    must_request(json!({
        "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
        "root": "C:/repo",
        "records": records
    }))
}

fn response(request: SourceUseAssemblyRequest) -> SourceUseAssemblyResponse {
    match build_source_use_assembly_response(request) {
        Ok(response) => response,
        Err(error) => panic!("test response must build: {error}"),
    }
}

mod compact;
mod core;
mod external_generated;
mod glob;
mod namespace;
mod resolution;
