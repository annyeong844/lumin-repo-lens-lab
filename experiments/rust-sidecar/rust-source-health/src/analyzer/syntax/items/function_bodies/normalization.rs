use ra_ap_syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

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
        let Some(normalized) = normalize_token(&token, literal_policy) else {
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

fn normalize_token(token: &SyntaxToken, literal_policy: LiteralPolicy) -> Option<String> {
    let kind = token.kind();
    let text = token.text();
    match kind {
        SyntaxKind::WHITESPACE | SyntaxKind::COMMENT => None,
        SyntaxKind::IDENT | SyntaxKind::LIFETIME_IDENT => Some(
            if should_preserve_identifier(token) {
                text
            } else {
                "<id>"
            }
            .to_string(),
        ),
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

fn should_preserve_identifier(token: &SyntaxToken) -> bool {
    previous_non_trivia_token(token).is_some_and(|previous| matches!(previous.text(), "." | "::"))
        || is_record_field_key(token)
}

fn is_record_field_key(token: &SyntaxToken) -> bool {
    let Some(name_node) = token.parent() else {
        return false;
    };
    if !matches!(name_node.kind(), SyntaxKind::NAME | SyntaxKind::NAME_REF) {
        return false;
    }
    name_node.parent().is_some_and(|parent| {
        matches!(
            parent.kind(),
            SyntaxKind::RECORD_EXPR_FIELD | SyntaxKind::RECORD_PAT_FIELD | SyntaxKind::RECORD_FIELD
        )
    })
}

fn previous_non_trivia_token(token: &SyntaxToken) -> Option<SyntaxToken> {
    let mut previous = token.prev_token();
    while let Some(candidate) = previous {
        if !is_trivia(candidate.kind()) {
            return Some(candidate);
        }
        previous = candidate.prev_token();
    }
    None
}

fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::WHITESPACE | SyntaxKind::COMMENT)
}
