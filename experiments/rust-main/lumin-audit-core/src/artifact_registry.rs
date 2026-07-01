use anyhow::Result;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const RUST_ANALYZER_ARTIFACT: &str = "rust-analyzer-health.latest.json";
const JSON_SUFFIX: &str = ".json";
const MARKDOWN_SUFFIX: &str = ".md";

const ARTIFACT_CANDIDATES: &[&str] = &[
    "triage.json",
    "topology.json",
    "discipline.json",
    "call-graph.json",
    "barrels.json",
    "shape-index.json",
    "function-clones.json",
    "block-clones.json",
    "framework-resource-surfaces.json",
    "resolver-capabilities.json",
    "resolver-diagnostics.json",
    "symbols.json",
    "unused-deps.json",
    "entry-surface.json",
    "module-reachability.json",
    "dead-classify.json",
    "runtime-evidence.json",
    "staleness.json",
    "fix-plan.json",
    "checklist-facts.json",
    RUST_ANALYZER_ARTIFACT,
    "producer-performance.json",
    "canon-drift.json",
    "topology.mermaid.md",
    "audit-summary.latest.md",
    "audit-review-pack.latest.md",
    "lumin-repo-lens-lab.sarif",
];

pub fn collect_produced_artifacts(
    out_dir: &Path,
    rust_analysis_usable: bool,
) -> Result<Vec<String>> {
    let mut produced = BTreeSet::new();
    for name in ARTIFACT_CANDIDATES {
        if *name == RUST_ANALYZER_ARTIFACT && !rust_analysis_usable {
            continue;
        }
        if out_dir.join(name).exists() {
            produced.insert((*name).to_string());
        }
    }

    let entries = match fs::read_dir(out_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(produced.into_iter().collect()),
    };

    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_dynamic_artifact_name(&name) {
            produced.insert(name);
        }
    }

    Ok(produced.into_iter().collect())
}

fn is_dynamic_artifact_name(name: &str) -> bool {
    is_canon_drift_markdown(name)
        || is_pre_write_advisory(name)
        || is_post_write_delta(name)
        || is_any_inventory(name, "pre")
        || is_any_inventory(name, "post")
}

fn is_canon_drift_markdown(name: &str) -> bool {
    has_required_middle(name, "canon-drift.", MARKDOWN_SUFFIX)
}

fn is_pre_write_advisory(name: &str) -> bool {
    name == "pre-write-advisory.json"
        || has_required_middle(name, "pre-write-advisory.", JSON_SUFFIX)
}

fn is_post_write_delta(name: &str) -> bool {
    name == "post-write-delta.json" || has_required_middle(name, "post-write-delta.", JSON_SUFFIX)
}

fn is_any_inventory(name: &str, phase: &str) -> bool {
    let prefix = format!("any-inventory.{phase}.");
    has_required_middle(name, &prefix, JSON_SUFFIX)
}

fn has_required_middle(name: &str, prefix: &str, suffix: &str) -> bool {
    name.starts_with(prefix) && name.ends_with(suffix) && name.len() > prefix.len() + suffix.len()
}
