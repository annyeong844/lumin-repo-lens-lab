pub(super) const TOKENIZER_VERSION: &str = "camel-snake-kebab-digit-v1";
pub(super) const TOKEN_POLICY_VERSION: &str = "prewrite-token-policy-v1";
pub(super) const WEAK_COMMON_TOKENS: [&str; 15] = [
    "add", "build", "check", "create", "delete", "get", "load", "make", "parse", "read", "return",
    "save", "set", "update", "write",
];

pub(super) fn unique_tokens(parts: &[&str]) -> Vec<String> {
    unique_prewrite_tokens(parts)
        .into_iter()
        .filter(|token| token.len() >= 2 && !is_semantic_stop_token(token))
        .collect()
}

pub(super) fn unique_prewrite_tokens(parts: &[&str]) -> Vec<String> {
    let mut tokens = Vec::new();
    for part in parts {
        for token in tokenize(part) {
            if !tokens.contains(&token) {
                tokens.push(token);
            }
        }
    }
    tokens
}

pub(super) fn common_tokens(left: &str, right: &str) -> Vec<String> {
    let left = unique_tokens(&[left]);
    unique_tokens(&[right])
        .into_iter()
        .filter(|token| left.contains(token))
        .collect()
}

pub(super) fn has_only_weak_common_tokens(left: &str, right: &str) -> bool {
    let common = common_tokens(left, right);
    !common.is_empty() && common.iter().all(|token| is_weak_common_token(token))
}

pub(super) fn is_weak_common_token(token: &str) -> bool {
    WEAK_COMMON_TOKENS.contains(&token.to_ascii_lowercase().as_str())
}

fn tokenize(value: &str) -> Vec<String> {
    let chars = value.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut current = String::new();
    for (index, current_char) in chars.iter().copied().enumerate() {
        if !current_char.is_ascii_alphanumeric() {
            push_normalized(&mut tokens, &mut current);
            continue;
        }

        let previous = index
            .checked_sub(1)
            .and_then(|index| chars.get(index))
            .copied();
        let next = chars.get(index + 1).copied();
        if !current.is_empty() && should_split(previous, current_char, next) {
            push_normalized(&mut tokens, &mut current);
        }
        current.push(current_char);
    }
    push_normalized(&mut tokens, &mut current);
    tokens
}

fn should_split(previous: Option<char>, current: char, next: Option<char>) -> bool {
    let Some(previous) = previous.filter(char::is_ascii_alphanumeric) else {
        return false;
    };
    (previous.is_ascii_uppercase()
        && current.is_ascii_uppercase()
        && next.is_some_and(|character| character.is_ascii_lowercase()))
        || ((previous.is_ascii_lowercase() || previous.is_ascii_digit())
            && current.is_ascii_uppercase())
        || (previous.is_ascii_alphabetic() && current.is_ascii_digit())
        || (previous.is_ascii_digit() && current.is_ascii_alphabetic())
}

fn push_normalized(tokens: &mut Vec<String>, current: &mut String) {
    if current.is_empty() {
        return;
    }
    tokens.push(normalize_token(current));
    current.clear();
}

fn normalize_token(token: &str) -> String {
    let token = token.to_ascii_lowercase();
    match token.as_str() {
        "artifacts" => "artifact".to_string(),
        "rel" => "relative".to_string(),
        "ctx" => "context".to_string(),
        "cfg" => "config".to_string(),
        "config" => "configuration".to_string(),
        "exists" | "existing" | "existence" => "exist".to_string(),
        "series" | "species" => token,
        value if value.len() > 4 && value.ends_with("ies") => {
            format!("{}y", &value[..value.len() - 3])
        }
        _ => token,
    }
}

fn is_semantic_stop_token(token: &str) -> bool {
    matches!(
        token,
        "a" | "an"
            | "and"
            | "as"
            | "at"
            | "by"
            | "for"
            | "from"
            | "in"
            | "into"
            | "of"
            | "on"
            | "or"
            | "the"
            | "this"
            | "that"
            | "to"
            | "with"
            | "add"
            | "new"
            | "helper"
            | "function"
            | "type"
            | "file"
            | "module"
            | "service"
            | "manager"
            | "index"
            | "main"
            | "src"
            | "lib"
            | "utils"
            | "util"
            | "ts"
            | "js"
            | "mjs"
            | "cjs"
            | "tsx"
            | "jsx"
    )
}
