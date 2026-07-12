use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestCompanionUpdateInput {
    #[serde(default)]
    pub topology_mermaid_path: Option<String>,
    #[serde(default)]
    pub audit_summary_path: Option<String>,
    #[serde(default)]
    pub review_pack_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestCompanionUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topology_mermaid: Option<TopologyMermaidBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_summary: Option<MarkdownCompanionBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_pack: Option<ReviewPackBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyMermaidBlock {
    pub path: String,
    pub format: &'static str,
    pub source: &'static str,
    #[serde(rename = "use")]
    pub use_: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownCompanionBlock {
    pub path: String,
    pub format: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewPackBlock {
    pub path: String,
    pub format: &'static str,
    #[serde(rename = "use")]
    pub use_: &'static str,
}

pub fn build_manifest_companion_update(
    input: ManifestCompanionUpdateInput,
) -> Result<ManifestCompanionUpdate> {
    let topology_mermaid = match input.topology_mermaid_path {
        Some(path) => {
            validate_path("manifest-companion-update: topologyMermaidPath", &path)?;
            Some(TopologyMermaidBlock {
                path,
                format: "markdown",
                source: "topology.json",
                use_: "human visual companion; topology.json remains authoritative for exact citations",
            })
        }
        None => None,
    };
    let audit_summary = match input.audit_summary_path {
        Some(path) => {
            validate_path("manifest-companion-update: auditSummaryPath", &path)?;
            Some(MarkdownCompanionBlock {
                path,
                format: "markdown",
            })
        }
        None => None,
    };
    let review_pack = match input.review_pack_path {
        Some(path) => {
            validate_path("manifest-companion-update: reviewPackPath", &path)?;
            Some(ReviewPackBlock {
                path,
                format: "markdown",
                use_: "main assistant reads lanes as artifact briefs; if using built-in reviewer subagents, translate lanes into focused codebase-reading tasks with file:line evidence; the engine never calls external APIs",
            })
        }
        None => None,
    };

    Ok(ManifestCompanionUpdate {
        topology_mermaid,
        audit_summary,
        review_pack,
    })
}

fn validate_path(label: &str, path: &str) -> Result<()> {
    if path.trim().is_empty() {
        bail!("{label} must be a non-empty string");
    }
    Ok(())
}
