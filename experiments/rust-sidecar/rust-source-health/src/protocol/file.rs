use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{
    AstFacts, ParseStatus, PathMeta, Signal, SignalKind, SignalMuteReason, SignalVisibility,
    SignalVisibilityState,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHealth {
    pub sha256: String,
    pub facts: Facts,
    pub ast: AstFacts,
    #[serde(skip)]
    pub signal_summary: FileSignalSummary,
    pub signals: Vec<Signal>,
    pub parse: ParseStatus,
    pub path: PathMeta,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Facts {
    pub items: usize,
    pub functions: usize,
    pub max_function_lines: usize,
    pub unsafe_blocks: usize,
    pub unsafe_functions: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSignalSummary {
    pub total: usize,
    pub review: usize,
    pub muted: usize,
    pub signals_by_kind: BTreeMap<SignalKind, usize>,
    pub signals_by_visibility: BTreeMap<SignalVisibility, usize>,
    pub review_signals_by_kind: BTreeMap<SignalKind, usize>,
    pub muted_signals_by_reason: BTreeMap<SignalMuteReason, usize>,
}

impl FileSignalSummary {
    pub fn from_signals(signals: &[Signal]) -> Self {
        let mut summary = Self {
            total: signals.len(),
            ..Self::default()
        };
        for signal in signals {
            *summary.signals_by_kind.entry(signal.kind).or_insert(0) += 1;
            *summary
                .signals_by_visibility
                .entry(signal.visibility.visibility())
                .or_insert(0) += 1;
            match signal.visibility {
                SignalVisibilityState::Review => {
                    summary.review += 1;
                    *summary
                        .review_signals_by_kind
                        .entry(signal.kind)
                        .or_insert(0) += 1;
                }
                SignalVisibilityState::Muted { mute_reason } => {
                    summary.muted += 1;
                    *summary
                        .muted_signals_by_reason
                        .entry(mute_reason)
                        .or_insert(0) += 1;
                }
            }
        }
        summary
    }
}
