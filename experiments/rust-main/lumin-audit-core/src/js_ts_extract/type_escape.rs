use super::{
    code_shape::normalize_code_shape, line_for_span, ExportedIdentityRange, TypeEscapeRecord,
};
use lumin_rust_common::sha256_text;
use oxc_ast::ast::{
    Comment, Expression, FormalParameterRest, Program, TSAnyKeyword, TSAsExpression,
    TSIndexSignature, TSType, TSTypeAssertion, TSTypeParameter,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::{GetSpan, Span};
use std::collections::BTreeSet;
pub(super) fn collect_type_escapes(
    program: &Program<'_>,
    comments: &[Comment],
    source: &str,
    artifact_file_path: &str,
    line_starts: &[usize],
    exported_identity_ranges: &[ExportedIdentityRange],
) -> Vec<TypeEscapeRecord> {
    let mut specific = SpecificTypeEscapeVisitor::new(
        source,
        artifact_file_path,
        line_starts,
        exported_identity_ranges,
    );
    specific.visit_program(program);

    let mut explicit = ExplicitAnyVisitor::new(
        source,
        artifact_file_path,
        line_starts,
        exported_identity_ranges,
        specific.consumed_any_starts.clone(),
    );
    explicit.visit_program(program);

    let mut facts = specific.facts;
    facts.extend(explicit.facts);
    facts.extend(collect_comment_type_escapes(
        comments,
        source,
        artifact_file_path,
        line_starts,
    ));
    facts.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then_with(|| left.occurrence_key.cmp(&right.occurrence_key))
    });
    facts
}

struct SpecificTypeEscapeVisitor<'a> {
    source: &'a str,
    artifact_file_path: &'a str,
    line_starts: &'a [usize],
    exported_identity_ranges: &'a [ExportedIdentityRange],
    consumed_any_starts: BTreeSet<u32>,
    facts: Vec<TypeEscapeRecord>,
}

impl<'a> SpecificTypeEscapeVisitor<'a> {
    fn new(
        source: &'a str,
        artifact_file_path: &'a str,
        line_starts: &'a [usize],
        exported_identity_ranges: &'a [ExportedIdentityRange],
    ) -> Self {
        Self {
            source,
            artifact_file_path,
            line_starts,
            exported_identity_ranges,
            consumed_any_starts: BTreeSet::new(),
            facts: Vec::new(),
        }
    }

    fn push_fact(&mut self, span: Span, escape_kind: &'static str) {
        self.facts.push(type_escape_record(
            self.source,
            self.artifact_file_path,
            self.line_starts,
            self.exported_identity_ranges,
            span,
            escape_kind,
        ));
    }

    fn consume_any_starts(&mut self, starts: BTreeSet<u32>) {
        self.consumed_any_starts.extend(starts);
    }
}

impl<'a> Visit<'a> for SpecificTypeEscapeVisitor<'_> {
    fn visit_formal_parameter_rest(&mut self, it: &FormalParameterRest<'a>) {
        if let Some(type_annotation) = &it.type_annotation {
            let any_starts = collect_any_type_starts(&type_annotation.type_annotation);
            if !any_starts.is_empty() {
                self.consume_any_starts(any_starts);
                self.push_fact(it.span, "rest-any-args");
                return;
            }
        }
        walk::walk_formal_parameter_rest(self, it);
    }

    fn visit_ts_index_signature(&mut self, it: &TSIndexSignature<'a>) {
        let any_starts = collect_any_type_starts(&it.type_annotation.type_annotation);
        if !any_starts.is_empty() {
            self.consume_any_starts(any_starts);
            self.push_fact(it.span, "index-sig-any");
            return;
        }
        walk::walk_ts_index_signature(self, it);
    }

    fn visit_ts_type_parameter(&mut self, it: &TSTypeParameter<'a>) {
        if it.default.as_ref().is_some_and(is_any_type) {
            if let Some(default) = &it.default {
                self.consumed_any_starts.insert(default.span().start);
            }
            self.push_fact(it.span, "generic-default-any");
            return;
        }
        walk::walk_ts_type_parameter(self, it);
    }

    fn visit_ts_type_assertion(&mut self, it: &TSTypeAssertion<'a>) {
        if is_any_type(&it.type_annotation) {
            self.consumed_any_starts
                .insert(it.type_annotation.span().start);
            self.push_fact(it.span, "angle-any");
            return;
        }
        walk::walk_ts_type_assertion(self, it);
    }

    fn visit_ts_as_expression(&mut self, it: &TSAsExpression<'a>) {
        if let Expression::TSAsExpression(inner) = &it.expression {
            if is_unknown_type(&inner.type_annotation) {
                if is_any_type(&it.type_annotation) {
                    self.consumed_any_starts
                        .insert(it.type_annotation.span().start);
                }
                self.push_fact(it.span, "as-unknown-as-T");
                walk::walk_ts_as_expression(self, it);
                return;
            }
        }
        if is_any_type(&it.type_annotation) {
            self.consumed_any_starts
                .insert(it.type_annotation.span().start);
            self.push_fact(it.span, "as-any");
            walk::walk_ts_as_expression(self, it);
            return;
        }
        walk::walk_ts_as_expression(self, it);
    }
}

struct ExplicitAnyVisitor<'a> {
    source: &'a str,
    artifact_file_path: &'a str,
    line_starts: &'a [usize],
    exported_identity_ranges: &'a [ExportedIdentityRange],
    consumed_any_starts: BTreeSet<u32>,
    facts: Vec<TypeEscapeRecord>,
}

impl<'a> ExplicitAnyVisitor<'a> {
    fn new(
        source: &'a str,
        artifact_file_path: &'a str,
        line_starts: &'a [usize],
        exported_identity_ranges: &'a [ExportedIdentityRange],
        consumed_any_starts: BTreeSet<u32>,
    ) -> Self {
        Self {
            source,
            artifact_file_path,
            line_starts,
            exported_identity_ranges,
            consumed_any_starts,
            facts: Vec::new(),
        }
    }
}

impl<'a> Visit<'a> for ExplicitAnyVisitor<'_> {
    fn visit_ts_any_keyword(&mut self, it: &TSAnyKeyword) {
        if self.consumed_any_starts.contains(&it.span.start) {
            return;
        }
        self.facts.push(type_escape_record(
            self.source,
            self.artifact_file_path,
            self.line_starts,
            self.exported_identity_ranges,
            it.span,
            "explicit-any",
        ));
    }
}

struct AnyTypeStartCollector {
    starts: BTreeSet<u32>,
}

impl<'a> Visit<'a> for AnyTypeStartCollector {
    fn visit_ts_any_keyword(&mut self, it: &TSAnyKeyword) {
        self.starts.insert(it.span.start);
    }
}

fn collect_any_type_starts(ty: &TSType<'_>) -> BTreeSet<u32> {
    let mut collector = AnyTypeStartCollector {
        starts: BTreeSet::new(),
    };
    collector.visit_ts_type(ty);
    collector.starts
}

fn is_any_type(ty: &TSType<'_>) -> bool {
    matches!(ty, TSType::TSAnyKeyword(_))
}

fn is_unknown_type(ty: &TSType<'_>) -> bool {
    matches!(ty, TSType::TSUnknownKeyword(_))
}

fn collect_comment_type_escapes(
    comments: &[Comment],
    source: &str,
    artifact_file_path: &str,
    line_starts: &[usize],
) -> Vec<TypeEscapeRecord> {
    let mut facts = Vec::new();
    for comment in comments {
        let value = source_slice_span(source, comment.content_span());
        let escape_kind = if comment.is_line() {
            line_comment_escape_kind(value)
        } else {
            block_comment_escape_kind(value)
        };
        let Some(escape_kind) = escape_kind else {
            continue;
        };
        let code_shape = source_slice_span(source, comment.span).to_string();
        let normalized_code_shape = normalize_code_shape(&code_shape);
        let occurrence_key = type_escape_occurrence_key(
            artifact_file_path,
            escape_kind,
            &normalized_code_shape,
            None,
        );
        facts.push(TypeEscapeRecord {
            file: artifact_file_path.to_string(),
            line: line_for_span(line_starts, comment.span),
            escape_kind: escape_kind.to_string(),
            code_shape,
            normalized_code_shape,
            inside_exported_identity: None,
            occurrence_key,
        });
    }
    facts
}

fn line_comment_escape_kind(value: &str) -> Option<&'static str> {
    let trimmed = value.trim_start();
    if starts_with_directive(trimmed, "@ts-ignore") {
        return Some("ts-ignore");
    }
    if starts_with_directive(trimmed, "@ts-expect-error") {
        return Some("ts-expect-error");
    }
    eslint_no_explicit_any(trimmed).then_some("no-explicit-any-disable")
}

fn block_comment_escape_kind(value: &str) -> Option<&'static str> {
    let trimmed = value.trim_start();
    if eslint_no_explicit_any(trimmed) {
        return Some("no-explicit-any-disable");
    }
    jsdoc_any(value).then_some("jsdoc-any")
}

fn starts_with_directive(value: &str, directive: &str) -> bool {
    let Some(rest) = value.strip_prefix(directive) else {
        return false;
    };
    rest.chars()
        .next()
        .is_none_or(|c| !matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-'))
}

fn eslint_no_explicit_any(value: &str) -> bool {
    value.starts_with("eslint-disable") && value.contains("no-explicit-any")
}

fn jsdoc_any(value: &str) -> bool {
    value
        .lines()
        .map(|line| line.trim_start().trim_start_matches('*').trim_start())
        .any(|line| {
            [
                "@type",
                "@param",
                "@return",
                "@returns",
                "@typedef",
                "@property",
            ]
            .iter()
            .any(|directive| starts_with_directive(line, directive) && contains_braced_any(line))
        })
}

fn contains_braced_any(value: &str) -> bool {
    let mut rest = value;
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('}') else {
            return false;
        };
        if rest[..close].trim() == "any" {
            return true;
        }
        rest = &rest[close + 1..];
    }
    false
}

fn type_escape_record(
    source: &str,
    artifact_file_path: &str,
    line_starts: &[usize],
    exported_identity_ranges: &[ExportedIdentityRange],
    span: Span,
    escape_kind: &'static str,
) -> TypeEscapeRecord {
    let code_shape = source_slice_span(source, span).to_string();
    let normalized_code_shape = normalize_code_shape(&code_shape);
    let inside_exported_identity = inside_exported_identity(exported_identity_ranges, span);
    let occurrence_key = type_escape_occurrence_key(
        artifact_file_path,
        escape_kind,
        &normalized_code_shape,
        inside_exported_identity.as_deref(),
    );
    TypeEscapeRecord {
        file: artifact_file_path.to_string(),
        line: line_for_span(line_starts, span),
        escape_kind: escape_kind.to_string(),
        code_shape,
        normalized_code_shape,
        inside_exported_identity,
        occurrence_key,
    }
}

fn inside_exported_identity(ranges: &[ExportedIdentityRange], span: Span) -> Option<String> {
    ranges
        .iter()
        .filter(|range| range.start <= span.start && span.end <= range.end)
        .min_by_key(|range| range.end.saturating_sub(range.start))
        .map(|range| range.identity.clone())
}

fn type_escape_occurrence_key(
    file: &str,
    escape_kind: &str,
    normalized_code_shape: &str,
    inside_exported_identity: Option<&str>,
) -> String {
    sha256_text(&format!(
        "{}|{}|{}|{}",
        file,
        escape_kind,
        normalized_code_shape,
        inside_exported_identity.unwrap_or("<top-level>")
    ))
}

fn source_slice_span(source: &str, span: Span) -> &str {
    let start = span.start as usize;
    let end = span.end as usize;
    source.get(start..end).unwrap_or("")
}
