use crate::protocol::Location;

pub struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        for (index, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }
        Self { line_starts }
    }

    /// Returns 1-based line numbers and 1-based UTF-8 byte columns.
    pub fn location(&self, byte_start: usize, byte_end: usize) -> Location {
        let (line, column) = self.line_column(byte_start);
        let (end_line, end_column) = self.line_column(byte_end);
        Location {
            line,
            column,
            end_line,
            end_column,
            byte_start,
            byte_end,
        }
    }

    fn line_column(&self, byte_offset: usize) -> (usize, usize) {
        let line_index = match self.line_starts.binary_search(&byte_offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        let line_start = self.line_starts.get(line_index).copied().unwrap_or(0);
        (line_index + 1, byte_offset.saturating_sub(line_start) + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::LineIndex;

    #[test]
    fn reports_one_based_byte_columns() {
        let index = LineIndex::new("a\néx\n");

        let location = index.location(4, 4);

        assert_eq!(location.line, 2);
        assert_eq!(location.column, 3);
        assert_eq!(location.end_line, 2);
        assert_eq!(location.end_column, 3);
    }

    #[test]
    fn reports_ranges_across_lines() {
        let index = LineIndex::new("abc\ndef\n");

        let location = index.location(1, 5);

        assert_eq!(location.line, 1);
        assert_eq!(location.column, 2);
        assert_eq!(location.end_line, 2);
        assert_eq!(location.end_column, 2);
    }
}
