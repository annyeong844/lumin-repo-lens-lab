use super::super::domain_cluster::DOMAIN_CLUSTER_MIN_PREFIX_LEN;

const GENERIC_DOMAIN_PREFIXES: &[&str] = &[
    "index", "main", "test", "tests", "spec", "helper", "helpers", "utils", "util", "types", "type",
];

pub(super) fn is_usable_domain_key(key: &str) -> bool {
    key.len() >= DOMAIN_CLUSTER_MIN_PREFIX_LEN && !GENERIC_DOMAIN_PREFIXES.contains(&key)
}

pub(super) fn split_name_tokens(base_name: &str) -> Vec<String> {
    let chars = base_name.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut current = String::new();

    for (index, ch) in chars.iter().copied().enumerate() {
        if matches!(ch, '-' | '_' | '.' | ' ' | '\t' | '\n' | '\r') {
            push_token(&mut tokens, &mut current);
            continue;
        }

        if let Some(previous) = current.chars().last() {
            let next = chars.get(index + 1).copied();
            let lower_to_upper = (previous.is_ascii_lowercase() || previous.is_ascii_digit())
                && ch.is_ascii_uppercase();
            let acronym_boundary = previous.is_ascii_uppercase()
                && ch.is_ascii_uppercase()
                && next.is_some_and(|next| next.is_ascii_lowercase());
            if lower_to_upper || acronym_boundary {
                push_token(&mut tokens, &mut current);
            }
        }
        current.push(ch);
    }
    push_token(&mut tokens, &mut current);

    tokens
}

pub(super) fn normalize_domain_key(value: &str) -> String {
    normalize_domain_token(value)
}

pub(super) fn normalize_domain_token(value: &str) -> String {
    let raw = value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect::<String>();
    if raw.len() > 4 && raw.ends_with("ies") {
        return format!("{}y", &raw[..raw.len() - 3]);
    }
    if raw.len() > 4 && raw.ends_with('s') {
        return raw[..raw.len() - 1].to_string();
    }
    raw
}

pub(super) fn display_prefix_from_tokens(tokens: &[String]) -> String {
    let Some((first, rest)) = tokens.split_first() else {
        return String::new();
    };
    let mut display = first.clone();
    for token in rest {
        display.push_str(&capitalize_first(token));
    }
    display
}

fn push_token(tokens: &mut Vec<String>, current: &mut String) {
    if current.is_empty() {
        return;
    }
    tokens.push(std::mem::take(current));
}

fn capitalize_first(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut capitalized = first.to_uppercase().collect::<String>();
    capitalized.push_str(chars.as_str());
    capitalized
}
