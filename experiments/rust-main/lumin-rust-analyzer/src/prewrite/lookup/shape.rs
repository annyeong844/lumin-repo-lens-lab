use serde::Serialize;

use crate::prewrite::intent::{NormalizedIntent, ShapeIntent};
use lumin_rust_source_health::protocol::{
    AstCallableKind, AstFunctionSignature, AstShapeConfidence, AstShapeField, AstShapeKind,
    AstVisibility, HealthResponse, PathClassification,
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
    matches: Vec<ShapeLookupMatch>,
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
enum ShapeLookupResult {
    #[serde(rename = "SHAPE_MATCH")]
    ShapeMatch,
    #[serde(rename = "SIGNATURE_MATCH")]
    SignatureMatch,
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) enum ShapeHashSource {
    Hash,
    FunctionSignature,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(in crate::prewrite) enum ShapeLookupMatch {
    Shape(ShapeMatch),
    Signature(SignatureMatch),
}

impl ShapeLookupMatch {
    pub(in crate::prewrite) fn identity(&self) -> &str {
        match self {
            Self::Shape(candidate) => &candidate.identity,
            Self::Signature(candidate) => &candidate.identity,
        }
    }

    pub(in crate::prewrite) fn owner_file(&self) -> &str {
        match self {
            Self::Shape(candidate) => &candidate.owner_file,
            Self::Signature(candidate) => &candidate.owner_file,
        }
    }

    pub(in crate::prewrite) fn name(&self) -> &str {
        match self {
            Self::Shape(candidate) => &candidate.name,
            Self::Signature(candidate) => &candidate.name,
        }
    }

    pub(in crate::prewrite) fn is_safe_signature_surface(&self) -> bool {
        match self {
            Self::Shape(_) => false,
            Self::Signature(candidate) => {
                if candidate.policy_excluded {
                    return false;
                }
                candidate.callable_kind == AstCallableKind::Function
                    && matches!(
                        candidate.visibility,
                        AstVisibility::Public | AstVisibility::Crate | AstVisibility::Restricted
                    )
            }
        }
    }

    pub(in crate::prewrite) fn signature_visibility(&self) -> Option<SignatureVisibility> {
        match self {
            Self::Shape(_) => None,
            Self::Signature(candidate) => Some(candidate.visibility_label()),
        }
    }

    pub(in crate::prewrite) fn path_classifications(&self) -> &[PathClassification] {
        match self {
            Self::Shape(candidate) => &candidate.path_classifications,
            Self::Signature(candidate) => &candidate.path_classifications,
        }
    }

    pub(in crate::prewrite) fn policy_excluded(&self) -> bool {
        match self {
            Self::Shape(candidate) => candidate.policy_excluded,
            Self::Signature(candidate) => candidate.policy_excluded,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum SignatureVisibility {
    Exported,
    FileLocal,
    Unknown,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct ShapeMatch {
    pub(in crate::prewrite) identity: String,
    pub(in crate::prewrite) owner_file: String,
    pub(in crate::prewrite) name: String,
    pub(in crate::prewrite) hash: String,
    shape_kind: AstShapeKind,
    fields: Vec<AstShapeField>,
    visibility: lumin_rust_source_health::protocol::AstVisibility,
    confidence: AstShapeConfidence,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    path_classifications: Vec<PathClassification>,
    policy_excluded: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct SignatureMatch {
    pub(in crate::prewrite) identity: String,
    pub(in crate::prewrite) owner_file: String,
    pub(in crate::prewrite) name: String,
    pub(in crate::prewrite) hash: String,
    pub(in crate::prewrite) visibility: AstVisibility,
    pub(in crate::prewrite) callable_kind: AstCallableKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) local_name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    path_classifications: Vec<PathClassification>,
    policy_excluded: bool,
    confidence: &'static str,
}

impl SignatureMatch {
    fn visibility_label(&self) -> SignatureVisibility {
        if self.is_exported_surface() {
            SignatureVisibility::Exported
        } else if matches!(self.visibility, AstVisibility::Private) {
            SignatureVisibility::FileLocal
        } else {
            SignatureVisibility::Unknown
        }
    }

    fn is_exported_surface(&self) -> bool {
        self.callable_kind == AstCallableKind::Function
            && matches!(
                self.visibility,
                AstVisibility::Public | AstVisibility::Crate | AstVisibility::Restricted
            )
    }
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
        if !matches.is_empty() {
            return ShapeLookup {
                kind: ShapeLookupKind::Shape,
                shape: shape.clone(),
                result: ShapeLookupResult::ShapeMatch,
                shape_hash: Some(hash.clone()),
                shape_hash_source: Some(ShapeHashSource::Hash),
                matches: matches.into_iter().map(ShapeLookupMatch::Shape).collect(),
                citations: vec![format!(
                "[grounded, rust-source-health files[*].ast.shapeHashes matched exact hash {hash}]"
            )],
            };
        }

        let matches = function_signature_matches(hash, syntax);
        if !matches.is_empty() {
            return ShapeLookup {
                kind: ShapeLookupKind::Shape,
                shape: shape.clone(),
                result: ShapeLookupResult::SignatureMatch,
                shape_hash: Some(hash.clone()),
                shape_hash_source: Some(ShapeHashSource::FunctionSignature),
                matches: matches
                    .into_iter()
                    .map(ShapeLookupMatch::Signature)
                    .collect(),
                citations: vec![format!(
                    "[grounded, rust-source-health files[*].ast.functionSignatures matched exact hash {hash}]"
                )],
            };
        }

        return unavailable(
            shape,
            Some(hash.clone()),
            Some(ShapeHashSource::Hash),
            vec![format!(
                "[확인 불가, rust-source-health files[*].ast.shapeHashes and files[*].ast.functionSignatures have no exact match for {hash}; Rust producers do not yet make complete absence claims]"
            )],
        );
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
                path_classifications: file.path.classifications.clone(),
                policy_excluded: file.path.suppressed
                    || file.path.classifications.iter().any(|classification| {
                        matches!(
                            classification,
                            PathClassification::Test | PathClassification::Generated
                        )
                    }),
            });
        }
    }
    matches.sort_by(|left, right| left.identity.cmp(&right.identity));
    matches
}

fn function_signature_matches(hash: &str, syntax: &HealthResponse) -> Vec<SignatureMatch> {
    let mut matches = Vec::new();
    for (owner_file, file) in &syntax.files {
        for fact in &file.ast.function_signatures {
            if fact.hash != hash {
                continue;
            }
            matches.push(signature_match(
                owner_file,
                fact,
                file.path.classifications.clone(),
                file.path.suppressed,
            ));
        }
    }
    matches.sort_by(|left, right| left.identity.cmp(&right.identity));
    matches
}

fn signature_match(
    owner_file: &str,
    fact: &AstFunctionSignature,
    path_classifications: Vec<PathClassification>,
    path_suppressed: bool,
) -> SignatureMatch {
    let identity = match &fact.owner {
        None => format!("{owner_file}::{}", fact.name),
        Some(owner) => match &owner.trait_path {
            None => format!("{owner_file}::{}#{}", owner.target, fact.name),
            Some(trait_path) => {
                format!(
                    "{owner_file}::{} as {trait_path}#{}",
                    owner.target, fact.name
                )
            }
        },
    };
    let local_name = if fact.name == "default" {
        None
    } else {
        Some(fact.name.clone())
    };
    SignatureMatch {
        identity,
        owner_file: owner_file.to_string(),
        name: fact.name.clone(),
        hash: fact.hash.clone(),
        visibility: fact.visibility,
        callable_kind: fact.callable_kind,
        local_name,
        policy_excluded: path_suppressed
            || path_classifications.iter().any(|classification| {
                matches!(
                    classification,
                    PathClassification::Test | PathClassification::Generated
                )
            }),
        path_classifications,
        confidence: "high",
    }
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
