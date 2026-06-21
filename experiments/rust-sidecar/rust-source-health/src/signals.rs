use crate::locations::LineIndex;
use crate::protocol::{
    Claim, ParseError, PathClassification, Severity, Signal, SignalKind, SignalMuteReason,
    SignalVisibilityState,
};
use ra_ap_syntax::TextRange;

pub(crate) fn review_signal(kind: SignalKind, line_index: &LineIndex, range: TextRange) -> Signal {
    Signal {
        kind,
        severity: Severity::Review,
        claim: Claim::SyntaxOnly,
        visibility: SignalVisibilityState::Review,
        location: location_for_range(line_index, range),
    }
}

pub(crate) fn mute_signal(signal: &mut Signal, reason: SignalMuteReason) {
    signal.visibility = SignalVisibilityState::Muted {
        mute_reason: reason,
    };
}

pub(crate) fn apply_signal_policy(signals: &mut [Signal], classifications: &[PathClassification]) {
    let mute_reason = if classifications.contains(&PathClassification::Generated) {
        Some(SignalMuteReason::GeneratedPath)
    } else if classifications.contains(&PathClassification::Test) {
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

pub(crate) fn syntax_parse_error(
    message: String,
    line_index: &LineIndex,
    range: TextRange,
) -> ParseError {
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

pub(crate) fn text_size_to_usize(value: ra_ap_syntax::TextSize) -> usize {
    u32::from(value) as usize
}
