use std::collections::BTreeSet;

use lumin_rust_source_health::protocol::HealthResponse;

use super::model::CandidateIndex;
use functions::function_definitions;
use local::collect_local_operations;
use sort::{sort_candidates, sort_local_operations};
use symbols::{collect_definitions, collect_impl_methods, collect_use_trees};

mod functions;
mod local;
mod sort;
mod symbols;

impl<'a> CandidateIndex<'a> {
    pub(in crate::prewrite) fn from_health(response: &'a HealthResponse) -> Self {
        let mut candidates = Vec::new();
        let mut local_operations = Vec::new();
        for (file, health) in &response.files {
            let impl_method_ranges = health
                .ast
                .impls
                .iter()
                .flat_map(|impl_block| impl_block.methods.iter())
                .map(|method| (method.location.byte_start, method.location.byte_end))
                .collect::<BTreeSet<_>>();
            let functions = function_definitions(health);
            let nested_function_ranges = functions
                .iter()
                .filter(|definition| {
                    functions::nearest_function_container(definition, &functions).is_some()
                })
                .map(|definition| (definition.location.byte_start, definition.location.byte_end))
                .collect::<BTreeSet<_>>();

            collect_local_operations(file, health, &functions, &mut local_operations);
            collect_definitions(
                file,
                health,
                &impl_method_ranges,
                &nested_function_ranges,
                &mut candidates,
            );
            collect_use_trees(file, health, &mut candidates);
            collect_impl_methods(file, health, &mut candidates);
        }

        sort_candidates(&mut candidates);
        sort_local_operations(&mut local_operations);
        Self {
            candidates,
            local_operations,
        }
    }
}
