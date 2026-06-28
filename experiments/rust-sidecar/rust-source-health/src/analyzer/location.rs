use crate::locations::LineIndex;
use crate::protocol::Location;
use crate::signals::text_size_to_usize;

use ra_ap_syntax::TextRange;

pub(super) fn ast_location(line_index: &LineIndex, range: TextRange) -> Location {
    line_index.location(
        text_size_to_usize(range.start()),
        text_size_to_usize(range.end()),
    )
}

pub(super) fn line_span(line_index: &LineIndex, range: TextRange) -> usize {
    let byte_start = text_size_to_usize(range.start());
    let byte_end = text_size_to_usize(range.end());
    let end_point = if byte_end > byte_start {
        byte_end - 1
    } else {
        byte_end
    };
    let start = line_index.location(byte_start, byte_start);
    let end = line_index.location(end_point, end_point);
    end.line.saturating_sub(start.line) + 1
}
