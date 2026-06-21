use crate::protocol::{ModuleEdge, RiskKind};

use super::edges::push_dynamic_edge;
use super::lex::{is_ident_char, trim_leading_block_comments};

fn import_call_arg(source: &str, start: usize) -> Option<&str> {
    let before = source[..start].chars().next_back();
    if is_ident_char(before) {
        return None;
    }
    let mut rest = &source[start + "import".len()..];
    rest = rest.trim_start();
    rest.strip_prefix('(')
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

pub(super) fn scan_dynamic_imports(
    line: &str,
    line_no: usize,
    edges: &mut Vec<ModuleEdge>,
    risk: &mut Vec<RiskKind>,
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
            risk.push(RiskKind::TemplateDynamicImport);
            continue;
        }
        if arg.starts_with('\'') || arg.starts_with('"') {
            let quote = arg.as_bytes()[0] as char;
            if let Some(end) = arg[1..].find(quote) {
                let rest = trim_leading_block_comments(&arg[1 + end + 1..]);
                if rest.trim_start().starts_with(',') {
                    risk.push(RiskKind::DynamicImportOptions);
                } else {
                    push_dynamic_edge(edges, arg[1..1 + end].to_string(), line_no);
                }
            } else {
                risk.push(RiskKind::ScannerStateAmbiguous);
            }
            continue;
        }
        if has_top_level_comma(arg) {
            risk.push(RiskKind::DynamicImportOptions);
        } else {
            risk.push(RiskKind::NonLiteralDynamicImport);
        }
    }
}
