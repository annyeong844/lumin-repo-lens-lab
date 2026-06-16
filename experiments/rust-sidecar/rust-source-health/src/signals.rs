use crate::locations::LineIndex;
use crate::protocol::{ParseError, Signal};
use ra_ap_syntax::TextRange;

pub fn review_signal(kind: &str, line_index: &LineIndex, range: TextRange) -> Signal {
    Signal {
        kind: kind.to_string(),
        severity: "review".to_string(),
        claim: "syntax-only".to_string(),
        location: location_for_range(line_index, range),
    }
}

pub fn syntax_parse_error(message: String, line_index: &LineIndex, range: TextRange) -> ParseError {
    ParseError {
        message,
        claim: "syntax-only".to_string(),
        location: location_for_range(line_index, range),
    }
}

fn location_for_range(line_index: &LineIndex, range: TextRange) -> crate::protocol::Location {
    let byte_start = text_size_to_usize(range.start());
    let byte_end = text_size_to_usize(range.end());
    line_index.location(byte_start, byte_end)
}

pub fn text_size_to_usize(value: ra_ap_syntax::TextSize) -> usize {
    u32::from(value) as usize
}
