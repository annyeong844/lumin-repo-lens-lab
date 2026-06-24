use anyhow::{Context, Result};

use super::support::{citations, dependency_lookup, examples, run_dependency_fixture};

#[test]
fn prewrite_dependency_lane_reports_declared_consumed_packages() -> Result<()> {
    let artifact = run_dependency_fixture()?;

    assert_eq!(artifact["coverage"]["dependencies"], "ran");
    let anyhow = dependency_lookup(&artifact, "anyhow")?;
    assert_eq!(anyhow["result"], "DEPENDENCY_AVAILABLE");
    assert_eq!(anyhow["declaredIn"], "dependencies");
    assert_eq!(anyhow["existingImports"]["countConfidence"], "grounded");
    assert!(
        anyhow["existingImports"]["observedImportCount"]
            .as_u64()
            .context("anyhow observed count")?
            > 0
    );
    assert!(citations(anyhow).any(|citation| {
        citation.contains("Cargo.toml.dependencies['anyhow'] declares anyhow")
    }));

    let tracing = dependency_lookup(&artifact, "tracing-subscriber")?;
    assert_eq!(tracing["result"], "DEPENDENCY_AVAILABLE");
    assert!(examples(tracing).any(|example| {
        example["fromSpec"]
            .as_str()
            .is_some_and(|from_spec| from_spec.contains("tracing_subscriber"))
    }));

    let serde_json = dependency_lookup(&artifact, "serde_json")?;
    assert_eq!(serde_json["result"], "DEPENDENCY_AVAILABLE");
    assert!(examples(serde_json).any(|example| {
        example["fromSpec"]
            .as_str()
            .is_some_and(|from_spec| from_spec.contains("serde_json::json"))
    }));
    Ok(())
}
