use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};

pub const EXPORT_ACTION_SAFETY_REQUEST_SCHEMA_VERSION: &str =
    "lumin-export-action-safety-producer-request.v1";

const TOOL_NAME: &str = "export-action-safety.mjs";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportActionSafetyRequest {
    pub schema_version: String,
    pub root: String,
    pub generated: String,
    #[serde(default)]
    pub findings: Vec<Value>,
    #[serde(default)]
    pub warnings: Vec<Value>,
}

pub fn build_export_action_safety_artifact(request: ExportActionSafetyRequest) -> Result<Value> {
    if request.schema_version != EXPORT_ACTION_SAFETY_REQUEST_SCHEMA_VERSION {
        bail!(
            "export-action-safety-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let mut by_id = Map::new();
    for finding in &request.findings {
        let id = finding
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!("export-action-safety-artifact: finding missing non-empty id")
            })?;
        by_id.insert(id.to_string(), finding.clone());
    }

    Ok(json!({
        "meta": {
            "tool": TOOL_NAME,
            "generated": request.generated,
            "root": request.root,
            "schemaVersion": 1,
            "total": request.findings.len(),
            "warnings": request.warnings,
        },
        "findings": request.findings,
        "byId": by_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> Result<ExportActionSafetyRequest> {
        Ok(serde_json::from_value(json!({
            "schemaVersion": EXPORT_ACTION_SAFETY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "generated": "2026-07-04T00:00:00.000Z",
            "findings": [
                {
                    "id": "dead-export:src/api.ts:unused:4",
                    "file": "src/api.ts",
                    "symbol": "unused",
                    "line": 4,
                    "bucket": "A",
                    "safeAction": {
                        "kind": "demote_export_declaration",
                        "proofComplete": true
                    },
                    "actionBlockers": []
                },
                {
                    "id": "dead-export:src/api.ts:blocked:8",
                    "file": "src/api.ts",
                    "symbol": "blocked",
                    "line": 8,
                    "bucket": "C",
                    "safeAction": null,
                    "actionBlockers": ["local-refs-present"]
                }
            ],
            "warnings": [
                { "file": "src/broken.ts", "symbol": "broken", "message": "parse failed" }
            ]
        }))?)
    }

    #[test]
    fn builds_export_action_safety_projection_from_js_facts() -> Result<()> {
        let artifact = build_export_action_safety_artifact(request()?)?;

        assert_eq!(artifact["meta"]["tool"], TOOL_NAME);
        assert_eq!(artifact["meta"]["schemaVersion"], 1);
        assert_eq!(artifact["meta"]["total"], 2);
        assert_eq!(
            artifact["meta"]["warnings"].as_array().map_or(0, Vec::len),
            1
        );
        assert_eq!(artifact["findings"].as_array().map_or(0, Vec::len), 2);
        assert_eq!(
            artifact["byId"]["dead-export:src/api.ts:unused:4"]["safeAction"]["kind"],
            "demote_export_declaration"
        );
        assert_eq!(
            artifact["byId"]["dead-export:src/api.ts:blocked:8"]["actionBlockers"][0],
            "local-refs-present"
        );
        Ok(())
    }

    #[test]
    fn rejects_unknown_request_schema() -> Result<()> {
        let mut request = request()?;
        request.schema_version = "other".to_string();
        let err = match build_export_action_safety_artifact(request) {
            Ok(_) => panic!("schema should reject"),
            Err(error) => error.to_string(),
        };

        assert!(err.contains("unsupported schemaVersion"));
        Ok(())
    }

    #[test]
    fn rejects_findings_without_id() -> Result<()> {
        let mut request = request()?;
        request.findings.push(json!({ "file": "src/api.ts" }));
        let err = match build_export_action_safety_artifact(request) {
            Ok(_) => panic!("finding without id should reject"),
            Err(error) => error.to_string(),
        };

        assert!(err.contains("finding missing non-empty id"));
        Ok(())
    }
}
