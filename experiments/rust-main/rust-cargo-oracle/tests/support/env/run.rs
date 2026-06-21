use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::{
    run_oracle, CargoCheckMode, CargoTargetDirMode, OracleOptions,
    DEFAULT_TARGETED_CARGO_CHECK_PACKAGES,
};
use serde_json::Value;

use super::super::process_env::{lock_process_env, with_clean_compilation_env};

use super::RealCargoEnv;

impl RealCargoEnv {
    pub fn run(&self) -> Result<Value> {
        self.run_with_mode(CargoCheckMode::CargoCheck)
    }

    pub fn run_with_mode(&self, cargo_check_mode: CargoCheckMode) -> Result<Value> {
        let _guard = lock_process_env();
        self.run_unlocked(cargo_check_mode)
    }

    pub fn run_targeted(&self, target_paths: Vec<String>) -> Result<Value> {
        self.run_targeted_with_cap(target_paths, DEFAULT_TARGETED_CARGO_CHECK_PACKAGES)
    }

    pub fn run_targeted_with_cap(
        &self,
        target_paths: Vec<String>,
        targeted_package_cap: usize,
    ) -> Result<Value> {
        let _guard = lock_process_env();
        self.run_unlocked_with_target_paths(
            CargoCheckMode::TargetedCargoCheck,
            target_paths,
            targeted_package_cap,
        )
    }

    pub fn run_targeted_with_timeout(
        &self,
        target_paths: Vec<String>,
        targeted_package_cap: usize,
        timeout_ms: u64,
    ) -> Result<Value> {
        let _guard = lock_process_env();
        self.run_unlocked_with_target_paths_and_timeout(
            CargoCheckMode::TargetedCargoCheck,
            target_paths,
            targeted_package_cap,
            timeout_ms,
        )
    }

    pub fn run_with_clean_compilation_env(
        &self,
        cargo_check_mode: CargoCheckMode,
    ) -> Result<Value> {
        let _guard = lock_process_env();
        with_clean_compilation_env(|| self.run_unlocked(cargo_check_mode))
    }

    pub fn run_with_target_dir_mode(
        &self,
        cargo_check_mode: CargoCheckMode,
        cargo_target_dir_mode: CargoTargetDirMode,
    ) -> Result<Value> {
        let _guard = lock_process_env();
        self.run_unlocked_with_target_dir_mode(cargo_check_mode, cargo_target_dir_mode)
    }

    pub fn run_unlocked(&self, cargo_check_mode: CargoCheckMode) -> Result<Value> {
        self.run_unlocked_with_target_paths(
            cargo_check_mode,
            Vec::new(),
            DEFAULT_TARGETED_CARGO_CHECK_PACKAGES,
        )
    }

    fn run_unlocked_with_target_dir_mode(
        &self,
        cargo_check_mode: CargoCheckMode,
        cargo_target_dir_mode: CargoTargetDirMode,
    ) -> Result<Value> {
        self.run_unlocked_with_target_paths_timeout_and_target_dir_mode(
            cargo_check_mode,
            Vec::new(),
            DEFAULT_TARGETED_CARGO_CHECK_PACKAGES,
            10_000,
            cargo_target_dir_mode,
        )
    }

    fn run_unlocked_with_target_paths(
        &self,
        cargo_check_mode: CargoCheckMode,
        target_paths: Vec<String>,
        targeted_package_cap: usize,
    ) -> Result<Value> {
        self.run_unlocked_with_target_paths_and_timeout(
            cargo_check_mode,
            target_paths,
            targeted_package_cap,
            10_000,
        )
    }

    fn run_unlocked_with_target_paths_and_timeout(
        &self,
        cargo_check_mode: CargoCheckMode,
        target_paths: Vec<String>,
        targeted_package_cap: usize,
        timeout_ms: u64,
    ) -> Result<Value> {
        self.run_unlocked_with_target_paths_timeout_and_target_dir_mode(
            cargo_check_mode,
            target_paths,
            targeted_package_cap,
            timeout_ms,
            CargoTargetDirMode::IsolatedTemp,
        )
    }

    fn run_unlocked_with_target_paths_timeout_and_target_dir_mode(
        &self,
        cargo_check_mode: CargoCheckMode,
        target_paths: Vec<String>,
        targeted_package_cap: usize,
        timeout_ms: u64,
        cargo_target_dir_mode: CargoTargetDirMode,
    ) -> Result<Value> {
        let artifact = run_oracle(OracleOptions {
            root: self.root.clone(),
            output: Some(self.root.join("semantic-health.json")),
            cargo_bin: "cargo".to_string(),
            timeout_ms,
            features: None,
            package_name: None,
            repo_root: self.repo_root.clone(),
            cargo_check_mode,
            cargo_target_dir_mode,
            target_paths,
            targeted_package_cap,
        })?;
        serde_json::to_value(artifact).context("serialize semantic health artifact")
    }
}
