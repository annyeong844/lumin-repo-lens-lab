use crate::analyzer::CompactSummaryFile;
use crate::protocol::{AstOpaqueSurfaceVisibility, CompactFileHealth, FileHealth, Summary};
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
        summary.signals += file.signal_summary.total;
        summary.definitions += file.ast.counts.definitions;
        summary.shape_hashes += file.ast.counts.shape_hashes;
        summary.function_signatures += file.ast.counts.function_signatures;
        summary.function_body_fingerprints += file.ast.counts.function_body_fingerprints;
        summary.inline_patterns += file.ast.counts.inline_patterns;
        summary.impl_blocks += file.ast.counts.impl_blocks;
        summary.impl_methods += file.ast.counts.impl_methods;
        summary.use_trees += file.ast.counts.use_trees;
        summary.path_refs += file.ast.counts.path_refs;
        summary.name_refs += file.ast.counts.name_refs;
        summary.method_call_sites += file.ast.counts.method_call_sites;
        summary.method_calls += file.ast.counts.method_calls;
        summary.macro_calls += file.ast.counts.macro_calls;
        summary.cfg_gates += file.ast.counts.cfg_gates;
        summary.opaque_surfaces += file.ast.counts.opaque_surfaces;

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

        summary.review_signals += file.signal_summary.review;
        summary.muted_signals += file.signal_summary.muted;
        merge_counts(
            &mut summary.signals_by_kind,
            &file.signal_summary.signals_by_kind,
        );
        merge_counts(
            &mut summary.signals_by_visibility,
            &file.signal_summary.signals_by_visibility,
        );
        merge_counts(
            &mut summary.review_signals_by_kind,
            &file.signal_summary.review_signals_by_kind,
        );
        merge_counts(
            &mut summary.muted_signals_by_reason,
            &file.signal_summary.muted_signals_by_reason,
        );
    }

    summary
}

pub(crate) fn summarize_compact_summary_files(
    files: &BTreeMap<String, CompactSummaryFile>,
) -> Summary {
    let mut summary = Summary {
        files: files.len(),
        ..Summary::default()
    };

    for file in files.values() {
        summarize_compact_file(&file.file, &mut summary);
        summary.review_signals += file.signal_summary.review;
        summary.muted_signals += file.signal_summary.muted;
        merge_counts(
            &mut summary.signals_by_kind,
            &file.signal_summary.signals_by_kind,
        );
        merge_counts(
            &mut summary.signals_by_visibility,
            &file.signal_summary.signals_by_visibility,
        );
        merge_counts(
            &mut summary.review_signals_by_kind,
            &file.signal_summary.review_signals_by_kind,
        );
        merge_counts(
            &mut summary.muted_signals_by_reason,
            &file.signal_summary.muted_signals_by_reason,
        );
    }

    summary
}

fn summarize_compact_file(file: &CompactFileHealth, summary: &mut Summary) {
    if !file.parse.ok {
        summary.parse_error_files += 1;
    }
    summary.parse_errors += file.parse.errors.len();
    summary.functions += file.facts.functions;
    summary.unsafe_blocks += file.facts.unsafe_blocks;
    summary.unsafe_functions += file.facts.unsafe_functions;
    summary.signals += file.signal_summary.total;
    summary.definitions += file.ast_summary.definitions;
    summary.shape_hashes += file.ast_summary.shape_hashes;
    summary.function_signatures += file.ast_summary.function_signatures;
    summary.function_body_fingerprints += file.ast_summary.function_body_fingerprints;
    summary.inline_patterns += file.ast_summary.inline_patterns;
    summary.impl_blocks += file.ast_summary.impl_blocks;
    summary.impl_methods += file.ast_summary.impl_methods;
    summary.use_trees += file.ast_summary.use_trees;
    summary.path_refs += file.ast_summary.path_refs;
    summary.name_refs += file.ast_summary.name_refs;
    summary.method_call_sites += file.ast_summary.method_call_sites;
    summary.method_calls += file.ast_summary.method_calls;
    summary.macro_calls += file.ast_summary.macro_calls;
    summary.cfg_gates += file.ast_summary.cfg_gates;
    summary.opaque_surfaces += file.ast_summary.opaque_surfaces;
    summary.review_opaque_surfaces += file.ast_summary.review_opaque_surfaces;
    summary.muted_opaque_surfaces += file.ast_summary.muted_opaque_surfaces;
    merge_counts(
        &mut summary.muted_opaque_surfaces_by_reason,
        &file.ast_summary.muted_opaque_surfaces_by_reason,
    );
}

fn merge_counts<K: Ord + Copy>(
    target: &mut std::collections::BTreeMap<K, usize>,
    source: &std::collections::BTreeMap<K, usize>,
) {
    for (key, count) in source {
        *target.entry(*key).or_insert(0) += count;
    }
}
