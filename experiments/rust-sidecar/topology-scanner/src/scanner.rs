mod dynamic;
mod edges;
mod lex;
mod mask;
mod risk;
mod statement;

use crate::protocol::{FileScanResult, RiskKind};

use mask::{risk_visible_lines, scanner_state_ambiguous_like_js};
use statement::{scan_line, starts_multiline_import_export_block};

pub fn scan_file_text(file: &str, source: &str) -> FileScanResult {
    let mut edges = Vec::new();
    let mut risk = Vec::new();
    let mut pending_statement: Option<(usize, String, String, char)> = None;
    let risk_lines = risk_visible_lines(source);

    if scanner_state_ambiguous_like_js(source) {
        risk.push(RiskKind::ScannerStateAmbiguous);
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
    risk.sort_by(|left, right| left.as_str().cmp(right.as_str()));
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
