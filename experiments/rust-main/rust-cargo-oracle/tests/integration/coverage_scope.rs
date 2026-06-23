use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::{CargoCheckMode, CargoTargetDirMode};
use std::path::PathBuf;

use crate::support::{coverage::coverage, real_cargo_env::RealCargoEnv};
use crate::{dependency_scope_contract, dependency_unavailable_contract, metadata_only_contract};

#[test]
fn build_finished_false_does_not_prove_clean() -> Result<()> {
    let env = RealCargoEnv::type_error()?;
    let artifact = env.run()?;

    let stream = coverage(&artifact, "cov.cargo-check.cargo-event-stream")?;
    assert_eq!(stream["status"], "ran");

    let absence = coverage(&artifact, "cov.cargo-check.absence-clean")?;
    assert_eq!(absence["status"], "unavailable");
    assert!(!absence
        .as_object()
        .context("absence-clean coverage object")?
        .contains_key("clean"));
    assert!(absence["reason"]
        .as_str()
        .is_some_and(|reason| reason.contains("build-finished success was false")));
    Ok(())
}

#[test]
fn multiline_cargo_config_rustflags_feed_scope_cfg_set() -> Result<()> {
    let env = RealCargoEnv::success()?;
    env.write_cargo_config(
        "[build]\nrustflags = [\n  \"--cfg\",\n  \"lumin_config_array\",\n  \"--cfg=lumin_config_equals\",\n]\n",
    )?;

    let artifact = env.run_with_clean_compilation_env(CargoCheckMode::MetadataOnly)?;
    let scope = &coverage(&artifact, "cov.cargo-check.absence-clean")?["scope"];
    let cfgs = scope["cfgSet"].as_array().context("cfgSet")?;

    assert!(cfgs.iter().any(|cfg| cfg == "lumin_config_array"));
    assert!(cfgs.iter().any(|cfg| cfg == "lumin_config_equals"));
    let source = scope["cfgSetSource"].as_str().context("cfgSetSource")?;
    assert!(source.starts_with("cargo-config:"));
    assert!(source.ends_with("config.toml"));
    Ok(())
}

#[test]
fn dependency_events_do_not_replace_selected_scope_target() -> Result<()> {
    dependency_scope_contract::assert_dependency_events_do_not_replace_selected_scope()
}

#[test]
fn dependency_primary_error_is_coverage_unavailable_not_user_finding() -> Result<()> {
    dependency_unavailable_contract::assert_dependency_primary_error_is_not_user_finding()
}

#[test]
fn multi_target_fallback_scope_does_not_pick_an_arbitrary_target() -> Result<()> {
    let env = RealCargoEnv::multi_target_success()?;
    let artifact = env.run()?;
    let scope = &coverage(&artifact, "cov.cargo-check.absence-clean")?["scope"];

    assert_eq!(scope["target"], "<multiple>");
    assert_eq!(scope["targets"].as_array().context("targets")?.len(), 2);
    Ok(())
}

#[test]
fn metadata_only_mode_records_explicit_not_run_coverage_without_cargo_findings() -> Result<()> {
    metadata_only_contract::assert_metadata_only_without_cargo_findings()
}

#[test]
fn reusable_temp_target_mode_uses_owned_temp_cache_without_repo_target_dir() -> Result<()> {
    let env = RealCargoEnv::success()?;
    let artifact =
        env.run_with_target_dir_mode(CargoCheckMode::CargoCheck, CargoTargetDirMode::ReusableTemp)?;

    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirMode"],
        "reusable-temp"
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["repoTargetDirUsed"],
        false
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["ownedTempTargetDir"],
        true
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["incrementalDisabled"],
        true
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["debugSymbolsDisabled"],
        true
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["staleCleanupOwnedTempTargetDirs"],
        true
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["staleIsolatedTargetDirMaxAgeSeconds"],
        86_400
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["staleReusableTargetDirMaxAgeSeconds"],
        604_800
    );
    assert!(!env.path_exists("target"));
    let target_dir = PathBuf::from(
        artifact["meta"]["input"]["cargoTargetDir"]
            .as_str()
            .context("cargoTargetDir")?,
    );
    assert!(target_dir.exists(), "{}", target_dir.display());
    assert!(target_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("lumin-rust-cargo-oracle-reusable-target-")));
    Ok(())
}

#[test]
fn targeted_cargo_check_orders_independent_small_package_before_local_dependency() -> Result<()> {
    let env = RealCargoEnv::targeted_local_dependency_ranking_workspace()?;
    let artifact = env.run_targeted(vec![
        "a-dep/src/lib.rs".to_string(),
        "b-plain/src/lib.rs".to_string(),
    ])?;

    assert_eq!(artifact["oraclePlan"]["status"], "ran");
    assert_eq!(
        artifact["oraclePlan"]["sampleLimits"]["targetPathExamples"],
        10
    );
    assert_eq!(
        artifact["oraclePlan"]["sampleLimits"]["omittedPackageExamples"],
        5
    );
    assert_eq!(
        artifact["oraclePlan"]["sampleLimits"]["selectedPackageTargetPathExamples"],
        5
    );
    assert_eq!(artifact["oraclePlan"]["selectedPackageCount"], 2);
    assert_eq!(artifact["oraclePlan"]["selectedTargetPathCount"], 2);
    assert_eq!(artifact["oraclePlan"]["omittedTargetPathCount"], 0);
    assert_eq!(
        artifact["oraclePlan"]["selectedPackages"][0]["packageName"],
        "b-plain"
    );
    assert!(artifact["oraclePlan"]["omittedPackageExamples"]
        .as_array()
        .context("omitted package examples")?
        .is_empty());
    Ok(())
}

#[test]
fn targeted_multi_package_nonzero_exit_does_not_mark_stream_clean() -> Result<()> {
    let env = RealCargoEnv::targeted_success_then_build_script_failure_workspace()?;
    let artifact = env.run_targeted(vec![
        "a-clean/src/lib.rs".to_string(),
        "b-build-fails/src/lib.rs".to_string(),
    ])?;

    assert_eq!(artifact["oraclePlan"]["status"], "ran");
    assert_eq!(artifact["oraclePlan"]["selectedPackageCount"], 2);
    assert_eq!(
        artifact["oraclePlan"]["selectedPackages"][0]["packageName"],
        "a-clean"
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoArgs"],
        serde_json::json!([
            "check",
            "--message-format=json",
            "--package",
            "a-clean",
            "--package",
            "b-build-fails"
        ])
    );

    let absence = coverage(&artifact, "cov.cargo-check.absence-clean")?;
    assert_eq!(absence["status"], "unavailable");
    assert!(!absence
        .as_object()
        .context("absence-clean coverage object")?
        .contains_key("clean"));
    assert!(absence["reason"]
        .as_str()
        .is_some_and(|reason| reason.contains("cargo check exited with status")));
    Ok(())
}

#[test]
fn targeted_repeated_dependency_diagnostics_are_deduped() -> Result<()> {
    let env = RealCargoEnv::targeted_duplicate_dependency_diagnostic_workspace()?;
    let artifact = env.run_targeted(vec![
        "a-app/src/lib.rs".to_string(),
        "b-app/src/lib.rs".to_string(),
    ])?;

    assert_eq!(artifact["oraclePlan"]["status"], "ran");
    assert_eq!(artifact["oraclePlan"]["selectedPackageCount"], 2);

    let diagnostics = artifact["diagnostics"].as_array().context("diagnostics")?;
    let e0308_diagnostics = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic["normalized"]["codeValue"] == "E0308")
        .collect::<Vec<_>>();
    assert_eq!(e0308_diagnostics.len(), 1);
    let span = e0308_diagnostics[0]["primarySpans"]
        .as_array()
        .and_then(|spans| spans.first())
        .context("E0308 primary span")?;
    assert_eq!(span["fileName"], "shared-bad/src/lib.rs");
    assert_eq!(span["lineStart"], 1);
    assert_eq!(span["columnStart"], 25);
    Ok(())
}
