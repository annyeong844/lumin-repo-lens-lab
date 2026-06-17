use crate::protocol::{FileHealth, Summary};
use std::collections::BTreeMap;

pub fn summarize(files: &BTreeMap<String, FileHealth>) -> Summary {
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

        for signal in &file.signals {
            *summary.signals_by_kind.entry(signal.kind).or_insert(0) += 1;
        }
    }

    summary
}
