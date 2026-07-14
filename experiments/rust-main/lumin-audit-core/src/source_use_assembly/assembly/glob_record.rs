use serde_json::Value;

use super::super::glob::{expand_import_meta_glob, ImportMetaGlobExpansion};
use super::super::input::SourceUseAssemblyRecord;
use super::super::path::root_relative;
use super::super::protocol::{NamespaceUserAddition, ResolvedInternalEdge};
use super::support::{increment_branch, mark_handled, AssemblyState};
use super::terminal::push_unresolved_specifier_record;

pub(super) fn handle_import_meta_glob_record(
    state: &mut AssemblyState,
    record: SourceUseAssemblyRecord,
) {
    let record_id = record.record_id.clone();
    match expand_import_meta_glob(
        &state.root,
        &state.resolver,
        &record,
        state.import_meta_glob_cap,
    ) {
        ImportMetaGlobExpansion::Resolved { targets } => {
            increment_branch(state, "importMetaGlobResolved");
            mark_handled(state, record_id);
            let from = root_relative(&state.root, &record.consumer_file);
            let source = record.from_spec.clone();
            for target in targets {
                let to = root_relative(&state.root, &target);
                state.response.counters.total_uses += 1;
                state.response.counters.resolved_internal_uses += 1;
                state
                    .response
                    .resolved_internal_edges
                    .push(ResolvedInternalEdge {
                        from: from.clone(),
                        to: to.clone(),
                        kind: "dynamic-import-meta-glob".to_string(),
                        source: source.clone(),
                        type_only: false,
                        line: record.line,
                        sfc_language: record.sfc_language.clone(),
                    });
                if state
                    .namespace_users_seen
                    .insert((to.clone(), from.clone()))
                {
                    state.response.namespace_users.push(NamespaceUserAddition {
                        def_file: to,
                        consumer_file: from.clone(),
                    });
                }
            }
        }
        ImportMetaGlobExpansion::Unsupported { evidence } => {
            increment_branch(state, "importMetaGlobUnsupported");
            increment_branch(state, "unresolved");
            mark_handled(state, record_id);
            state.response.counters.unresolved_uses += 1;
            state.response.counters.unresolved_internal_uses += 1;
            let from_spec = record.from_spec.clone().unwrap_or_default();
            if !from_spec.is_empty() {
                state
                    .response
                    .unresolved_internal_specifiers
                    .insert(from_spec.clone());
                let mut diagnostic = record;
                diagnostic.unresolved_evidence = Some(Value::Object(evidence));
                push_unresolved_specifier_record(
                    state,
                    &diagnostic,
                    &from_spec,
                    "import-meta-glob",
                );
            }
        }
    }
}
