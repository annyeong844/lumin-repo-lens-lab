use serde::Serialize;

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
    pub(in crate::prewrite) fn shape_hash(citations: Vec<String>) -> Self {
        Self {
            evidence_lane: UnavailableEvidenceLane::ShapeHash,
            status: UnavailableEvidenceStatus::Unavailable,
            reason: "lookup-unavailable",
            artifact: "rust-source-health",
            citations,
        }
    }

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
