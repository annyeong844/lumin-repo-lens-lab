use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::{run_oracle, CargoCheckMode, CargoTargetDirMode, OracleOptions};
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
        let _guard = lock_process_env();
        self.run_unlocked_with_target_paths(CargoCheckMode::TargetedCargoCheck, target_paths)
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
        self.run_unlocked_with_target_paths(cargo_check_mode, Vec::new())
    }

    fn run_unlocked_with_target_dir_mode(
        &self,
        cargo_check_mode: CargoCheckMode,
        cargo_target_dir_mode: CargoTargetDirMode,
    ) -> Result<Value> {
        self.run_unlocked_with_target_paths_and_target_dir_mode(
            cargo_check_mode,
            Vec::new(),
            cargo_target_dir_mode,
        )
    }

    fn run_unlocked_with_target_paths(
        &self,
        cargo_check_mode: CargoCheckMode,
        target_paths: Vec<String>,
    ) -> Result<Value> {
        self.run_unlocked_with_target_paths_and_target_dir_mode(
            cargo_check_mode,
            target_paths,
            CargoTargetDirMode::IsolatedTemp,
        )
    }

    fn run_unlocked_with_target_paths_and_target_dir_mode(
        &self,
        cargo_check_mode: CargoCheckMode,
        target_paths: Vec<String>,
        cargo_target_dir_mode: CargoTargetDirMode,
    ) -> Result<Value> {
        let artifact = run_oracle(OracleOptions {
            root: self.root.clone(),
            output: Some(self.root.join("semantic-health.json")),
            cargo_bin: "cargo".to_string(),
            features: None,
            package_name: None,
            repo_root: self.repo_root.clone(),
            cargo_check_mode,
            cargo_target_dir_mode,
            target_paths,
        })?;
        serde_json::to_value(artifact).context("serialize semantic health artifact")
    }
}
