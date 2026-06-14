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

fn scan_line(line: &str, line_no: usize, edges: &mut Vec<ModuleEdge>, risk: &mut Vec<String>) {
    let trimmed = line.trim_start();

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
    if !risk.is_empty() {
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
    for (index, line) in source.lines().enumerate() {
        scan_line(line, index + 1, &mut edges, &mut risk);
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
