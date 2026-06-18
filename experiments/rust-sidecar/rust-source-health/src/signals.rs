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

pub fn mute_signal(signal: &mut Signal, reason: SignalMuteReason) {
    signal.visibility = SignalVisibility::Muted;
    signal.mute_reason = Some(reason);
}

pub fn apply_signal_policy(signals: &mut [Signal], classifications: &[String]) {
    let mute_reason = if classifications.iter().any(|value| value == "generated") {
        Some(SignalMuteReason::GeneratedPath)
    } else if classifications.iter().any(|value| value == "test") {
        Some(SignalMuteReason::TestPath)
    } else {
        None
    };

    if let Some(reason) = mute_reason {
        for signal in signals {
            mute_signal(signal, reason);
        }
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
