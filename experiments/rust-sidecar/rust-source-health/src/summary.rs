use crate::protocol::{AstOpaqueSurfaceVisibility, FileHealth, SignalVisibilityState, Summary};
use std::collections::BTreeMap;

pub(crate) fn summarize(files: &BTreeMap<String, FileHealth>) -> Summary {
    let mut summary = Summary {
        files: files.len(),
        ..Summary::default()
    };

    for file in files.values() {
        if !file.parse.ok {
            summary.parse_error_files += 1;
        }
        summary.parse_errors += file.parse.errors.len();
        summary.functions += file.facts.functions;
        summary.unsafe_blocks += file.facts.unsafe_blocks;
        summary.unsafe_functions += file.facts.unsafe_functions;
        summary.signals += file.signals.len();
        summary.definitions += file.ast.definitions.len();
        summary.impl_blocks += file.ast.impls.len();
        summary.impl_methods += file
            .ast
            .impls
            .iter()
            .map(|impl_block| impl_block.methods.len())
            .sum::<usize>();
        summary.use_trees += file.ast.use_trees.len();
        summary.path_refs += file.ast.path_refs.len();
        summary.method_call_sites += file.ast.method_call_counts.values().sum::<usize>();
        summary.method_calls += file.ast.method_calls.len();
        summary.macro_calls += file.ast.macro_calls.len();
        summary.cfg_gates += file.ast.cfg_gates.len();
        summary.opaque_surfaces += file.ast.opaque_surfaces.len();

        for surface in &file.ast.opaque_surfaces {
            match surface.visibility {
                AstOpaqueSurfaceVisibility::Review => summary.review_opaque_surfaces += 1,
                AstOpaqueSurfaceVisibility::Muted { mute_reason } => {
                    summary.muted_opaque_surfaces += 1;
                    *summary
                        .muted_opaque_surfaces_by_reason
                        .entry(mute_reason)
                        .or_insert(0) += 1;
                }
            }
        }

        for signal in &file.signals {
            *summary.signals_by_kind.entry(signal.kind).or_insert(0) += 1;
            *summary
                .signals_by_visibility
                .entry(signal.visibility.visibility())
                .or_insert(0) += 1;
            match signal.visibility {
                SignalVisibilityState::Review => {
                    summary.review_signals += 1;
                    *summary
                        .review_signals_by_kind
                        .entry(signal.kind)
                        .or_insert(0) += 1;
                }
                SignalVisibilityState::Muted { mute_reason } => {
                    summary.muted_signals += 1;
                    *summary
                        .muted_signals_by_reason
                        .entry(mute_reason)
                        .or_insert(0) += 1;
                }
            }
        }
    }

    summary
}
