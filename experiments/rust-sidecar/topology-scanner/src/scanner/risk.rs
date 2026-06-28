use super::lex::is_ident_char;

pub(super) fn has_unsupported_angle_syntax(line: &str) -> bool {
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '<' && matches!(chars.peek(), Some(next) if next.is_ascii_alphabetic()) {
            return true;
        }
    }
    false
}

pub(super) fn contains_export_assignment_risk(line: &str) -> bool {
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
