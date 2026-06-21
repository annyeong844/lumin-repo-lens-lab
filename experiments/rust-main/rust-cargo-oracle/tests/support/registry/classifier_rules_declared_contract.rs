use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::protocol::ClassificationRule;

use crate::support::registry::{cargo_check_oracle, serialized_string};

pub fn assert_classifier_rules_are_declared_in_registry() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    let registry_rules = cargo_check["diagnosticClassification"]
        .as_array()
        .context("diagnosticClassification array")?
        .iter()
        .filter_map(|entry| entry["rule"].as_str())
        .collect::<Vec<_>>();

    for rule in ClassificationRule::EMITTED_BY_CLASSIFIER {
        let rule = serialized_string(rule)?;
        assert!(
            registry_rules.contains(&rule.as_str()),
            "classifier rule {rule} is not declared in oracle-registry.json"
        );
    }
    Ok(())
}
