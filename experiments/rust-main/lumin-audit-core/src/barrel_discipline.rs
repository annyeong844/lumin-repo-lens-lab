use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Value};

pub const BARREL_DISCIPLINE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-barrel-discipline-producer-request.v1";

const TOOL_NAME: &str = "check-barrel-discipline.mjs";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BarrelDisciplineRequest {
    pub schema_version: String,
    pub root: String,
    pub generated: String,
    pub mode: String,
    #[serde(default)]
    pub skipped: bool,
    pub reason: Option<String>,
    pub summary: Option<Value>,
    pub by_package: Option<Value>,
}

pub fn build_barrel_discipline_artifact(request: BarrelDisciplineRequest) -> Result<Value> {
    if request.schema_version != BARREL_DISCIPLINE_REQUEST_SCHEMA_VERSION {
        bail!(
            "barrel-discipline-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    if request.skipped {
        let reason = request
            .reason
            .as_deref()
            .filter(|reason| !reason.trim().is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!("barrel-discipline-artifact: skipped reason is required")
            })?;
        return Ok(json!({
            "meta": {
                "generated": request.generated,
                "root": request.root,
                "tool": TOOL_NAME,
            },
            "mode": request.mode,
            "skipped": true,
            "reason": reason,
        }));
    }

    let summary = request
        .summary
        .filter(Value::is_object)
        .ok_or_else(|| anyhow::anyhow!("barrel-discipline-artifact: summary object is required"))?;
    let by_package = request.by_package.filter(Value::is_object).ok_or_else(|| {
        anyhow::anyhow!("barrel-discipline-artifact: byPackage object is required")
    })?;

    Ok(json!({
        "meta": {
            "generated": request.generated,
            "root": request.root,
            "mode": request.mode,
            "tool": TOOL_NAME,
        },
        "summary": summary,
        "byPackage": by_package,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_single_package_skip_shape() -> Result<()> {
        let request = serde_json::from_value(json!({
            "schemaVersion": BARREL_DISCIPLINE_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "generated": "2026-07-04T00:00:00.000Z",
            "mode": "single-package",
            "skipped": true,
            "reason": "Single-package repo has no workspace barrels to discipline. This check is monorepo-only."
        }))?;

        let artifact = build_barrel_discipline_artifact(request)?;

        assert_eq!(artifact["meta"]["tool"], TOOL_NAME);
        assert_eq!(artifact["mode"], "single-package");
        assert_eq!(artifact["skipped"], true);
        assert!(artifact["meta"].get("mode").is_none());
        Ok(())
    }

    #[test]
    fn builds_monorepo_barrel_projection() -> Result<()> {
        let request = serde_json::from_value(json!({
            "schemaVersion": BARREL_DISCIPLINE_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "generated": "2026-07-04T00:00:00.000Z",
            "mode": "monorepo",
            "summary": {
                "workspacePackages": ["@scope/pkg"],
                "filesScanned": 3,
                "totalImports": 2,
                "parseErrors": 0,
                "unreadableFiles": 0
            },
            "byPackage": {
                "@scope/pkg": {
                    "rootImports": 1,
                    "subpathImports": 1,
                    "total": 2,
                    "policyCompliance": "50.0%",
                    "rootImportDisabledByEslint": 0,
                    "subpathBreakdown": { "@scope/pkg/button": 1 },
                    "sampleRootImporters": []
                }
            }
        }))?;

        let artifact = build_barrel_discipline_artifact(request)?;

        assert_eq!(artifact["meta"]["mode"], "monorepo");
        assert_eq!(artifact["summary"]["filesScanned"], 3);
        assert_eq!(
            artifact["byPackage"]["@scope/pkg"]["policyCompliance"],
            "50.0%"
        );
        assert!(artifact.get("mode").is_none());
        Ok(())
    }

    #[test]
    fn rejects_unknown_request_schema() -> Result<()> {
        let request = serde_json::from_value(json!({
            "schemaVersion": "other",
            "root": "C:/repo",
            "generated": "2026-07-04T00:00:00.000Z",
            "mode": "single-package",
            "skipped": true,
            "reason": "skip"
        }))?;

        let err = match build_barrel_discipline_artifact(request) {
            Ok(_) => panic!("schema should reject"),
            Err(error) => error.to_string(),
        };

        assert!(err.contains("unsupported schemaVersion"));
        Ok(())
    }

    #[test]
    fn rejects_incomplete_monorepo_shape() -> Result<()> {
        let request = serde_json::from_value(json!({
            "schemaVersion": BARREL_DISCIPLINE_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "generated": "2026-07-04T00:00:00.000Z",
            "mode": "monorepo",
            "summary": {}
        }))?;

        let err = match build_barrel_discipline_artifact(request) {
            Ok(_) => panic!("incomplete monorepo shape should reject"),
            Err(error) => error.to_string(),
        };

        assert!(err.contains("byPackage object is required"));
        Ok(())
    }
}
