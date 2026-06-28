mod support;

#[path = "support/registry/absence_claim_sets_contract.rs"]
mod absence_claim_sets_contract;
#[path = "support/registry/claim_vocabulary_contract/mod.rs"]
mod claim_vocabulary_contract;
#[path = "support/registry/classifier_rules_declared_contract.rs"]
mod classifier_rules_declared_contract;
#[path = "support/registry/classifier_rules_known_contract.rs"]
mod classifier_rules_known_contract;
#[path = "support/registry/coverage_kinds_contract.rs"]
mod coverage_kinds_contract;
#[path = "support/coverage_contract/dependency_scope.rs"]
mod dependency_scope_contract;
#[path = "support/coverage_contract/dependency_unavailable.rs"]
mod dependency_unavailable_contract;
#[path = "support/registry/diagnostic_schema/mod.rs"]
mod diagnostic_schema;
#[path = "support/coverage_contract/metadata_only.rs"]
mod metadata_only_contract;
#[path = "support/cli_usage/package_scope.rs"]
mod package_scope_usage;
#[path = "support/registry/coverage_contract.rs"]
mod registry_coverage_contract;

#[path = "integration/cli_usage.rs"]
mod cli_usage;
#[path = "integration/coverage_scope.rs"]
mod coverage_scope;
#[path = "integration/diagnostics.rs"]
mod diagnostics;
#[path = "integration/input_identity.rs"]
mod input_identity;
#[path = "integration/registry_contracts.rs"]
mod registry_contracts;
#[path = "integration/safe_actions.rs"]
mod safe_actions;
