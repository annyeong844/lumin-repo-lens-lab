use std::collections::{BTreeMap, BTreeSet};

use lumin_rust_source_health::protocol::{
    HealthResponse, Severity, Signal, SignalKind, SignalMuteReason, SignalVisibilityState,
};
use serde::Serialize;

use crate::policy::SIGNAL_SAMPLE_LIMIT;

use super::ProductLocation;

pub(super) fn partition_by_visibility(signals: &[Signal]) -> (Vec<&Signal>, Vec<&Signal>) {
    let mut review_signals = Vec::new();
    let mut muted_signals = Vec::new();
    for signal in signals {
        match signal.visibility {
            SignalVisibilityState::Review => review_signals.push(signal),
            SignalVisibilityState::Muted { .. } => muted_signals.push(signal),
        }
    }
    (review_signals, muted_signals)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProductSignalExample {
    kind: SignalKind,
    severity: Severity,
    #[serde(skip_serializing_if = "Option::is_none")]
    mute_reason: Option<SignalMuteReason>,
    location: ProductLocation,
}

pub(super) fn signals_for_product(signals: &[&Signal], limit: usize) -> Vec<ProductSignalExample> {
    signals
        .iter()
        .take(limit)
        .map(|signal| ProductSignalExample {
            kind: signal.kind,
            severity: signal.severity,
            mute_reason: signal.visibility.mute_reason(),
            location: ProductLocation::from(&signal.location),
        })
        .collect()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SignalSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    review: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    muted: Option<usize>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    by_kind: BTreeMap<SignalKind, usize>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    muted_by_reason: BTreeMap<SignalMuteReason, usize>,
}

impl SignalSummary {
    pub(super) fn is_empty(&self) -> bool {
        self.review.is_none()
            && self.muted.is_none()
            && self.by_kind.is_empty()
            && self.muted_by_reason.is_empty()
    }
}

pub(super) fn signal_summary(
    review_signals: &[&Signal],
    muted_signals: &[&Signal],
) -> SignalSummary {
    let mut by_kind: BTreeMap<SignalKind, usize> = BTreeMap::new();
    let mut muted_by_reason: BTreeMap<SignalMuteReason, usize> = BTreeMap::new();
    for signal in review_signals.iter().chain(muted_signals.iter()) {
        *by_kind.entry(signal.kind).or_insert(0usize) += 1;
    }
    for signal in muted_signals {
        let SignalVisibilityState::Muted { mute_reason } = signal.visibility else {
            continue;
        };
        *muted_by_reason.entry(mute_reason).or_insert(0usize) += 1;
    }

    SignalSummary {
        review: (!review_signals.is_empty()).then_some(review_signals.len()),
        muted: (!muted_signals.is_empty()).then_some(muted_signals.len()),
        by_kind,
        muted_by_reason,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyntaxReviewSignalExample<'a> {
    file: &'a str,
    kind: SignalKind,
    location: ProductLocation,
    #[serde(skip)]
    byte_start: usize,
}

pub(crate) fn syntax_review_signal_examples(
    response: &HealthResponse,
) -> Vec<SyntaxReviewSignalExample<'_>> {
    let mut examples = response
        .files
        .iter()
        .flat_map(|(path, file)| {
            file.signals
                .iter()
                .filter(|signal| signal.visibility == SignalVisibilityState::Review)
                .map(|signal| SyntaxReviewSignalExample {
                    file: path,
                    kind: signal.kind,
                    location: ProductLocation::from(&signal.location),
                    byte_start: signal.location.byte_start,
                })
        })
        .collect::<Vec<_>>();
    examples.sort_by(|left, right| {
        signal_example_priority(left.kind)
            .cmp(&signal_example_priority(right.kind))
            .then_with(|| left.file.cmp(right.file))
            .then_with(|| left.byte_start.cmp(&right.byte_start))
            .then_with(|| left.kind.cmp(&right.kind))
    });
    diversified_signal_examples(examples)
}

fn diversified_signal_examples<'a>(
    examples: Vec<SyntaxReviewSignalExample<'a>>,
) -> Vec<SyntaxReviewSignalExample<'a>> {
    let mut selected = Vec::new();
    let mut selected_kinds = BTreeSet::new();
    let mut remaining = Vec::new();
    for example in examples {
        if selected.len() < SIGNAL_SAMPLE_LIMIT && selected_kinds.insert(example.kind) {
            selected.push(example);
            continue;
        }
        remaining.push(example);
    }
    for example in remaining {
        if selected.len() >= SIGNAL_SAMPLE_LIMIT {
            break;
        }
        if selected.iter().any(|selected: &SyntaxReviewSignalExample| {
            selected.kind == example.kind
                && selected.file == example.file
                && selected.byte_start == example.byte_start
        }) {
            continue;
        }
        selected.push(example);
    }
    selected
}

fn signal_example_priority(kind: SignalKind) -> usize {
    match kind {
        SignalKind::PanicMacro | SignalKind::TodoMacro | SignalKind::UnimplementedMacro => 0,
        SignalKind::UnsafeBlock => 1,
        SignalKind::UnwrapCall | SignalKind::ExpectCall => 2,
        SignalKind::CloneCall => 3,
    }
}
