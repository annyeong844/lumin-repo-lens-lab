use anyhow::{anyhow, Result};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use std::path::Path;

pub(super) fn source_type_for_path(file_path: &str) -> SourceType {
    SourceType::from_path(Path::new(file_path)).unwrap_or_else(|_| SourceType::ts())
}

pub(super) fn parse_program<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
) -> Result<oxc_parser::ParserReturn<'a>> {
    let first = Parser::new(allocator, source, source_type).parse();
    if first.diagnostics.is_empty() {
        return Ok(first);
    }
    if source_type.is_javascript() && !source_type.is_jsx() {
        let jsx = Parser::new(allocator, source, source_type.with_jsx(true)).parse();
        if jsx.diagnostics.is_empty() {
            return Ok(jsx);
        }
    }
    Err(anyhow!(
        "oxc-parser: {}",
        first
            .diagnostics
            .first()
            .map(|diagnostic| format!("{diagnostic:?}"))
            .unwrap_or_else(|| "syntax error".to_string())
    ))
}

pub(super) fn line_count(source: &str) -> usize {
    source.split('\n').count()
}

pub(super) fn line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

pub(super) fn line_for_span(line_starts: &[usize], span: Span) -> usize {
    let offset = span.start as usize;
    match line_starts.binary_search(&offset) {
        Ok(index) => index + 1,
        Err(index) => index,
    }
}
