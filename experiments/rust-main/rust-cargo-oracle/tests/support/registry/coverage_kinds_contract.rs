use std::collections::BTreeSet;

use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::protocol::CoverageKind;

use crate::support::registry::{cargo_check_oracle, serialized_string};

pub fn assert_coverage_kinds_are_declared_in_registry() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    let registry_kinds = cargo_check["coverageKinds"]
        .as_array()
        .context("coverageKinds array")?
        .iter()
        .filter_map(|entry| entry["coverageKind"].as_str())
        .collect::<BTreeSet<_>>();

    for coverage_kind in CoverageKind::EMITTED_BY_ORACLE {
        let coverage_kind = serialized_string(coverage_kind)?;
        assert!(
            registry_kinds.contains(coverage_kind.as_str()),
            "coverage kind {coverage_kind} is not declared in oracle-registry.json"
        );
    }
    Ok(())
}
