use anyhow::Result;
use serde_json::Value;

use super::support::{dependency_lookup, run_dependency_fixture};

#[test]
fn prewrite_dependency_lane_reports_new_package_for_undeclared_dependency() -> Result<()> {
    let artifact = run_dependency_fixture()?;

    let new_package = dependency_lookup(&artifact, "serde_yaml")?;
    assert_eq!(new_package["result"], "NEW_PACKAGE");
    assert_eq!(new_package["declaredIn"], Value::Null);
    Ok(())
}
