use std::collections::BTreeSet;

use crate::protocol::{
    RUST_FUNCTION_CLONE_NEAR_MIN_SIGNIFICANT_CALL_TOKEN_LEN,
    RUST_FUNCTION_CLONE_NEAR_SUPPRESSED_GENERIC_CALL_TOKENS,
};

use crate::function_clones::common::FunctionBodyFactView;

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

const DISPLAY_FORMATTER_SINK_CALL_TOKENS: &[&str] = &["write", "write_str"];

pub(super) fn significant_call_tokens(fact: &impl FunctionBodyFactView) -> Vec<&str> {
    fact.call_tokens()
        .iter()
        .filter(|token| {
            let token = token.as_ref();
            token.len() >= RUST_FUNCTION_CLONE_NEAR_MIN_SIGNIFICANT_CALL_TOKEN_LEN
                && !RUST_FUNCTION_CLONE_NEAR_SUPPRESSED_GENERIC_CALL_TOKENS.contains(&token)
        })
        .map(AsRef::as_ref)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn is_debug_formatter_boilerplate(fact: &impl FunctionBodyFactView) -> bool {
    fact.name() == "fmt"
        && trait_terminal_name(fact) == Some("Debug")
        && fact
            .call_tokens()
            .iter()
            .any(|token| DEBUG_FORMATTER_CALL_TOKENS.contains(&token.as_ref()))
}

pub(super) fn is_display_formatter(fact: &impl FunctionBodyFactView) -> bool {
    fact.name() == "fmt"
        && trait_terminal_name(fact) == Some("Display")
        && fact
            .call_tokens()
            .iter()
            .any(|token| DISPLAY_FORMATTER_SINK_CALL_TOKENS.contains(&token.as_ref()))
}

pub(super) fn shared_tokens_are_only_display_formatter_sinks(
    left: &[&str],
    right: &[&str],
) -> bool {
    let mut has_sink = false;
    let mut left_index = 0usize;
    let mut right_index = 0usize;
    while left_index < left.len() && right_index < right.len() {
        match left[left_index].cmp(right[right_index]) {
            std::cmp::Ordering::Less => left_index += 1,
            std::cmp::Ordering::Greater => right_index += 1,
            std::cmp::Ordering::Equal => {
                if !DISPLAY_FORMATTER_SINK_CALL_TOKENS.contains(&left[left_index]) {
                    return false;
                }
                has_sink = true;
                left_index += 1;
                right_index += 1;
            }
        }
    }
    has_sink
}

pub(super) fn name_tokens(name: &str) -> Vec<Box<str>> {
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
        .map(String::into_boxed_str)
        .collect()
}

fn trait_terminal_name(fact: &impl FunctionBodyFactView) -> Option<&str> {
    fact.owner()
        .and_then(|owner| owner.trait_path.as_deref())
        .and_then(path_terminal_name)
}

fn path_terminal_name(path: &str) -> Option<&str> {
    path.rsplit(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .find(|segment| !segment.is_empty())
}
