use std::collections::BTreeSet;

use crate::protocol::{
    AstFunctionBodyFingerprint, RUST_FUNCTION_CLONE_NEAR_MIN_SIGNIFICANT_CALL_TOKEN_LEN,
    RUST_FUNCTION_CLONE_NEAR_SUPPRESSED_GENERIC_CALL_TOKENS,
};

const DEBUG_FORMATTER_CALL_TOKENS: &[&str] = &[
    "debug_struct",
    "debug_tuple",
    "debug_list",
    "debug_set",
    "debug_map",
    "field",
    "entry",
    "finish",
    "finish_non_exhaustive",
];

pub(super) fn significant_call_tokens(fact: &AstFunctionBodyFingerprint) -> Vec<String> {
    fact.call_tokens
        .iter()
        .filter(|token| {
            token.len() >= RUST_FUNCTION_CLONE_NEAR_MIN_SIGNIFICANT_CALL_TOKEN_LEN
                && !RUST_FUNCTION_CLONE_NEAR_SUPPRESSED_GENERIC_CALL_TOKENS
                    .contains(&token.as_str())
        })
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn is_debug_formatter_boilerplate(fact: &AstFunctionBodyFingerprint) -> bool {
    fact.name == "fmt"
        && fact
            .owner
            .as_ref()
            .and_then(|owner| owner.trait_path.as_deref())
            .and_then(path_terminal_name)
            == Some("Debug")
        && fact
            .call_tokens
            .iter()
            .any(|token| DEBUG_FORMATTER_CALL_TOKENS.contains(&token.as_str()))
}

pub(super) fn name_tokens(name: &str) -> Vec<String> {
    let mut expanded = String::new();
    let mut previous_lower_or_digit = false;
    for ch in name.chars() {
        if ch.is_ascii_uppercase() && previous_lower_or_digit {
            expanded.push(' ');
        }
        expanded.push(ch);
        previous_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
    }
    expanded
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(str::trim)
        .filter(|token| token.len() >= 2)
        .map(str::to_ascii_lowercase)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn path_terminal_name(path: &str) -> Option<&str> {
    path.rsplit(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .find(|segment| !segment.is_empty())
}
