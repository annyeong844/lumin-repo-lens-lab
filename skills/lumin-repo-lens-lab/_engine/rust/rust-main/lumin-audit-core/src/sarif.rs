use anyhow::{bail, Result};
use serde_json::Value;

mod dead_exports;
mod projection;
mod protocol;
mod rules;
mod secondary;
mod support;

pub use protocol::{SarifRequest, SARIF_REQUEST_SCHEMA_VERSION};

use dead_exports::collect_dead_export_results;
use projection::{build_artifact, ArtifactProjectionInput};
use secondary::collect_secondary_results;
use support::{present_artifact, slash_path, SarifState};

const TOOL_VERSION: &str = "0.0.0-lab.0";

pub fn build_sarif_artifact(request: SarifRequest) -> Result<Value> {
    if request.schema_version != SARIF_REQUEST_SCHEMA_VERSION {
        bail!(
            "sarif-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if request.root.trim().is_empty() {
        bail!("sarif-artifact: root must be non-empty");
    }

    let SarifRequest {
        schema_version: _,
        root: scan_root,
        generated,
        fix_plan,
        runtime_evidence,
        staleness,
        dead_classify,
        symbols,
        topology,
        discipline,
        barrels,
    } = request;
    let root = slash_path(&scan_root);
    let generated = generated.unwrap_or_else(|| "1970-01-01T00:00:00.000Z".to_string());
    let fix_plan = present_artifact(fix_plan.as_ref());
    let runtime_evidence = present_artifact(runtime_evidence.as_ref());
    let staleness = present_artifact(staleness.as_ref());
    let dead_classify = present_artifact(dead_classify.as_ref());
    let symbols = present_artifact(symbols.as_ref());
    let topology = present_artifact(topology.as_ref());
    let discipline = present_artifact(discipline.as_ref());
    let barrels = present_artifact(barrels.as_ref());
    let mut state = SarifState::default();

    collect_dead_export_results(
        &mut state,
        &root,
        fix_plan,
        runtime_evidence,
        staleness,
        dead_classify,
        symbols,
    );
    collect_secondary_results(&mut state, &root, topology, discipline, barrels);

    Ok(build_artifact(ArtifactProjectionInput {
        root: &root,
        scan_root,
        generated,
        state,
        symbols,
        dead_classify,
        topology,
        discipline,
    }))
}

#[cfg(test)]
mod tests;
