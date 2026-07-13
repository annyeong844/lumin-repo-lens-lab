use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use super::rules::{sarif_rules, tool_info_uri};
use super::support::{insert_string, string_field, SarifState};
use super::TOOL_VERSION;

pub(super) struct ArtifactProjectionInput<'a> {
    pub(super) root: &'a str,
    pub(super) scan_root: String,
    pub(super) generated: String,
    pub(super) state: SarifState,
    pub(super) symbols: Option<&'a Value>,
    pub(super) dead_classify: Option<&'a Value>,
    pub(super) topology: Option<&'a Value>,
    pub(super) discipline: Option<&'a Value>,
}

pub(super) fn build_artifact(input: ArtifactProjectionInput<'_>) -> Value {
    let mut by_level = BTreeMap::from([
        ("error".to_string(), 0usize),
        ("warning".to_string(), 0usize),
        ("note".to_string(), 0usize),
    ]);
    for result in &input.state.results {
        if let Some(level) = string_field(result, "level") {
            *by_level.entry(level).or_default() += 1;
        }
    }

    json!({
        "$schema": "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "lumin-repo-lens-lab",
                    "version": TOOL_VERSION,
                    "informationUri": tool_info_uri(),
                    "shortDescription": {
                        "text": "AST-based repository structural audit with layered evidence (AST + runtime + git history)."
                    },
                    "rules": sarif_rules()
                }
            },
            "invocations": [{
                "executionSuccessful": true,
                "startTimeUtc": input.generated,
                "endTimeUtc": input.generated,
                "workingDirectory": { "uri": format!("file://{}", input.root) }
            }],
            "originalUriBaseIds": {
                "SRCROOT": { "uri": format!("file://{}/", input.root) }
            },
            "results": input.state.results,
            "properties": {
                "artifactsUsed": input.state.artifacts_used,
                "scanRoot": input.scan_root,
                "generatedAt": input.generated,
                "totalFindings": by_level.values().sum::<usize>(),
                "upstreamWarnings": upstream_warnings(
                    input.symbols,
                    input.dead_classify,
                    input.topology,
                    input.discipline,
                )
            }
        }]
    })
}

fn upstream_warnings(
    symbols: Option<&Value>,
    dead_classify: Option<&Value>,
    topology: Option<&Value>,
    discipline: Option<&Value>,
) -> Value {
    let mut warnings = Vec::new();
    append_warnings(&mut warnings, "symbols.json", symbols);
    append_warnings(&mut warnings, "dead-classify.json", dead_classify);
    append_warnings(&mut warnings, "topology.json", topology);
    append_warnings(&mut warnings, "discipline.json", discipline);
    Value::Array(warnings)
}

fn append_warnings(warnings: &mut Vec<Value>, source: &str, artifact: Option<&Value>) {
    let Some(meta_warnings) = artifact
        .and_then(|artifact| artifact.get("meta"))
        .and_then(|meta| meta.get("warnings"))
        .and_then(Value::as_array)
    else {
        return;
    };
    for warning in meta_warnings {
        let mut object = Map::new();
        insert_string(&mut object, "source", source);
        if let Some(warning) = warning.as_object() {
            for (key, value) in warning {
                object.insert(key.clone(), value.clone());
            }
        }
        warnings.push(Value::Object(object));
    }
}
