use anyhow::Result;

use super::support::{citations, dependency_lookup, run_dependency_fixture};

#[test]
fn prewrite_dependency_lane_reports_declared_zero_consumers_without_cleanup_claims() -> Result<()> {
    let artifact = run_dependency_fixture()?;

    let pretty = dependency_lookup(&artifact, "pretty_assertions")?;
    assert_eq!(pretty["result"], "DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS");
    assert_eq!(pretty["declaredIn"], "dev-dependencies");
    assert_eq!(pretty["existingImports"]["observedImportCount"], 0);
    assert!(citations(pretty)
        .all(|citation| { !citation.contains("unused") && !citation.contains("cleanup") }));

    let build_dependency = dependency_lookup(&artifact, "cc")?;
    assert_eq!(
        build_dependency["result"],
        "DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS"
    );
    assert_eq!(build_dependency["declaredIn"], "build-dependencies");
    Ok(())
}
