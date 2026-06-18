use crate::locations::LineIndex;
use crate::protocol::{
    Facts, FileHealth, ParseStatus, ParserRequest, PathMeta, RequestFile, Signal, SignalKind,
    SignalMuteReason, Thresholds, PARSER_EDITION, PARSER_EDITION_POLICY, PARSER_EDITION_SOURCE,
};
use crate::signals::{
    apply_signal_policy, mute_signal, review_signal, syntax_parse_error, text_size_to_usize,
};
use anyhow::{bail, Result};
use ra_ap_syntax::{ast, AstNode, Edition, SourceFile, SyntaxKind, SyntaxNode, TextRange};
use rayon::prelude::*;
use std::collections::BTreeMap;

pub fn analyze_files(
    files: &[RequestFile],
    thresholds: &Thresholds,
    parser: &ParserRequest,
) -> Result<BTreeMap<String, FileHealth>> {
    let edition = parser_edition(parser)?;
    let analyzed = files
        .par_iter()
        .map(|file| analyze_file(file, thresholds, edition))
        .collect::<Vec<_>>();

    let mut out = BTreeMap::new();
    for result in analyzed {
        let (path, health) = result?;
        out.insert(path, health);
    }
    Ok(out)
}

fn parser_edition(parser: &ParserRequest) -> Result<Edition> {
    if parser.edition_policy != PARSER_EDITION_POLICY
        || parser.edition != PARSER_EDITION
        || parser.edition_source != PARSER_EDITION_SOURCE
    {
        bail!("unsupported parser edition policy");
    }
    configured_parser_edition()
}

fn configured_parser_edition() -> Result<Edition> {
    match PARSER_EDITION {
        "2021" => Ok(Edition::Edition2021),
        value => bail!("unsupported configured parser edition {}", value),
    }
}

fn analyze_file(
    file: &RequestFile,
    thresholds: &Thresholds,
    edition: Edition,
) -> Result<(String, FileHealth)> {
    let parse = SourceFile::parse(&file.text, edition);
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
    let classifications = classify_path(&file.path);
    apply_signal_policy(&mut signals, &classifications);

    let health = FileHealth {
        sha256: file.sha256.clone(),
        facts,
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
                    signals.push(contextual_review_signal(
                        SignalKind::OversizedFunction,
                        line_index,
                        &node,
                    ));
                }
            }
            SyntaxKind::IMPL => {
                let lines = line_span(line_index, node.text_range());
                if lines > thresholds.max_impl_lines {
                    signals.push(contextual_review_signal(
                        SignalKind::OversizedImpl,
                        line_index,
                        &node,
                    ));
                }
            }
            _ if is_unsafe_block_expr(&node) => {
                facts.unsafe_blocks += 1;
                signals.push(contextual_review_signal(
                    SignalKind::UnsafeBlock,
                    line_index,
                    &node,
                ));
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
    if has_path_segment(path, "generated") || file_name(path) == "generated.rs" {
        vec!["generated".to_string()]
    } else if is_test_like_path(path) {
        vec!["test".to_string()]
    } else {
        vec!["source".to_string()]
    }
}

fn is_test_like_path(path: &str) -> bool {
    let base = file_name(path);
    if base == "tests.rs"
        || base == "test.rs"
        || base.ends_with("_test.rs")
        || base.ends_with(".test.rs")
        || base.ends_with(".spec.rs")
    {
        return true;
    }

    path.split('/').any(|segment| {
        matches!(
            segment,
            "test"
                | "tests"
                | "e2e"
                | "integration"
                | "fixtures"
                | "fixture"
                | "mocks"
                | "mock"
                | "test-support"
                | "test-utils"
                | "runtime-tests"
                | "playground"
                | "playgrounds"
                | "examples"
                | "example"
                | "benches"
                | "bench"
        ) || (segment.len() >= 4 && segment.starts_with("__") && segment.ends_with("__"))
            || segment.ends_with("-fixture")
            || segment.ends_with("-fixtures")
    })
}

fn has_path_segment(path: &str, segment: &str) -> bool {
    path.split('/').any(|part| part == segment)
}

fn file_name(path: &str) -> &str {
    path.rsplit_once('/').map_or(path, |(_, name)| name)
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

fn contextual_review_signal(kind: SignalKind, line_index: &LineIndex, node: &SyntaxNode) -> Signal {
    let mut signal = review_signal(kind, line_index, node.text_range());
    if let Some(reason) = test_context_mute_reason(node) {
        mute_signal(&mut signal, reason);
    }
    signal
}

fn test_context_mute_reason(node: &SyntaxNode) -> Option<SignalMuteReason> {
    for ancestor in node.ancestors() {
        if let Some(function) = ast::Fn::cast(ancestor.clone()) {
            if has_direct_test_attr(&function) {
                return Some(SignalMuteReason::TestAttribute);
            }
            if has_direct_cfg_test_attr(&function) {
                return Some(SignalMuteReason::CfgTest);
            }
        }
        if let Some(module) = ast::Module::cast(ancestor.clone()) {
            if has_direct_cfg_test_attr(&module) {
                return Some(SignalMuteReason::CfgTest);
            }
        }
        if let Some(impl_block) = ast::Impl::cast(ancestor) {
            if has_direct_cfg_test_attr(&impl_block) {
                return Some(SignalMuteReason::CfgTest);
            }
        }
    }
    None
}

fn has_direct_test_attr<T: ast::HasAttrs>(owner: &T) -> bool {
    owner
        .attrs()
        .any(|attr| normalized_attr_text(&attr) == "#[test]")
}

fn has_direct_cfg_test_attr<T: ast::HasAttrs>(owner: &T) -> bool {
    owner.attrs().any(|attr| {
        matches!(
            normalized_attr_text(&attr).as_str(),
            "#[cfg(test)]" | "#![cfg(test)]"
        )
    })
}

fn normalized_attr_text(attr: &ast::Attr) -> String {
    attr.syntax()
        .text()
        .to_string()
        .chars()
        .filter(|value| !value.is_whitespace())
        .collect()
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
                    "unwrap" => signals.push(contextual_review_signal(
                        SignalKind::UnwrapCall,
                        line_index,
                        &node,
                    )),
                    "expect" => signals.push(contextual_review_signal(
                        SignalKind::ExpectCall,
                        line_index,
                        &node,
                    )),
                    "clone" => signals.push(contextual_review_signal(
                        SignalKind::CloneCall,
                        line_index,
                        &node,
                    )),
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
            let Some(name_ref) = call
                .path()
                .and_then(|path| path.segment())
                .and_then(|segment| segment.name_ref())
            else {
                continue;
            };
            let name = name_ref.text();
            let name = name.as_str();
            match name {
                "panic" => signals.push(contextual_review_signal(
                    SignalKind::PanicMacro,
                    line_index,
                    &node,
                )),
                "todo" => signals.push(contextual_review_signal(
                    SignalKind::TodoMacro,
                    line_index,
                    &node,
                )),
                "unimplemented" => signals.push(contextual_review_signal(
                    SignalKind::UnimplementedMacro,
                    line_index,
                    &node,
                )),
                _ => {}
            }
        }
    }
}
