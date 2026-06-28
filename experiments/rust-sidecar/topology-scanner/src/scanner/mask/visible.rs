use super::regex::{looks_like_regex_start, regex_literal_tail};

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

pub(super) fn risk_visible_lines(source: &str) -> Vec<String> {
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
        let current_index = match lines.len() {
            0 => {
                lines.push(String::new());
                0
            }
            len => len - 1,
        };
        let current = &mut lines[current_index];

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
                current.extend(std::iter::repeat_n(' ', skipped));
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
