pub(super) fn is_ident_char(ch: Option<char>) -> bool {
    matches!(ch, Some(c) if c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

pub(super) fn trim_leading_block_comments(mut value: &str) -> &str {
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
