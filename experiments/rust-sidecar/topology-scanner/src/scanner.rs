use crate::protocol::{FileScanResult, ModuleEdge};

pub const POLICY_VERSION: &str = "module-edge-scanner-v1";

fn quoted_after(text: &str, needle: &str) -> Option<String> {
    let idx = text.find(needle)?;
    let rest = &text[idx + needle.len()..];
    let quote_pos = rest.find(|c| c == '\'' || c == '"')?;
    let quote = rest.as_bytes()[quote_pos] as char;
    let after = &rest[quote_pos + 1..];
    let end = after.find(quote)?;
    Some(after[..end].to_string())
}

fn push_edge(
    edges: &mut Vec<ModuleEdge>,
    source: String,
    line: usize,
    type_only: bool,
    re_export: bool,
) {
    edges.push(ModuleEdge {
        source,
        line,
        type_only,
        re_export,
        dynamic: false,
    });
}

fn push_dynamic_edge(edges: &mut Vec<ModuleEdge>, source: String, line: usize) {
    edges.push(ModuleEdge {
        source,
        line,
        type_only: false,
        re_export: false,
        dynamic: true,
    });
}

fn is_ident_char(ch: Option<char>) -> bool {
    matches!(ch, Some(c) if c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn import_call_arg(source: &str, start: usize) -> Option<&str> {
    let before = source[..start].chars().next_back();
    if is_ident_char(before) {
        return None;
    }
    let mut rest = &source[start + "import".len()..];
    rest = rest.trim_start();
    rest.strip_prefix('(')
}

fn trim_leading_block_comments(mut value: &str) -> &str {
    loop {
        value = value.trim_start();
        if !value.starts_with("/*") {
            return value;
        }
        let Some(end) = value.find("*/") else {
            return value;
        };
        value = &value[end + 2..];
    }
}

fn has_top_level_comma(value: &str) -> bool {
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;
    for ch in value.chars() {
        match ch {
            '(' => paren_depth += 1,
            ')' => {
                if paren_depth == 0 {
                    return false;
                }
                paren_depth -= 1;
            }
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => return true,
            _ => {}
        }
    }
    false
}

fn scan_dynamic_imports(
    line: &str,
    line_no: usize,
    edges: &mut Vec<ModuleEdge>,
    risk: &mut Vec<String>,
) {
    let mut offset = 0;
    while let Some(found) = line[offset..].find("import") {
        let start = offset + found;
        offset = start + "import".len();
        let Some(after_paren) = import_call_arg(line, start) else {
            continue;
        };
        let arg = trim_leading_block_comments(after_paren);
        if arg.starts_with('`') {
            risk.push("template-dynamic-import".to_string());
            continue;
        }
        if arg.starts_with('\'') || arg.starts_with('"') {
            let quote = arg.as_bytes()[0] as char;
            if let Some(end) = arg[1..].find(quote) {
                let rest = trim_leading_block_comments(&arg[1 + end + 1..]);
                if rest.trim_start().starts_with(',') {
                    risk.push("dynamic-import-options".to_string());
                } else {
                    push_dynamic_edge(edges, arg[1..1 + end].to_string(), line_no);
                }
            } else {
                risk.push("scanner-state-ambiguous".to_string());
            }
            continue;
        }
        if has_top_level_comma(arg) {
            risk.push("dynamic-import-options".to_string());
        } else {
            risk.push("non-literal-dynamic-import".to_string());
        }
    }
}

fn previous_visible_non_space(lines: &[String]) -> Option<char> {
    for line in lines.iter().rev() {
        for ch in line.chars().rev() {
            if !ch.is_whitespace() {
                return Some(ch);
            }
        }
    }
    None
}

fn looks_like_regex_start(prev: Option<char>) -> bool {
    match prev {
        None => true,
        Some(ch) => "=(:,[!&|?{};".contains(ch),
    }
}

fn regex_literal_tail(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<usize> {
    let mut probe = chars.clone();
    let mut skipped = 0usize;
    let mut escaped = false;
    let mut in_class = false;

    while let Some(ch) = probe.next() {
        if ch == '\n' || ch == '\r' {
            return None;
        }
        skipped += 1;
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '[' {
            in_class = true;
            continue;
        }
        if ch == ']' {
            in_class = false;
            continue;
        }
        if ch == '/' && !in_class {
            while matches!(probe.peek(), Some(flag) if flag.is_ascii_alphabetic()) {
                probe.next();
                skipped += 1;
            }
            for _ in 0..skipped {
                chars.next();
            }
            return Some(skipped);
        }
    }

    None
}

fn regex_literal_end_chars(chars: &[char], start: usize) -> Option<usize> {
    let mut index = start + 1;
    let mut escaped = false;
    let mut in_class = false;

    while index < chars.len() {
        let ch = chars[index];
        if ch == '\n' || ch == '\r' {
            return None;
        }
        if escaped {
            escaped = false;
            index += 1;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            index += 1;
            continue;
        }
        if ch == '[' {
            in_class = true;
            index += 1;
            continue;
        }
        if ch == ']' {
            in_class = false;
            index += 1;
            continue;
        }
        if ch == '/' && !in_class {
            index += 1;
            while matches!(chars.get(index), Some(flag) if flag.is_ascii_alphabetic()) {
                index += 1;
            }
            return Some(index);
        }
        index += 1;
    }

    None
}

fn read_quoted_chars(chars: &[char], start: usize, quote: char) -> Option<usize> {
    let mut escaped = false;
    let mut index = start + 1;
    while index < chars.len() {
        let ch = chars[index];
        if escaped {
            escaped = false;
            index += 1;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            index += 1;
            continue;
        }
        if ch == quote {
            return Some(index + 1);
        }
        index += 1;
    }
    None
}

fn read_template_chars(chars: &[char], start: usize) -> Option<usize> {
    let mut escaped = false;
    let mut index = start + 1;
    while index < chars.len() {
        let ch = chars[index];
        if escaped {
            escaped = false;
            index += 1;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            index += 1;
            continue;
        }
        if ch == '`' {
            return Some(index + 1);
        }
        index += 1;
    }
    None
}

fn previous_non_space_text(out: &str) -> Option<char> {
    out.chars().rev().find(|ch| !ch.is_whitespace())
}

fn scanner_state_ambiguous_like_js(source: &str) -> bool {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        let ch = chars[index];
        let next = chars.get(index + 1).copied();

        if ch == '/' && next == Some('/') {
            while index < chars.len() && chars[index] != '\n' {
                out.push(' ');
                index += 1;
            }
            continue;
        }

        if ch == '/' && next == Some('*') {
            let mut probe = index + 2;
            while probe + 1 < chars.len() && !(chars[probe] == '*' && chars[probe + 1] == '/') {
                probe += 1;
            }
            if probe + 1 >= chars.len() {
                return true;
            }
            for masked in &chars[index..probe + 2] {
                out.push(if *masked == '\n' || *masked == '\r' {
                    *masked
                } else {
                    ' '
                });
            }
            index = probe + 2;
            continue;
        }

        if ch == '\'' || ch == '"' {
            let Some(end) = read_quoted_chars(&chars, index, ch) else {
                return true;
            };
            out.push_str("__STR__");
            index = end;
            continue;
        }

        if ch == '`' {
            let Some(end) = read_template_chars(&chars, index) else {
                return true;
            };
            out.push_str("__TPL__");
            for masked in &chars[index..end] {
                if *masked == '\n' || *masked == '\r' {
                    out.push(*masked);
                }
            }
            index = end;
            continue;
        }

        if ch == '/' && looks_like_regex_start(previous_non_space_text(&out)) {
            if let Some(end) = regex_literal_end_chars(&chars, index) {
                out.extend(std::iter::repeat(' ').take(end - index));
                out.push_str("__REGEX__");
                index = end;
                continue;
            }
        }

        out.push(ch);
        index += 1;
    }

    false
}

fn risk_visible_lines(source: &str) -> Vec<String> {
    let mut lines = vec![String::new()];
    let mut chars = source.chars().peekable();
    let mut in_block_comment = false;
    let mut in_quote: Option<char> = None;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if ch == '\r' {
            continue;
        }
        if ch == '\n' {
            lines.push(String::new());
            escaped = false;
            continue;
        }

        let regex_start = ch == '/' && looks_like_regex_start(previous_visible_non_space(&lines));
        let current = lines
            .last_mut()
            .expect("risk_visible_lines always has one line");

        if in_block_comment {
            if ch == '*' && matches!(chars.peek(), Some('/')) {
                chars.next();
                current.push_str("  ");
                in_block_comment = false;
            } else {
                current.push(' ');
            }
            continue;
        }

        if let Some(quote) = in_quote {
            current.push(' ');
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote {
                in_quote = None;
            }
            continue;
        }

        if ch == '/' && matches!(chars.peek(), Some('/')) {
            current.push(' ');
            chars.next();
            current.push(' ');
            for rest in chars.by_ref() {
                if rest == '\n' {
                    lines.push(String::new());
                    break;
                }
                current.push(' ');
            }
            continue;
        }

        if ch == '/' && matches!(chars.peek(), Some('*')) {
            current.push(' ');
            chars.next();
            current.push(' ');
            in_block_comment = true;
            continue;
        }

        if regex_start {
            if let Some(skipped) = regex_literal_tail(&mut chars) {
                current.push(' ');
                current.extend(std::iter::repeat(' ').take(skipped));
                continue;
            }
        }

        if ch == '\'' || ch == '"' || ch == '`' {
            current.push(' ');
            in_quote = Some(ch);
            escaped = false;
            continue;
        }

        current.push(ch);
    }

    lines
}

fn has_unsupported_angle_syntax(line: &str) -> bool {
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '<' && matches!(chars.peek(), Some(next) if next.is_ascii_alphabetic()) {
            return true;
        }
    }
    false
}

fn export_specifiers_type_only(statement: &str) -> bool {
    let Some(open) = statement.find('{') else {
        return false;
    };
    let Some(close_offset) = statement[open + 1..].find('}') else {
        return false;
    };
    let body = &statement[open + 1..open + 1 + close_offset];
    let mut saw_specifier = false;
    for item in body
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        saw_specifier = true;
        if !item.starts_with("type ") {
            return false;
        }
    }
    saw_specifier
}

fn contains_export_assignment_risk(line: &str) -> bool {
    for (index, _) in line.match_indices("export") {
        let before = line[..index].chars().next_back();
        let after = line[index + "export".len()..].chars().next();
        if is_ident_char(before) || is_ident_char(after) {
            continue;
        }
        if line[index + "export".len()..].trim_start().starts_with('=') {
            return true;
        }
    }
    false
}

fn starts_multiline_import_export_block(trimmed: &str) -> bool {
    trimmed.starts_with("import {")
        || trimmed.starts_with("import type {")
        || trimmed.starts_with("export {")
        || trimmed.starts_with("export type {")
}

fn scan_line(
    line: &str,
    risk_line: &str,
    line_no: usize,
    edges: &mut Vec<ModuleEdge>,
    risk: &mut Vec<String>,
) {
    let trimmed = line.trim_start();
    let risk_trimmed = risk_line.trim_start();
    let starting_risk_len = risk.len();

    if risk_trimmed.starts_with('@') || risk_trimmed.contains("Reflect.metadata(") {
        risk.push("decorator-or-reflect".to_string());
    }
    if risk_trimmed.contains("require.context(") {
        risk.push("require-context".to_string());
    } else if risk_trimmed.contains("require(") {
        risk.push("require-call".to_string());
    }
    if risk_trimmed.starts_with("import ") && risk_trimmed.contains(" = require(") {
        risk.push("ts-import-equals".to_string());
    }
    if contains_export_assignment_risk(risk_line) {
        risk.push("ts-export-assignment".to_string());
    }
    if risk_trimmed.contains("import.meta.glob(") {
        risk.push("import-meta-glob".to_string());
    }
    if risk_trimmed.starts_with("declare module ") {
        risk.push("ts-ambient-module".to_string());
    }
    if has_unsupported_angle_syntax(risk_line) {
        risk.push("unsupported-syntax".to_string());
    }

    if risk_line.contains("import") {
        scan_dynamic_imports(line, line_no, edges, risk);
    }
    if risk.len() > starting_risk_len {
        return;
    }

    if risk_trimmed.starts_with("import ") {
        let type_only = trimmed.starts_with("import type ");
        if let Some(source) = quoted_after(trimmed, " from ") {
            push_edge(edges, source, line_no, type_only, false);
        } else if let Some(source) = quoted_after(trimmed, "import ") {
            push_edge(edges, source, line_no, false, false);
        }
    }

    if risk_trimmed.starts_with("export ") && trimmed.contains(" from ") {
        let type_only = trimmed.starts_with("export type ") || export_specifiers_type_only(trimmed);
        if let Some(source) = quoted_after(trimmed, " from ") {
            push_edge(edges, source, line_no, type_only, true);
        }
    }
}

pub fn scan_file_text(file: &str, source: &str) -> FileScanResult {
    let mut edges = Vec::new();
    let mut risk = Vec::new();
    let mut pending_statement: Option<(usize, String, String, char)> = None;
    let risk_lines = risk_visible_lines(source);

    if scanner_state_ambiguous_like_js(source) {
        risk.push("scanner-state-ambiguous".to_string());
    }

    for (index, line) in source.lines().enumerate() {
        let risk_line = risk_lines.get(index).map(String::as_str).unwrap_or("");
        let line_no = index + 1;
        if let Some((start_line, mut statement, mut risk_statement, terminator)) =
            pending_statement.take()
        {
            statement.push('\n');
            statement.push_str(line);
            risk_statement.push('\n');
            risk_statement.push_str(risk_line);
            if line.contains(terminator) || (terminator == ';' && risk_line.contains(" from ")) {
                scan_line(
                    &statement,
                    &risk_statement,
                    start_line,
                    &mut edges,
                    &mut risk,
                );
            } else {
                pending_statement = Some((start_line, statement, risk_statement, terminator));
            }
            continue;
        }

        let trimmed = line.trim_start();
        if starts_multiline_import_export_block(trimmed)
            && !line.contains(';')
            && !line.contains(" from ")
        {
            pending_statement = Some((line_no, line.to_string(), risk_line.to_string(), ';'));
            continue;
        }
        if risk_line.contains("import(") && !line.contains(')') {
            pending_statement = Some((line_no, line.to_string(), risk_line.to_string(), ')'));
            continue;
        }

        scan_line(line, risk_line, line_no, &mut edges, &mut risk);
    }

    if let Some((start_line, statement, risk_statement, _)) = pending_statement {
        scan_line(
            &statement,
            &risk_statement,
            start_line,
            &mut edges,
            &mut risk,
        );
    }
    risk.sort();
    risk.dedup();
    if !risk.is_empty() {
        edges.clear();
    }

    FileScanResult {
        file: file.to_string(),
        ok: risk.is_empty(),
        loc: source.split('\n').count(),
        edges,
        risk,
    }
}
