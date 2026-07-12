use serde::Serialize;
use std::path::Path;

const LIVING_AUDIT_DOC_CANDIDATES: &[&str] = &[
    "docs/current/audit/lumin-structural-audit.md",
    "LUMIN_REPO_LENS.md",
    "LUMIN_AUDIT.md",
    "TECH_DEBT_AUDIT.md",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LivingAuditSummary {
    pub preferred_path: &'static str,
    pub existing_docs: Vec<LivingAuditDocument>,
    pub action: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LivingAuditDocument {
    pub path: String,
    pub absolute_path: String,
}

pub fn summarize_living_audit(root: &Path) -> LivingAuditSummary {
    let existing_docs = LIVING_AUDIT_DOC_CANDIDATES
        .iter()
        .filter_map(|candidate| {
            let absolute = root.join(candidate);
            absolute.is_file().then(|| LivingAuditDocument {
                path: (*candidate).to_string(),
                absolute_path: absolute.to_string_lossy().into_owned(),
            })
        })
        .collect::<Vec<_>>();
    let action = if existing_docs.is_empty() {
        "create-only-on-explicit-tracking-request"
    } else {
        "read-and-update-before-final-answer"
    };

    LivingAuditSummary {
        preferred_path: LIVING_AUDIT_DOC_CANDIDATES[0],
        existing_docs,
        action,
    }
}
