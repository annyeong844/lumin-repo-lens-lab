use super::blocks::{is_relative_spec, line_of, SfcLanguage, StyleBlock};
use super::protocol::SfcStyleAssetReference;

pub(super) fn extract_style_asset_references(
    source: &str,
    file_path: &str,
    language: SfcLanguage,
    blocks: &[StyleBlock<'_>],
) -> Vec<SfcStyleAssetReference> {
    let mut out = Vec::new();
    for block in blocks {
        parse_style_block(source, file_path, language, block, &mut out);
    }
    out
}

fn parse_style_block(
    file_source: &str,
    file_path: &str,
    language: SfcLanguage,
    block: &StyleBlock<'_>,
    out: &mut Vec<SfcStyleAssetReference>,
) {
    let style = block.content;
    let mut index = 0;
    while index < style.len() {
        if style[index..].starts_with("/*") {
            index = style[index + 2..]
                .find("*/")
                .map_or(style.len(), |offset| index + offset + 4);
            continue;
        }
        let Some(character) = style[index..].chars().next() else {
            break;
        };
        if character == '\'' || character == '"' {
            index = skip_css_string(style, index);
            continue;
        }
        if character == '@' && starts_with_ascii_case_insensitive(&style[index..], "@import") {
            if let Some(parsed) = parse_css_import(style, index) {
                if is_relative_spec(&parsed.value) {
                    out.push(SfcStyleAssetReference {
                        consumer_file: file_path.to_string(),
                        from_spec: parsed.value,
                        kind: "sfc-style-import",
                        source: "sfc-style-import",
                        style_kind: "import",
                        import_syntax: Some(parsed.syntax),
                        confidence: "grounded-asset-reference",
                        line: line_of(file_source, block.start_offset + index),
                        sfc_block_kind: block.kind.clone(),
                        sfc_language: language.as_str(),
                    });
                }
                index = parsed.end;
                continue;
            }
        }
        if starts_with_ascii_case_insensitive(&style[index..], "url") {
            if let Some(parsed) = parse_css_url(style, index) {
                if is_relative_spec(&parsed.value) {
                    out.push(SfcStyleAssetReference {
                        consumer_file: file_path.to_string(),
                        from_spec: parsed.value,
                        kind: "sfc-style-url",
                        source: "sfc-style-url",
                        style_kind: "url",
                        import_syntax: None,
                        confidence: "grounded-asset-reference",
                        line: line_of(file_source, block.start_offset + index),
                        sfc_block_kind: block.kind.clone(),
                        sfc_language: language.as_str(),
                    });
                }
                index = parsed.end;
                continue;
            }
        }
        index += character.len_utf8();
    }
}

struct ParsedValue {
    value: String,
    end: usize,
    syntax: &'static str,
}

fn parse_css_import(source: &str, index: usize) -> Option<ParsedValue> {
    let mut cursor = index + "@import".len();
    if source[cursor..]
        .chars()
        .next()
        .is_some_and(is_css_ident_char)
    {
        return None;
    }
    cursor = skip_css_whitespace(source, cursor);
    if starts_with_ascii_case_insensitive(&source[cursor..], "url") {
        let parsed = parse_css_url(source, cursor)?;
        return Some(ParsedValue {
            value: parsed.value,
            end: parsed.end,
            syntax: "url",
        });
    }
    let quote = source[cursor..].chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let (value, end) = parse_css_quoted(source, cursor)?;
    Some(ParsedValue {
        value: value.trim().to_string(),
        end,
        syntax: "string",
    })
}

fn parse_css_url(source: &str, index: usize) -> Option<ParsedValue> {
    let previous = source[..index].chars().next_back();
    let mut cursor = index + 3;
    if previous.is_some_and(is_css_ident_char)
        || source[cursor..]
            .chars()
            .next()
            .is_some_and(is_css_ident_char)
    {
        return None;
    }
    cursor = skip_css_whitespace(source, cursor);
    if !source[cursor..].starts_with('(') {
        return None;
    }
    cursor = skip_css_whitespace(source, cursor + 1);
    let next = source[cursor..].chars().next()?;
    if next == '\'' || next == '"' {
        let (value, end) = parse_css_quoted(source, cursor)?;
        cursor = skip_css_whitespace(source, end);
        if !source[cursor..].starts_with(')') {
            return None;
        }
        return Some(ParsedValue {
            value: value.trim().to_string(),
            end: cursor + 1,
            syntax: "url",
        });
    }

    let mut value = String::new();
    while cursor < source.len() {
        let character = source[cursor..].chars().next()?;
        if character == ')' {
            return Some(ParsedValue {
                value: value.trim().to_string(),
                end: cursor + 1,
                syntax: "url",
            });
        }
        if character == '\\' {
            let (escaped, end) = parse_css_escape(source, cursor);
            value.push_str(&escaped);
            cursor = end;
            continue;
        }
        value.push(character);
        cursor += character.len_utf8();
    }
    None
}

fn parse_css_quoted(source: &str, index: usize) -> Option<(String, usize)> {
    let quote = source[index..].chars().next()?;
    let mut cursor = index + quote.len_utf8();
    let mut value = String::new();
    while cursor < source.len() {
        let character = source[cursor..].chars().next()?;
        if character == quote {
            return Some((value, cursor + character.len_utf8()));
        }
        if character == '\\' {
            let (escaped, end) = parse_css_escape(source, cursor);
            value.push_str(&escaped);
            cursor = end;
            continue;
        }
        value.push(character);
        cursor += character.len_utf8();
    }
    None
}

fn parse_css_escape(source: &str, index: usize) -> (String, usize) {
    let mut cursor = index + 1;
    if cursor >= source.len() {
        return (String::new(), cursor);
    }
    if source[cursor..].starts_with("\r\n") {
        return (String::new(), cursor + 2);
    }
    let Some(first) = source[cursor..].chars().next() else {
        return (String::new(), cursor);
    };
    if matches!(first, '\n' | '\r' | '\u{000c}') {
        return (String::new(), cursor + first.len_utf8());
    }
    if first.is_ascii_hexdigit() {
        let mut hex = String::new();
        while cursor < source.len() && hex.len() < 6 {
            let Some(character) = source[cursor..].chars().next() else {
                break;
            };
            if !character.is_ascii_hexdigit() {
                break;
            }
            hex.push(character);
            cursor += character.len_utf8();
        }
        if source[cursor..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            cursor += source[cursor..].chars().next().map_or(0, char::len_utf8);
        }
        let value = u32::from_str_radix(&hex, 16)
            .ok()
            .and_then(char::from_u32)
            .filter(|character| *character != '\0')
            .unwrap_or('\u{fffd}');
        return (value.to_string(), cursor);
    }
    (first.to_string(), cursor + first.len_utf8())
}

fn skip_css_string(source: &str, index: usize) -> usize {
    let Some(quote) = source[index..].chars().next() else {
        return source.len();
    };
    let mut cursor = index + quote.len_utf8();
    while cursor < source.len() {
        let Some(character) = source[cursor..].chars().next() else {
            break;
        };
        if character == '\\' {
            cursor += character.len_utf8();
            if let Some(escaped) = source[cursor..].chars().next() {
                cursor += escaped.len_utf8();
            }
            continue;
        }
        cursor += character.len_utf8();
        if character == quote {
            return cursor;
        }
    }
    source.len()
}

fn skip_css_whitespace(source: &str, mut index: usize) -> usize {
    while let Some(character) = source[index..].chars().next() {
        if !character.is_whitespace() {
            break;
        }
        index += character.len_utf8();
    }
    index
}

fn is_css_ident_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
}

fn starts_with_ascii_case_insensitive(source: &str, prefix: &str) -> bool {
    source
        .get(..prefix.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
}
