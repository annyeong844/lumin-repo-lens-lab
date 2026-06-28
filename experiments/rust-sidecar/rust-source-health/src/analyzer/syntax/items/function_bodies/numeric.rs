use ra_ap_syntax::SyntaxKind;

pub(super) fn canonical_numeric_literal(kind: SyntaxKind, text: &str) -> String {
    match kind {
        SyntaxKind::INT_NUMBER => canonical_int_literal(text),
        SyntaxKind::FLOAT_NUMBER => canonical_float_literal(text),
        _ => text.to_string(),
    }
}

fn canonical_int_literal(text: &str) -> String {
    let Some(without_suffix) = strip_numeric_suffix(text, INT_SUFFIXES) else {
        return text.to_string();
    };
    let compact = without_suffix.replace('_', "");
    let (digits, radix) = if let Some(rest) = compact
        .strip_prefix("0x")
        .or_else(|| compact.strip_prefix("0X"))
    {
        (rest, 16)
    } else if let Some(rest) = compact
        .strip_prefix("0o")
        .or_else(|| compact.strip_prefix("0O"))
    {
        (rest, 8)
    } else if let Some(rest) = compact
        .strip_prefix("0b")
        .or_else(|| compact.strip_prefix("0B"))
    {
        (rest, 2)
    } else {
        (compact.as_str(), 10)
    };

    u128::from_str_radix(digits, radix)
        .map(|value| format!("int:{value}"))
        .unwrap_or_else(|_| text.to_string())
}

fn canonical_float_literal(text: &str) -> String {
    let Some(without_suffix) = strip_numeric_suffix(text, FLOAT_SUFFIXES) else {
        return text.to_string();
    };
    let compact = without_suffix.replace('_', "");
    compact
        .parse::<f64>()
        .map(|value| format!("float:{:016x}", value.to_bits()))
        .unwrap_or_else(|_| text.to_string())
}

fn strip_numeric_suffix<'a>(text: &'a str, suffixes: &[&str]) -> Option<&'a str> {
    let lower = text.to_ascii_lowercase();
    let without_suffix = suffixes
        .iter()
        .find_map(|suffix| {
            lower
                .ends_with(suffix)
                .then(|| &text[..text.len() - suffix.len()])
        })
        .unwrap_or(text);
    (!without_suffix.is_empty()).then_some(without_suffix)
}

const INT_SUFFIXES: &[&str] = &[
    "usize", "isize", "u128", "i128", "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8",
];

const FLOAT_SUFFIXES: &[&str] = &["f64", "f32"];
