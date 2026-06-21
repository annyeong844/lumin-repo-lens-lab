use std::collections::BTreeSet;

use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::protocol::ClassificationRule;

use crate::support::registry::{cargo_check_oracle, serialized_string};

pub fn assert_registry_diagnostic_rules_are_known_to_rust_code() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    let code_rules = ClassificationRule::EMITTED_BY_CLASSIFIER
        .iter()
        .map(|rule| serialized_string(*rule))
        .collect::<Result<BTreeSet<_>>>()?;

    for rule in cargo_check["diagnosticClassification"]
        .as_array()
        .context("diagnosticClassification array")?
        .iter()
        .filter_map(|entry| entry["rule"].as_str())
    {
        assert!(
            code_rules.contains(rule),
            "registry rule {rule} is not represented in ClassificationRule"
        );
    }
    Ok(())
}
