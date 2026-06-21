use std::collections::BTreeMap;

use serde::Serialize;

use crate::policy::{semantic_examples, DegradedReason, ACTION_SAMPLE_LIMIT};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SemanticExamples<T: Serialize> {
    pub(super) findings: usize,
    pub(super) sample_limit: usize,
    pub(super) examples: Vec<T>,
}

impl<T: Serialize> SemanticExamples<T> {
    pub(super) fn new(findings: usize, examples: Vec<T>) -> Self {
        Self {
            findings,
            sample_limit: ACTION_SAMPLE_LIMIT,
            examples,
        }
    }

    pub(super) fn findings(&self) -> usize {
        self.findings
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SemanticReasonExamples<K: Ord + Serialize, T: Serialize> {
    pub(super) findings: usize,
    pub(super) sample_limit: usize,
    pub(super) by_reason: BTreeMap<K, usize>,
    pub(super) examples: Vec<T>,
}

impl<K: Ord + Serialize, T: Serialize> SemanticReasonExamples<K, T> {
    pub(super) fn new(findings: usize, by_reason: BTreeMap<K, usize>, examples: Vec<T>) -> Self {
        Self {
            findings,
            sample_limit: ACTION_SAMPLE_LIMIT,
            by_reason,
            examples,
        }
    }

    pub(super) fn findings(&self) -> usize {
        self.findings
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SemanticDegradedExamples<'a> {
    pub(super) findings: usize,
    pub(super) coverage_entries: usize,
    pub(super) sample_limit: usize,
    pub(super) by_reason: BTreeMap<DegradedReason, usize>,
    pub(super) examples: Vec<semantic_examples::DegradedExample<'a>>,
}

impl<'a> SemanticDegradedExamples<'a> {
    pub(super) fn new(
        findings: usize,
        coverage_entries: usize,
        by_reason: BTreeMap<DegradedReason, usize>,
        examples: Vec<semantic_examples::DegradedExample<'a>>,
    ) -> Self {
        Self {
            findings,
            coverage_entries,
            sample_limit: ACTION_SAMPLE_LIMIT,
            by_reason,
            examples,
        }
    }

    pub(super) fn findings(&self) -> usize {
        self.findings
    }

    pub(super) fn coverage_entries(&self) -> usize {
        self.coverage_entries
    }
}
