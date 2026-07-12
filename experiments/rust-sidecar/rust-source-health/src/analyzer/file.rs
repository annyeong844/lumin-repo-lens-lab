use crate::locations::LineIndex;
use crate::protocol::{FileHealth, FileSignalSummary, ParseStatus, PathMeta, RequestFile};
use crate::signals::{apply_signal_policy, syntax_parse_error};
use anyhow::Result;
use ra_ap_syntax::{AstNode, Edition, SourceFile};

use super::path::classify_path;
use super::syntax::collect_file_syntax;

pub(super) fn analyze_file(
    file: &RequestFile,
    edition: Edition,
    retain_raw_name_refs: bool,
    retain_raw_signals: bool,
    retain_raw_ast_lanes: bool,
) -> Result<(String, FileHealth)> {
    analyze_file_text(
        &file.path,
        &file.sha256,
        &file.text,
        edition,
        retain_raw_name_refs,
        retain_raw_signals,
        retain_raw_ast_lanes,
    )
}

pub(super) fn analyze_file_text(
    path: &str,
    sha256: &str,
    text: &str,
    edition: Edition,
    retain_raw_name_refs: bool,
    retain_raw_signals: bool,
    retain_raw_ast_lanes: bool,
) -> Result<(String, FileHealth)> {
    let parse = SourceFile::parse(text, edition);
    let source_file = parse.tree();
    let root = source_file.syntax();
    let line_index = LineIndex::new(text);

    let classifications = classify_path(path, text);
    let mut syntax = collect_file_syntax(
        root,
        &line_index,
        &classifications,
        retain_raw_name_refs,
        retain_raw_ast_lanes,
    );

    let mut errors = parse
        .errors()
        .iter()
        .map(|error| syntax_parse_error(error.to_string(), &line_index, error.range()))
        .collect::<Vec<_>>();
    errors.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.message.cmp(&right.message))
    });

    syntax.signals.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.kind.cmp(&right.kind))
    });
    apply_signal_policy(&mut syntax.signals, &classifications);
    let signal_summary = FileSignalSummary::from_signals(&syntax.signals);
    let signals = if retain_raw_signals {
        syntax.signals
    } else {
        Vec::new()
    };
    syntax.ast.refresh_counts();
    if !retain_raw_ast_lanes {
        syntax.ast.prune_raw_lanes_for_compact_source_health();
    }

    let health = FileHealth {
        sha256: sha256.to_string(),
        facts: syntax.facts,
        ast: syntax.ast,
        signal_summary,
        signals,
        parse: ParseStatus {
            ok: errors.is_empty(),
            errors,
        },
        path: PathMeta {
            classifications,
            suppressed: false,
        },
    };

    Ok((path.to_string(), health))
}
