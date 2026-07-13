#[derive(Debug, Clone, Copy)]
enum CodeShapeState {
    Code,
    Single,
    Double,
    Template,
    LineComment,
    BlockComment,
}

pub(super) fn normalize_code_shape(raw: &str) -> String {
    if raw.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let mut state = CodeShapeState::Code;
    let mut prev_space = false;
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        let next = chars.peek().copied();
        match state {
            CodeShapeState::Single => {
                out.push(c);
                if c == '\\' {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else if c == '\'' {
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::Double => {
                out.push(c);
                if c == '\\' {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else if c == '"' {
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::Template => {
                out.push(c);
                if c == '\\' {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else if c == '`' {
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::LineComment => {
                out.push(c);
                if c == '\n' {
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::BlockComment => {
                out.push(c);
                if c == '*' && next == Some('/') {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::Code => {
                if c == '\'' {
                    state = CodeShapeState::Single;
                    out.push(c);
                    prev_space = false;
                } else if c == '"' {
                    state = CodeShapeState::Double;
                    out.push(c);
                    prev_space = false;
                } else if c == '`' {
                    state = CodeShapeState::Template;
                    out.push(c);
                    prev_space = false;
                } else if c == '/' && next == Some('/') {
                    state = CodeShapeState::LineComment;
                    out.push(c);
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                    prev_space = false;
                } else if c == '/' && next == Some('*') {
                    state = CodeShapeState::BlockComment;
                    out.push(c);
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                    prev_space = false;
                } else if c.is_whitespace() {
                    if !prev_space {
                        out.push(' ');
                        prev_space = true;
                    }
                } else {
                    out.push(c);
                    prev_space = false;
                }
            }
        }
    }
    let mut normalized = out.trim().to_string();
    if normalized.ends_with(';') {
        normalized.pop();
        normalized = normalized.trim_end().to_string();
    }
    normalized
}

pub(super) fn normalize_type_text(raw: &str) -> String {
    compact_type_punctuation(&normalize_code_shape(raw))
}

fn compact_type_punctuation(input: &str) -> String {
    let mut out = String::new();
    let mut state = CodeShapeState::Code;
    let mut skip_spaces = false;
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match state {
            CodeShapeState::Single => {
                out.push(c);
                if c == '\\' {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else if c == '\'' {
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::Double => {
                out.push(c);
                if c == '\\' {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else if c == '"' {
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::Template => {
                out.push(c);
                if c == '\\' {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else if c == '`' {
                    state = CodeShapeState::Code;
                }
            }
            CodeShapeState::Code | CodeShapeState::LineComment | CodeShapeState::BlockComment => {
                if c == '\'' || c == '"' || c == '`' {
                    state = match c {
                        '\'' => CodeShapeState::Single,
                        '"' => CodeShapeState::Double,
                        _ => CodeShapeState::Template,
                    };
                    out.push(c);
                    skip_spaces = false;
                } else if c.is_whitespace() {
                    if !skip_spaces && !out.ends_with(' ') {
                        out.push(' ');
                    }
                } else if matches!(
                    c,
                    '<' | '>'
                        | '('
                        | ')'
                        | '['
                        | ']'
                        | '{'
                        | '}'
                        | '|'
                        | '&'
                        | ':'
                        | ','
                        | ';'
                        | '='
                        | '?'
                ) {
                    if out.ends_with(' ') {
                        out.pop();
                    }
                    out.push(c);
                    skip_spaces = true;
                } else {
                    out.push(c);
                    skip_spaces = false;
                }
            }
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_type_text;

    #[test]
    fn type_text_compacts_punctuation_without_touching_literals() {
        assert_eq!(
            normalize_type_text("Array < string | number >"),
            "Array<string|number>"
        );
        assert_eq!(
            normalize_type_text("\"a | b\" | string"),
            "\"a | b\"|string"
        );
    }
}
