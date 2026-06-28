use super::regex::{looks_like_regex_start, regex_literal_end_chars};
use super::strings::{read_quoted_chars, read_template_chars};

fn previous_non_space_text(out: &str) -> Option<char> {
    out.chars().rev().find(|ch| !ch.is_whitespace())
}

pub(super) fn scanner_state_ambiguous_like_js(source: &str) -> bool {
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
                out.extend(std::iter::repeat_n(' ', end - index));
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
