pub(super) fn looks_like_regex_start(prev: Option<char>) -> bool {
    match prev {
        None => true,
        Some(ch) => "=(:,[!&|?{};".contains(ch),
    }
}

pub(super) fn regex_literal_tail(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Option<usize> {
    let mut probe = chars.clone();
    let mut skipped = 0usize;
    let mut escaped = false;
    let mut in_class = false;

    while let Some(ch) = probe.next() {
        if ch == '\n' || ch == '\r' {
            return None;
        }
        skipped += 1;
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '[' {
            in_class = true;
            continue;
        }
        if ch == ']' {
            in_class = false;
            continue;
        }
        if ch == '/' && !in_class {
            while matches!(probe.peek(), Some(flag) if flag.is_ascii_alphabetic()) {
                probe.next();
                skipped += 1;
            }
            for _ in 0..skipped {
                chars.next();
            }
            return Some(skipped);
        }
    }

    None
}

pub(super) fn regex_literal_end_chars(chars: &[char], start: usize) -> Option<usize> {
    let mut index = start + 1;
    let mut escaped = false;
    let mut in_class = false;

    while index < chars.len() {
        let ch = chars[index];
        if ch == '\n' || ch == '\r' {
            return None;
        }
        if escaped {
            escaped = false;
            index += 1;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            index += 1;
            continue;
        }
        if ch == '[' {
            in_class = true;
            index += 1;
            continue;
        }
        if ch == ']' {
            in_class = false;
            index += 1;
            continue;
        }
        if ch == '/' && !in_class {
            index += 1;
            while matches!(chars.get(index), Some(flag) if flag.is_ascii_alphabetic()) {
                index += 1;
            }
            return Some(index);
        }
        index += 1;
    }

    None
}
