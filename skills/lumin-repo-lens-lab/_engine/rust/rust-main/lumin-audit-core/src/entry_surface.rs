use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const ENTRY_SURFACE_SCHEMA_VERSION: &str = "entry-surface.v1";
pub const ENTRY_SURFACE_REQUEST_SCHEMA_VERSION: &str = "lumin-entry-surface-producer-request.v1";

const TOOL_NAME: &str = "build-entry-surface.mjs";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrySurfaceRequest {
    pub schema_version: String,
    pub root: String,
    pub generated: String,
    #[serde(default)]
    pub include_tests: bool,
    #[serde(default)]
    pub known_files: Vec<String>,
    #[serde(default)]
    pub parse_error_count: usize,
    #[serde(default)]
    pub submodule_by_file: BTreeMap<String, String>,
    #[serde(default)]
    pub public_api: PublicApiFacts,
    #[serde(default)]
    pub script: ScriptFacts,
    #[serde(default)]
    pub html: HtmlFacts,
    #[serde(default)]
    pub framework: EntryLaneFacts,
    #[serde(default)]
    pub config: EntryLaneFacts,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryLaneFacts {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub evidence_by_file: BTreeMap<String, Vec<Value>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicApiFacts {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub evidence_by_file: BTreeMap<String, Vec<Value>>,
    #[serde(default)]
    pub transitive_added: usize,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptFacts {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub evidence_by_file: BTreeMap<String, Vec<Value>>,
    #[serde(default)]
    pub unsupported_raw_count: usize,
    #[serde(default)]
    pub unsupported_sample_limit: usize,
    #[serde(default)]
    pub unsupported: Vec<Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtmlFacts {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub evidence_by_file: BTreeMap<String, Vec<Value>>,
    #[serde(default)]
    pub unresolved: Vec<Value>,
}

pub fn build_entry_surface_artifact(request: EntrySurfaceRequest) -> Result<Value> {
    if request.schema_version != ENTRY_SURFACE_REQUEST_SCHEMA_VERSION {
        bail!(
            "entry-surface-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let known_files = sorted_files(request.known_files);
    let public_api_files = sorted_files(request.public_api.files.clone());
    let script_files = sorted_files(request.script.files.clone());
    let html_files = sorted_files(request.html.files.clone());
    let framework_files = sorted_files(request.framework.files.clone());
    let config_files = sorted_files(request.config.files.clone());

    let mut entry_file_set = BTreeSet::new();
    extend_set(&mut entry_file_set, &public_api_files);
    extend_set(&mut entry_file_set, &script_files);
    extend_set(&mut entry_file_set, &html_files);
    extend_set(&mut entry_file_set, &framework_files);
    extend_set(&mut entry_file_set, &config_files);
    let entry_files = entry_file_set.into_iter().collect::<Vec<_>>();

    let mut evidence_by_file = BTreeMap::new();
    merge_evidence(&mut evidence_by_file, request.public_api.evidence_by_file);
    merge_evidence(&mut evidence_by_file, request.script.evidence_by_file);
    merge_evidence(&mut evidence_by_file, request.html.evidence_by_file);
    merge_evidence(&mut evidence_by_file, request.framework.evidence_by_file);
    merge_evidence(&mut evidence_by_file, request.config.evidence_by_file);

    let unsupported_script_entrypoints = sorted_records(request.script.unsupported);
    let unresolved_html_entrypoints = sorted_records(request.html.unresolved);
    let global_completeness =
        if request.parse_error_count > 0 || !unresolved_html_entrypoints.is_empty() {
            "medium"
        } else {
            "high"
        };
    let completeness_by_submodule = completeness_by_submodule(
        &known_files,
        &entry_files,
        &request.submodule_by_file,
        global_completeness,
    );

    Ok(json!({
        "meta": {
            "tool": TOOL_NAME,
            "generated": request.generated,
            "root": request.root,
            "schemaVersion": ENTRY_SURFACE_SCHEMA_VERSION,
            "supports": {
                "publicApiFiles": true,
                "scriptEntrypointFiles": true,
                "unsupportedScriptEntrypoints": true,
                "htmlEntrypointFiles": true,
                "unresolvedHtmlEntrypoints": true,
                "frameworkEntrypointFiles": true,
                "configEntrypointFiles": true,
                "submoduleCompleteness": true,
            },
            "includeTests": request.include_tests,
            "transitivePublicReexports": request.public_api.transitive_added,
            "knownFileCount": known_files.len(),
        },
        "publicApiFiles": public_api_files,
        "scriptEntrypointFiles": script_files,
        "unsupportedScriptEntrypointCount": request.script.unsupported_raw_count,
        "unsupportedScriptEntrypointSampleLimit": request.script.unsupported_sample_limit,
        "unsupportedScriptEntrypoints": unsupported_script_entrypoints,
        "htmlEntrypointFiles": html_files,
        "unresolvedHtmlEntrypoints": unresolved_html_entrypoints,
        "frameworkEntrypointFiles": framework_files,
        "configEntrypointFiles": config_files,
        "entryFiles": entry_files,
        "evidenceByFile": evidence_by_file,
        "globalCompleteness": global_completeness,
        "completenessBySubmodule": completeness_by_submodule,
    }))
}

fn normalize_rel(path: impl AsRef<str>) -> String {
    path.as_ref().replace('\\', "/")
}

fn sorted_files(files: Vec<String>) -> Vec<String> {
    files
        .into_iter()
        .map(normalize_rel)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn extend_set(target: &mut BTreeSet<String>, files: &[String]) {
    for file in files {
        target.insert(file.clone());
    }
}

fn merge_evidence(
    target: &mut BTreeMap<String, Vec<Value>>,
    evidence: BTreeMap<String, Vec<Value>>,
) {
    for (file, values) in evidence {
        target
            .entry(normalize_rel(file))
            .or_default()
            .extend(values);
    }
}

fn sorted_records(records: Vec<Value>) -> Vec<Value> {
    let mut records = records;
    records.sort_by_key(record_key);
    records
}

fn record_key(record: &Value) -> [String; 5] {
    [
        record_string(record, "packageDir")
            .or_else(|| record_string(record, "htmlFile"))
            .unwrap_or_default(),
        record_string(record, "scriptName").unwrap_or_default(),
        record_string(record, "reason").unwrap_or_default(),
        record_string(record, "src").unwrap_or_default(),
        record_string(record, "resolvedFile").unwrap_or_default(),
    ]
}

fn record_string(record: &Value, field: &str) -> Option<String> {
    record.get(field).map(|value| match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    })
}

fn completeness_by_submodule(
    known_files: &[String],
    entry_files: &[String],
    submodule_by_file: &BTreeMap<String, String>,
    completeness: &str,
) -> BTreeMap<String, String> {
    let mut submodules = BTreeSet::new();
    for file in known_files.iter().chain(entry_files.iter()) {
        submodules.insert(
            submodule_by_file
                .get(file)
                .cloned()
                .unwrap_or_else(|| ".".to_string()),
        );
    }
    submodules
        .into_iter()
        .map(|submodule| (submodule, completeness.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> Result<EntrySurfaceRequest> {
        Ok(serde_json::from_value(json!({
            "schemaVersion": ENTRY_SURFACE_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "generated": "2026-07-04T00:00:00.000Z",
            "includeTests": false,
            "knownFiles": ["src\\api.ts", "src/page.tsx", "vite.config.ts"],
            "parseErrorCount": 0,
            "submoduleByFile": {
                "src/api.ts": "src",
                "src/cli.ts": "src",
                "src/page.tsx": "src",
                "vite.config.ts": "."
            },
            "publicApi": {
                "files": ["src/api.ts"],
                "transitiveAdded": 1,
                "evidenceByFile": {
                    "src/api.ts": [{ "source": "package-json.exports" }]
                }
            },
            "script": {
                "files": ["src/cli.ts"],
                "unsupportedRawCount": 2,
                "unsupportedSampleLimit": 20,
                "unsupported": [
                    { "packageDir": "pkg-b", "scriptName": "start", "reason": "shell" },
                    { "packageDir": "pkg-a", "scriptName": "dev", "reason": "unsupported" }
                ],
                "evidenceByFile": {
                    "src/cli.ts": [{ "source": "package-script" }]
                }
            },
            "html": {
                "files": ["src/page.tsx"],
                "unresolved": [{ "htmlFile": "index.html", "src": "./missing.ts" }],
                "evidenceByFile": {
                    "src/page.tsx": [{ "source": "html-module" }]
                }
            },
            "framework": {
                "files": ["src/page.tsx"],
                "evidenceByFile": {
                    "src/page.tsx": [{ "source": "framework-policy" }]
                }
            },
            "config": {
                "files": ["vite.config.ts"],
                "evidenceByFile": {
                    "vite.config.ts": [{ "source": "config-file-convention" }]
                }
            }
        }))?)
    }

    #[test]
    fn builds_entry_surface_projection_from_js_facts() -> Result<()> {
        let artifact = build_entry_surface_artifact(request()?)?;

        assert_eq!(artifact["meta"]["tool"], TOOL_NAME);
        assert_eq!(
            artifact["meta"]["schemaVersion"],
            ENTRY_SURFACE_SCHEMA_VERSION
        );
        assert_eq!(artifact["meta"]["includeTests"], false);
        assert_eq!(artifact["meta"]["knownFileCount"], 3);
        assert_eq!(artifact["meta"]["transitivePublicReexports"], 1);
        assert_eq!(
            artifact["entryFiles"],
            json!(["src/api.ts", "src/cli.ts", "src/page.tsx", "vite.config.ts"])
        );
        assert_eq!(artifact["globalCompleteness"], "medium");
        assert_eq!(artifact["completenessBySubmodule"]["."], "medium");
        assert_eq!(artifact["completenessBySubmodule"]["src"], "medium");
        assert_eq!(
            artifact["unsupportedScriptEntrypoints"][0]["packageDir"],
            "pkg-a"
        );
        assert_eq!(
            artifact["evidenceByFile"]["src/page.tsx"]
                .as_array()
                .map_or(0, Vec::len),
            2
        );
        Ok(())
    }

    #[test]
    fn high_completeness_when_no_parse_errors_or_unresolved_html() -> Result<()> {
        let mut request = request()?;
        request.html.unresolved.clear();
        let artifact = build_entry_surface_artifact(request)?;

        assert_eq!(artifact["globalCompleteness"], "high");
        Ok(())
    }

    #[test]
    fn rejects_unknown_request_schema() -> Result<()> {
        let mut request = request()?;
        request.schema_version = "other".to_string();
        let err = match build_entry_surface_artifact(request) {
            Ok(_) => panic!("schema should reject"),
            Err(error) => error.to_string(),
        };

        assert!(err.contains("unsupported schemaVersion"));
        Ok(())
    }
}
