use serde::Serialize;

use crate::prewrite::intent::{NormalizedIntent, ShapeIntent};

const FIELD_ONLY_UNAVAILABLE_CITATION: &str =
    "[확인 불가, shape intent lacks exact sha256 shape hash or typeLiteral; field names alone are not structural equality evidence for P4 shape-hash lookup]";
const RUST_SHAPE_LOOKUP_UNSUPPORTED_CITATION: &str =
    "[확인 불가, Rust pre-write shape lookup is unsupported; coverage.shapes = unsupported until a Rust-owned shape-index equivalent exists]";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct ShapeLookup {
    kind: ShapeLookupKind,
    pub(in crate::prewrite) shape: ShapeIntent,
    result: ShapeLookupResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    shape_hash: Option<String>,
    citations: Vec<&'static str>,
}

impl ShapeLookup {
    pub(in crate::prewrite) fn unavailable_evidence(&self) -> UnavailableEvidence {
        UnavailableEvidence {
            evidence_lane: UnavailableEvidenceLane::ShapeHash,
            status: UnavailableEvidenceStatus::Unavailable,
            reason: "lookup-unavailable",
            artifact: "shape-index.json",
            citations: self.citations.clone(),
        }
    }

    pub(in crate::prewrite) fn is_unavailable(&self) -> bool {
        self.result == ShapeLookupResult::Unavailable
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
enum ShapeLookupKind {
    #[serde(rename = "shape")]
    Shape,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
enum ShapeLookupResult {
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct UnavailableEvidence {
    evidence_lane: UnavailableEvidenceLane,
    status: UnavailableEvidenceStatus,
    reason: &'static str,
    artifact: &'static str,
    citations: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum UnavailableEvidenceLane {
    ShapeHash,
}

#[derive(Debug, Clone, Copy, Serialize)]
enum UnavailableEvidenceStatus {
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
}

pub(in crate::prewrite) fn lookup_shapes(intent: &NormalizedIntent) -> Vec<ShapeLookup> {
    intent
        .shapes
        .iter()
        .map(|shape| {
            let citations = if shape.hash.is_none() && shape.type_literal.is_none() {
                vec![FIELD_ONLY_UNAVAILABLE_CITATION]
            } else {
                vec![RUST_SHAPE_LOOKUP_UNSUPPORTED_CITATION]
            };
            ShapeLookup {
                kind: ShapeLookupKind::Shape,
                shape: shape.clone(),
                result: ShapeLookupResult::Unavailable,
                shape_hash: shape.hash.clone(),
                citations,
            }
        })
        .collect()
}

pub(in crate::prewrite) fn unavailable_evidence_from_shape_lookups(
    lookups: &[ShapeLookup],
) -> Vec<UnavailableEvidence> {
    lookups
        .iter()
        .filter(|lookup| lookup.is_unavailable())
        .map(ShapeLookup::unavailable_evidence)
        .collect()
}
