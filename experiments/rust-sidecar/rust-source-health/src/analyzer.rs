use crate::locations::LineIndex;
use crate::protocol::{Facts, FileHealth, ParseStatus, PathMeta, RequestFile, Signal, Thresholds};
use crate::signals::{review_signal, syntax_parse_error, text_size_to_usize};
use anyhow::Result;
use ra_ap_syntax::{ast, AstNode, Edition, SourceFile, SyntaxKind, SyntaxNode, TextRange};
use rayon::prelude::*;
use std::collections::BTreeMap;

pub fn analyze_files(
    files: &[RequestFile],
    thresholds: &Thresholds,
) -> Result<BTreeMap<String, FileHealth>> {
    let analyzed = files
        .par_iter()
        .map(|file| analyze_file(file, thresholds))
        .collect::<Vec<_>>();

    let mut out = BTreeMap::new();
    for result in analyzed {
        let (path, health) = result?;
        out.insert(path, health);
    }
    Ok(out)
}

fn analyze_file(file: &RequestFile, thresholds: &Thresholds) -> Result<(String, FileHealth)> {
    let parse = SourceFile::parse(&file.text, Edition::Edition2021);
    let source_file = parse.tree();
    let root = source_file.syntax();
    let line_index = LineIndex::new(&file.text);

    let mut signals = Vec::new();
    let facts = collect_facts_and_signals(root, &line_index, thresholds, &mut signals);

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

    signals.sort_by(|left, right| {
        left.location
            .byte_start
            .cmp(&right.location.byte_start)
            .then(left.kind.cmp(&right.kind))
    });

    let health = FileHealth {
        sha256: file.sha256.clone(),
        facts,
        signals,
        parse: ParseStatus {
            ok: errors.is_empty(),
            errors,
        },
        path: PathMeta {
            classifications: classify_path(&file.path),
            suppressed: false,
        },
    };

    Ok((file.path.clone(), health))
}

fn collect_facts_and_signals(
    root: &SyntaxNode,
    line_index: &LineIndex,
    thresholds: &Thresholds,
    signals: &mut Vec<Signal>,
) -> Facts {
    let mut facts = Facts::default();
    for node in root.descendants() {
        match node.kind() {
            SyntaxKind::FN => {
                facts.functions += 1;
                if function_is_unsafe(&node) {
                    facts.unsafe_functions += 1;
                }
                let lines = line_span(line_index, node.text_range());
                facts.max_function_lines = facts.max_function_lines.max(lines);
                if lines > thresholds.max_function_lines {
                    signals.push(review_signal(
                        "oversized-function",
                        line_index,
                        node.text_range(),
                    ));
                }
            }
            SyntaxKind::IMPL => {
                let lines = line_span(line_index, node.text_range());
                if lines > thresholds.max_impl_lines {
                    signals.push(review_signal(
                        "oversized-impl",
                        line_index,
                        node.text_range(),
                    ));
                }
            }
            _ if is_unsafe_block_expr(&node) => {
                facts.unsafe_blocks += 1;
                signals.push(review_signal("unsafe-block", line_index, node.text_range()));
            }
            _ => {}
        }
    }
    facts.items = count_items(root);
    collect_method_call_signals(root, line_index, signals);
    collect_macro_call_signals(root, line_index, signals);
    facts
}

fn classify_path(path: &str) -> Vec<String> {
    if path.contains("/generated/") || path.ends_with("generated.rs") {
        vec!["generated".to_string()]
    } else if path.contains("/tests/") || path.ends_with("_test.rs") {
        vec!["test".to_string()]
    } else {
        vec!["source".to_string()]
    }
}

fn count_items(root: &SyntaxNode) -> usize {
    root.descendants()
        .filter(|node| {
            matches!(
                node.kind(),
                SyntaxKind::FN
                    | SyntaxKind::STRUCT
                    | SyntaxKind::ENUM
                    | SyntaxKind::TRAIT
                    | SyntaxKind::IMPL
                    | SyntaxKind::MODULE
                    | SyntaxKind::CONST
                    | SyntaxKind::STATIC
                    | SyntaxKind::TYPE_ALIAS
            )
        })
        .count()
}

fn function_is_unsafe(node: &SyntaxNode) -> bool {
    node.children_with_tokens()
        .any(|child| child.kind() == SyntaxKind::UNSAFE_KW)
}

fn is_unsafe_block_expr(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::BLOCK_EXPR
        && node
            .children_with_tokens()
            .any(|child| child.kind() == SyntaxKind::UNSAFE_KW)
}

fn line_span(line_index: &LineIndex, range: TextRange) -> usize {
    let byte_start = text_size_to_usize(range.start());
    let byte_end = text_size_to_usize(range.end());
    let end_point = if byte_end > byte_start {
        byte_end - 1
    } else {
        byte_end
    };
    let start = line_index.location(byte_start, byte_start);
    let end = line_index.location(end_point, end_point);
    end.line.saturating_sub(start.line) + 1
}

fn collect_method_call_signals(
    root: &SyntaxNode,
    line_index: &LineIndex,
    signals: &mut Vec<Signal>,
) {
    for node in root.descendants() {
        if let Some(call) = ast::MethodCallExpr::cast(node.clone()) {
            if let Some(name_ref) = call.name_ref() {
                match name_ref.text().as_str() {
                    "unwrap" => {
                        signals.push(review_signal("unwrap-call", line_index, node.text_range()))
                    }
                    "expect" => {
                        signals.push(review_signal("expect-call", line_index, node.text_range()))
                    }
                    "clone" => {
                        signals.push(review_signal("clone-call", line_index, node.text_range()))
                    }
                    _ => {}
                }
            }
        }
    }
}

fn collect_macro_call_signals(
    root: &SyntaxNode,
    line_index: &LineIndex,
    signals: &mut Vec<Signal>,
) {
    for node in root.descendants() {
        if let Some(call) = ast::MacroCall::cast(node.clone()) {
            let text = call.syntax().text().to_string();
            let name = text
                .split('!')
                .next()
                .unwrap_or_default()
                .trim()
                .rsplit("::")
                .next()
                .unwrap_or_default();
            match name {
                "panic" => {
                    signals.push(review_signal("panic-macro", line_index, node.text_range()))
                }
                "todo" => signals.push(review_signal("todo-macro", line_index, node.text_range())),
                "unimplemented" => signals.push(review_signal(
                    "unimplemented-macro",
                    line_index,
                    node.text_range(),
                )),
                _ => {}
            }
        }
    }
}
