pub(in crate::analyzer::syntax::items) fn compact_rust_type_text(raw: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    let mut skip_space_after_punctuation = false;
    for ch in raw.chars() {
        if ch.is_whitespace() {
            if !skip_space_after_punctuation {
                pending_space = true;
            }
            continue;
        }
        if compact_type_punctuation(ch) {
            if out.ends_with(' ') {
                out.pop();
            }
            out.push(ch);
            pending_space = false;
            skip_space_after_punctuation = true;
            continue;
        }
        if pending_space && !out.is_empty() {
            out.push(' ');
        }
        out.push(ch);
        pending_space = false;
        skip_space_after_punctuation = false;
    }
    out.trim().to_string()
}

fn compact_type_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '&' | ':' | ',' | ';' | '=' | '?'
    )
}
