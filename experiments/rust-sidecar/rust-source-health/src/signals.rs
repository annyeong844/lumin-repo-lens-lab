use crate::locations::LineIndex;
use crate::protocol::{
    Claim, ParseError, Severity, Signal, SignalKind, SignalMuteReason, SignalVisibility,
};
use ra_ap_syntax::TextRange;

pub fn review_signal(kind: SignalKind, line_index: &LineIndex, range: TextRange) -> Signal {
    Signal {
        kind,
        severity: Severity::Review,
        claim: Claim::SyntaxOnly,
        visibility: SignalVisibility::Review,
        mute_reason: None,
        location: location_for_range(line_index, range),
    }
}

pub fn apply_signal_policy(signals: &mut [Signal], classifications: &[String]) {
    let mute_reason = if classifications.iter().any(|value| value == "generated") {
        Some(SignalMuteReason::GeneratedPath)
    } else if classifications.iter().any(|value| value == "test") {
        Some(SignalMuteReason::TestPath)
    } else {
        None
    };

    for signal in signals {
        signal.visibility = if mute_reason.is_some() {
            SignalVisibility::Muted
        } else {
            SignalVisibility::Review
        };
        signal.mute_reason = mute_reason;
    }
}

pub fn syntax_parse_error(message: String, line_index: &LineIndex, range: TextRange) -> ParseError {
    ParseError {
        message,
        claim: Claim::SyntaxOnly,
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
