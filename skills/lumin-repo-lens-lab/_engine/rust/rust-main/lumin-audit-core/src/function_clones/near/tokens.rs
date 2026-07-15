use std::collections::BTreeSet;

use super::super::facts::FunctionFact;

const GENERIC_CALL_TOKENS: &[&str] = &[
    "apply", "bind", "call", "catch", "filter", "find", "forEach", "format", "includes", "join",
    "map", "push", "reduce", "slice", "split", "then", "toString", "trim",
];

pub(super) fn significant_call_tokens(fact: &FunctionFact) -> Vec<&str> {
    fact.call_tokens
        .iter()
        .map(String::as_str)
        .filter(|token| token.len() >= 4 && !GENERIC_CALL_TOKENS.contains(token))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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
