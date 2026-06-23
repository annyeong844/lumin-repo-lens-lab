use serde::Serialize;

use crate::prewrite::intent::ShapeIntent;
use crate::prewrite::lookup::unavailable::UnavailableEvidence;

use super::candidate::ShapeLookupMatch;

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
    matches: Vec<ShapeLookupMatch>,
    citations: Vec<String>,
}

impl ShapeLookup {
    pub(in crate::prewrite::lookup::shape) fn matched(
        shape: &ShapeIntent,
        result: ShapeLookupResult,
        shape_hash: &str,
        shape_hash_source: ShapeHashSource,
        matches: Vec<ShapeLookupMatch>,
        citations: Vec<String>,
    ) -> Self {
        Self {
            kind: ShapeLookupKind::Shape,
            shape: shape.clone(),
            result,
            shape_hash: Some(shape_hash.to_string()),
            shape_hash_source: Some(shape_hash_source),
            matches,
            citations,
        }
    }

    pub(in crate::prewrite::lookup::shape) fn unavailable(
        shape: &ShapeIntent,
        shape_hash: Option<String>,
        shape_hash_source: Option<ShapeHashSource>,
        citations: Vec<String>,
    ) -> Self {
        Self {
            kind: ShapeLookupKind::Shape,
            shape: shape.clone(),
            result: ShapeLookupResult::Unavailable,
            shape_hash,
            shape_hash_source,
            matches: Vec::new(),
            citations,
        }
    }

    pub(in crate::prewrite) fn unavailable_evidence(&self) -> UnavailableEvidence {
        UnavailableEvidence::shape_hash(self.citations.clone())
    }

    pub(in crate::prewrite) fn is_unavailable(&self) -> bool {
        self.result == ShapeLookupResult::Unavailable
    }

    pub(in crate::prewrite) fn is_shape_match(&self) -> bool {
        self.result == ShapeLookupResult::ShapeMatch && !self.matches.is_empty()
    }

    pub(in crate::prewrite) fn is_signature_match(&self) -> bool {
        self.result == ShapeLookupResult::SignatureMatch && !self.matches.is_empty()
    }

    pub(in crate::prewrite) fn is_match(&self) -> bool {
        self.is_shape_match() || self.is_signature_match()
    }

    pub(in crate::prewrite) fn shape_hash(&self) -> Option<&str> {
        self.shape_hash.as_deref()
    }

    pub(in crate::prewrite) fn matches(&self) -> &[ShapeLookupMatch] {
        &self.matches
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
enum ShapeLookupKind {
    #[serde(rename = "shape")]
    Shape,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(in crate::prewrite::lookup::shape) enum ShapeLookupResult {
    #[serde(rename = "SHAPE_MATCH")]
    ShapeMatch,
    #[serde(rename = "SIGNATURE_MATCH")]
    SignatureMatch,
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite::lookup::shape) enum ShapeHashSource {
    Hash,
    FunctionSignature,
}
