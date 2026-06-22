use serde::Serialize;

use crate::prewrite::intent::{NormalizedIntent, ShapeIntent};
use lumin_rust_source_health::protocol::{
    AstShapeConfidence, AstShapeField, AstShapeKind, HealthResponse,
};

const FIELD_ONLY_UNAVAILABLE_CITATION: &str =
    "[확인 불가, shape intent lacks exact sha256 shape hash or typeLiteral; field names alone are not structural equality evidence for P4 shape-hash lookup]";
const TYPE_LITERAL_UNAVAILABLE_CITATION: &str =
    "[확인 불가, Rust pre-write shape lookup does not normalize TS/JS shape.typeLiteral; provide an exact Rust source-health shape hash]";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct ShapeLookup {
    kind: ShapeLookupKind,
    pub(in crate::prewrite) shape: ShapeIntent,
    result: ShapeLookupResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    shape_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shape_hash_source: Option<ShapeHashSource>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    matches: Vec<ShapeMatch>,
    citations: Vec<String>,
}

impl ShapeLookup {
    pub(in crate::prewrite) fn unavailable_evidence(&self) -> UnavailableEvidence {
        UnavailableEvidence {
            evidence_lane: UnavailableEvidenceLane::ShapeHash,
            status: UnavailableEvidenceStatus::Unavailable,
            reason: "lookup-unavailable",
            artifact: "rust-source-health",
            citations: self.citations.clone(),
        }
    }

    pub(in crate::prewrite) fn is_unavailable(&self) -> bool {
        self.result == ShapeLookupResult::Unavailable
    }

    pub(in crate::prewrite) fn is_shape_match(&self) -> bool {
        self.result == ShapeLookupResult::ShapeMatch && !self.matches.is_empty()
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
enum ShapeLookupKind {
    #[serde(rename = "shape")]
    Shape,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
enum ShapeLookupResult {
    #[serde(rename = "SHAPE_MATCH")]
    ShapeMatch,
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum ShapeHashSource {
    Hash,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShapeMatch {
    identity: String,
    owner_file: String,
    name: String,
    hash: String,
    shape_kind: AstShapeKind,
    fields: Vec<AstShapeField>,
    visibility: lumin_rust_source_health::protocol::AstVisibility,
    confidence: AstShapeConfidence,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct UnavailableEvidence {
    evidence_lane: UnavailableEvidenceLane,
    status: UnavailableEvidenceStatus,
    reason: &'static str,
    artifact: &'static str,
    citations: Vec<String>,
}

impl UnavailableEvidence {
    pub(in crate::prewrite) fn inline_extraction(
        reason: &'static str,
        artifact: &'static str,
        citations: Vec<&'static str>,
    ) -> Self {
        Self {
            evidence_lane: UnavailableEvidenceLane::InlineExtraction,
            status: UnavailableEvidenceStatus::Unavailable,
            reason,
            artifact,
            citations: citations.into_iter().map(str::to_string).collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum UnavailableEvidenceLane {
    ShapeHash,
    InlineExtraction,
}

#[derive(Debug, Clone, Copy, Serialize)]
enum UnavailableEvidenceStatus {
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
}

pub(in crate::prewrite) fn lookup_shapes(
    intent: &NormalizedIntent,
    syntax: &HealthResponse,
) -> Vec<ShapeLookup> {
    intent
        .shapes
        .iter()
        .map(|shape| lookup_shape(shape, syntax))
        .collect()
}

fn lookup_shape(shape: &ShapeIntent, syntax: &HealthResponse) -> ShapeLookup {
    if let Some(hash) = &shape.hash {
        let matches = shape_hash_matches(hash, syntax);
        if matches.is_empty() {
            return unavailable(
                shape,
                Some(hash.clone()),
                Some(ShapeHashSource::Hash),
                vec![format!(
                    "[확인 불가, rust-source-health files[*].ast.shapeHashes has no exact match for {hash}; Rust shape producer does not yet make complete absence claims]"
                )],
            );
        }
        return ShapeLookup {
            kind: ShapeLookupKind::Shape,
            shape: shape.clone(),
            result: ShapeLookupResult::ShapeMatch,
            shape_hash: Some(hash.clone()),
            shape_hash_source: Some(ShapeHashSource::Hash),
            matches,
            citations: vec![format!(
                "[grounded, rust-source-health files[*].ast.shapeHashes matched exact hash {hash}]"
            )],
        };
    }

    if shape.type_literal.is_some() {
        return unavailable(
            shape,
            None,
            None,
            vec![TYPE_LITERAL_UNAVAILABLE_CITATION.to_string()],
        );
    }

    unavailable(
        shape,
        None,
        None,
        vec![FIELD_ONLY_UNAVAILABLE_CITATION.to_string()],
    )
}

fn shape_hash_matches(hash: &str, syntax: &HealthResponse) -> Vec<ShapeMatch> {
    let mut matches = Vec::new();
    for (owner_file, file) in &syntax.files {
        for fact in &file.ast.shape_hashes {
            if fact.hash != hash {
                continue;
            }
            let identity = format!("{owner_file}::{}", fact.name);
            matches.push(ShapeMatch {
                identity,
                owner_file: owner_file.clone(),
                name: fact.name.clone(),
                hash: fact.hash.clone(),
                shape_kind: fact.shape_kind,
                fields: fact.fields.clone(),
                visibility: fact.visibility,
                confidence: fact.confidence,
            });
        }
    }
    matches.sort_by(|left, right| left.identity.cmp(&right.identity));
    matches
}

fn unavailable(
    shape: &ShapeIntent,
    shape_hash: Option<String>,
    shape_hash_source: Option<ShapeHashSource>,
    citations: Vec<String>,
) -> ShapeLookup {
    ShapeLookup {
        kind: ShapeLookupKind::Shape,
        shape: shape.clone(),
        result: ShapeLookupResult::Unavailable,
        shape_hash,
        shape_hash_source,
        matches: Vec::new(),
        citations,
    }
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
