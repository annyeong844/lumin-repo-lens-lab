use lumin_rust_source_health::protocol::{AstDefinition, FileHealth};

use super::functions::nearest_function_container;
use crate::prewrite::index::model::LocalOperationCandidate;
use crate::prewrite::operation::{is_local_operation_container_name, local_operation_info};

pub(super) fn collect_local_operations<'a>(
    file: &'a str,
    health: &'a FileHealth,
    functions: &[&'a AstDefinition],
    local_operations: &mut Vec<LocalOperationCandidate<'a>>,
) {
    for definition in functions {
        let Some(container) = nearest_function_container(definition, functions) else {
            continue;
        };
        if !is_local_operation_container_name(&container.name) {
            continue;
        }
        let Some(info) = local_operation_info(&definition.name) else {
            continue;
        };
        let Some(operation_family) = info.operation_family else {
            continue;
        };
        local_operations.push(LocalOperationCandidate {
            file,
            name: &definition.name,
            container_name: &container.name,
            container_kind: "function-declaration",
            operation_family,
            location: &definition.location,
            container_location: &container.location,
            path: &health.path,
        });
    }
}
