use crate::locations::LineIndex;
use crate::protocol::{FileHealth, ParseStatus, PathMeta, RequestFile};
use crate::signals::{apply_signal_policy, syntax_parse_error};
use anyhow::Result;
use ra_ap_syntax::{AstNode, Edition, SourceFile};

use super::path::classify_path;
use super::syntax::collect_file_syntax;

pub(super) fn analyze_file(file: &RequestFile, edition: Edition) -> Result<(String, FileHealth)> {
    let parse = SourceFile::parse(&file.text, edition);
    let source_file = parse.tree();
    let root = source_file.syntax();
    let line_index = LineIndex::new(&file.text);

    let classifications = classify_path(&file.path, &file.text);
    let mut syntax = collect_file_syntax(root, &line_index, &classifications);

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

    let health = FileHealth {
        sha256: file.sha256.clone(),
        facts: syntax.facts,
        ast: syntax.ast,
        signals: syntax.signals,
        parse: ParseStatus {
            ok: errors.is_empty(),
            errors,
        },
        path: PathMeta {
            classifications,
            suppressed: false,
        },
    };

    Ok((file.path.clone(), health))
}
