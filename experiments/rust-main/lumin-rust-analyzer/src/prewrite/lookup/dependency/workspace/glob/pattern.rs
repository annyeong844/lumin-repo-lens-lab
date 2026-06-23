pub(super) fn path_components_start_with(path: &[String], prefix: &[String]) -> bool {
    path.len() >= prefix.len() && path[..prefix.len()] == *prefix
}

pub(super) fn member_components(pattern: &str) -> Vec<String> {
    pattern
        .replace('\\', "/")
        .split('/')
        .filter(|component| !component.is_empty())
        .map(str::to_string)
        .collect()
}

pub(super) fn member_contains_glob(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?') || pattern.contains('[')
}

pub(super) fn component_contains_glob(component: &str) -> bool {
    component.contains('*') || component.contains('?') || component.contains('[')
}

pub(super) fn glob_component_matches(pattern: &str, value: &str) -> bool {
    glob_chars_match(
        &pattern.chars().collect::<Vec<_>>(),
        &value.chars().collect::<Vec<_>>(),
    )
}

fn glob_chars_match(pattern: &[char], value: &[char]) -> bool {
    if pattern.is_empty() {
        return value.is_empty();
    }
    match pattern[0] {
        '*' => {
            glob_chars_match(&pattern[1..], value)
                || (!value.is_empty() && glob_chars_match(pattern, &value[1..]))
        }
        '?' => !value.is_empty() && glob_chars_match(&pattern[1..], &value[1..]),
        '[' => {
            if let Some((matched, consumed)) = match_char_class(pattern, value.first().copied()) {
                matched && glob_chars_match(&pattern[consumed..], &value[1..])
            } else {
                value.first() == Some(&'[') && glob_chars_match(&pattern[1..], &value[1..])
            }
        }
        ch => value.first() == Some(&ch) && glob_chars_match(&pattern[1..], &value[1..]),
    }
}

fn match_char_class(pattern: &[char], value: Option<char>) -> Option<(bool, usize)> {
    let value = value?;
    let negated = matches!(pattern.get(1), Some('!' | '^'));
    let mut index = if negated { 2 } else { 1 };
    let mut matched = false;
    let mut has_member = false;
    while index < pattern.len() {
        if pattern[index] == ']' && has_member {
            return Some((if negated { !matched } else { matched }, index + 1));
        }
        if index + 2 < pattern.len() && pattern[index + 1] == '-' && pattern[index + 2] != ']' {
            let start = pattern[index];
            let end = pattern[index + 2];
            matched |= start <= value && value <= end;
            index += 3;
        } else {
            matched |= pattern[index] == value;
            index += 1;
        }
        has_member = true;
    }
    None
}
