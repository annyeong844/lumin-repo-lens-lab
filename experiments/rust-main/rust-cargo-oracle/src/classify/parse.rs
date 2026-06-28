use crate::cargo_json::CargoJsonStream;
use crate::protocol::StreamParseStatus;

use super::model::ParsedJsonl;

pub(crate) fn parse_cargo_jsonl(stdout: &str) -> ParsedJsonl {
    let mut messages = CargoJsonStream::empty();
    let mut invalid_json_line_count = 0;
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        match messages.push_json_line(line) {
            Ok(()) => {}
            Err(_) => invalid_json_line_count += 1,
        }
    }

    let stream_parse_status = if messages.is_empty() && invalid_json_line_count == 0 {
        StreamParseStatus::NoJsonEvents
    } else if invalid_json_line_count == 0 {
        StreamParseStatus::Complete
    } else {
        StreamParseStatus::InvalidJson
    };

    ParsedJsonl::new(messages, invalid_json_line_count, stream_parse_status)
}

pub(crate) fn skipped_cargo_jsonl() -> ParsedJsonl {
    ParsedJsonl::skipped()
}

#[cfg(test)]
mod tests {
    use crate::cargo_json::CargoJsonReason;

    use super::StreamParseStatus;
    use super::{parse_cargo_jsonl, skipped_cargo_jsonl};

    #[test]
    fn empty_stream_is_no_json_events() {
        let parsed = parse_cargo_jsonl("");

        assert_eq!(
            parsed.stream_parse_status(),
            StreamParseStatus::NoJsonEvents
        );
        assert_eq!(parsed.invalid_json_line_count(), 0);
        assert!(parsed.has_no_messages());
    }

    #[test]
    fn invalid_json_lines_make_stream_incomplete_without_dropping_valid_events() {
        let parsed =
            parse_cargo_jsonl("not-json\n{\"reason\":\"build-finished\",\"success\":true}\n");

        assert_eq!(parsed.stream_parse_status(), StreamParseStatus::InvalidJson);
        assert_eq!(parsed.invalid_json_line_count(), 1);
        assert_eq!(parsed.message_count(), 1);
        assert!(parsed.contains_reason(CargoJsonReason::BuildFinished));
    }

    #[test]
    fn skipped_stream_is_explicitly_not_run() {
        let parsed = skipped_cargo_jsonl();

        assert_eq!(parsed.stream_parse_status(), StreamParseStatus::NotRun);
        assert_eq!(parsed.invalid_json_line_count(), 0);
        assert!(parsed.has_no_messages());
    }
}
