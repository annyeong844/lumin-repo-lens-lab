use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::orchestration_plan::AuditProfile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestMetaInput {
    pub generated: String,
    pub profile: String,
    pub root: String,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestMeta {
    pub generated: String,
    pub tool: &'static str,
    pub profile: AuditProfile,
    pub root: String,
    pub output: String,
}

pub fn build_manifest_meta(input: ManifestMetaInput) -> Result<ManifestMeta> {
    validate_required("generated", &input.generated)?;
    validate_required("profile", &input.profile)?;
    validate_required("root", &input.root)?;
    validate_required("output", &input.output)?;
    let profile = AuditProfile::parse(&input.profile)
        .context("manifest-meta: invalid --profile <quick|full|ci>")?;

    Ok(ManifestMeta {
        generated: input.generated,
        tool: "audit-repo.mjs",
        profile,
        root: input.root,
        output: input.output,
    })
}

fn validate_required(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("manifest-meta: {field} must be a non-empty string");
    }
    Ok(())
}
