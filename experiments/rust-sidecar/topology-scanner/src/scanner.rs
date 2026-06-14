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

fn push_edge(edges: &mut Vec<ModuleEdge>, source: String, line: usize, type_only: bool, re_export: bool) {
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

fn scan_dynamic_imports(line: &str, line_no: usize, edges: &mut Vec<ModuleEdge>, risk: &mut Vec<String>) {
    let mut offset = 0;
    while let Some(found) = line[offset..].find("import") {
        let start = offset + found;
        offset = start + "import".len();
        let Some(after_paren) = import_call_arg(line, start) else {
            continue;
        };
        let arg = after_paren.trim_start();
        if arg.starts_with('`') {
            risk.push("template-dynamic-import".to_string());
            continue;
        }
        if arg.starts_with('\'') || arg.starts_with('"') {
            let quote = arg.as_bytes()[0] as char;
            if let Some(end) = arg[1..].find(quote) {
                push_dynamic_edge(edges, arg[1..1 + end].to_string(), line_no);
            } else {
                risk.push("scanner-state-ambiguous".to_string());
            }
            continue;
        }
        risk.push("non-literal-dynamic-import".to_string());
    }
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

fn scan_line(line: &str, line_no: usize, edges: &mut Vec<ModuleEdge>, risk: &mut Vec<String>) {
    let trimmed = line.trim_start();
    let starting_risk_len = risk.len();

    if trimmed.starts_with('@') || trimmed.contains("Reflect.metadata(") {
        risk.push("decorator-or-reflect".to_string());
    }
    if trimmed.contains("require.context(") {
        risk.push("require-context".to_string());
    } else if trimmed.contains("require(") {
        risk.push("require-call".to_string());
    }
    if trimmed.starts_with("import ") && trimmed.contains(" = require(") {
        risk.push("ts-import-equals".to_string());
    }
    if trimmed.starts_with("export =") {
        risk.push("ts-export-assignment".to_string());
    }
    if trimmed.contains("import.meta.glob(") {
        risk.push("import-meta-glob".to_string());
    }
    if trimmed.starts_with("declare module ") {
        risk.push("ts-ambient-module".to_string());
    }
    if has_unsupported_angle_syntax(line) {
        risk.push("unsupported-syntax".to_string());
    }
    if risk.len() > starting_risk_len {
        return;
    }

    scan_dynamic_imports(line, line_no, edges, risk);
    if risk.len() > starting_risk_len {
        return;
    }

    if trimmed.starts_with("import ") {
        let type_only = trimmed.starts_with("import type ");
        if let Some(source) = quoted_after(trimmed, " from ") {
            push_edge(edges, source, line_no, type_only, false);
        } else if let Some(source) = quoted_after(trimmed, "import ") {
            push_edge(edges, source, line_no, false, false);
        }
    }

    if trimmed.starts_with("export ") && trimmed.contains(" from ") {
        let type_only = trimmed.starts_with("export type ") || trimmed.contains("{ type ");
        if let Some(source) = quoted_after(trimmed, " from ") {
            push_edge(edges, source, line_no, type_only, true);
        }
    }
}

pub fn scan_file_text(file: &str, source: &str) -> FileScanResult {
    let mut edges = Vec::new();
    let mut risk = Vec::new();
    let mut pending_statement: Option<(usize, String)> = None;

    for (index, line) in source.lines().enumerate() {
        let line_no = index + 1;
        if let Some((start_line, mut statement)) = pending_statement.take() {
            statement.push('\n');
            statement.push_str(line);
            if line.contains(';') {
                scan_line(&statement, start_line, &mut edges, &mut risk);
            } else {
                pending_statement = Some((start_line, statement));
            }
            continue;
        }

        let trimmed = line.trim_start();
        if (trimmed.starts_with("import ") || trimmed.starts_with("export "))
            && !line.contains(';')
            && !line.contains(" from ")
        {
            pending_statement = Some((line_no, line.to_string()));
            continue;
        }

        scan_line(line, line_no, &mut edges, &mut risk);
    }

    if let Some((start_line, statement)) = pending_statement {
        scan_line(&statement, start_line, &mut edges, &mut risk);
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
