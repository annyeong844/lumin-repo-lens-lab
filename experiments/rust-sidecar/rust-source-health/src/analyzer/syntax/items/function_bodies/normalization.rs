use ra_ap_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

use super::numeric::canonical_numeric_literal;

#[derive(Clone, Copy)]
pub(super) enum LiteralPolicy {
    PreserveValues,
    AnonymizeValues,
}

pub(super) fn normalize_body(body: &SyntaxNode, literal_policy: LiteralPolicy) -> String {
    let mut tokens = Vec::new();
    for element in body.descendants_with_tokens() {
        let SyntaxElement::Token(token) = element else {
            continue;
        };
        let Some(normalized) = normalize_token(token.kind(), token.text(), literal_policy) else {
            continue;
        };
        tokens.push(normalized);
    }
    tokens.join(" ")
}

pub(super) fn compact_token_source(body: &SyntaxNode) -> String {
    body.descendants_with_tokens()
        .filter_map(|element| match element {
            SyntaxElement::Token(token) if token.kind() != SyntaxKind::WHITESPACE => {
                Some(token.text().to_string())
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_token(kind: SyntaxKind, text: &str, literal_policy: LiteralPolicy) -> Option<String> {
    match kind {
        SyntaxKind::WHITESPACE | SyntaxKind::COMMENT => None,
        SyntaxKind::IDENT | SyntaxKind::LIFETIME_IDENT => Some("<id>".to_string()),
        SyntaxKind::INT_NUMBER | SyntaxKind::FLOAT_NUMBER => Some(match literal_policy {
            LiteralPolicy::PreserveValues => canonical_numeric_literal(kind, text),
            LiteralPolicy::AnonymizeValues => "<number>".to_string(),
        }),
        SyntaxKind::STRING
        | SyntaxKind::BYTE_STRING
        | SyntaxKind::C_STRING
        | SyntaxKind::CHAR
        | SyntaxKind::BYTE => Some(match literal_policy {
            LiteralPolicy::PreserveValues => text.to_string(),
            LiteralPolicy::AnonymizeValues => "<literal>".to_string(),
        }),
        SyntaxKind::TRUE_KW | SyntaxKind::FALSE_KW => Some(match literal_policy {
            LiteralPolicy::PreserveValues => text.to_string(),
            LiteralPolicy::AnonymizeValues => "<bool>".to_string(),
        }),
        _ => Some(text.to_string()),
    }
}
