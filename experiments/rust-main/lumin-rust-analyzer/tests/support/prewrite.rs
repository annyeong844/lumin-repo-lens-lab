use std::fs;
use std::process::Output;

use anyhow::{Context, Result};
use serde_json::Value;
use tempfile::TempDir;

use crate::support::command::unified_analyzer_command;

const LIB_RS: &str = r#"pub fn load_task() {}

pub struct EventDispatcher;

impl EventDispatcher {
    pub fn handle_delete(&self) {}
}

macro_rules! generated_handlers {
    () => {};
}

generated_handlers!();

#[cfg(feature = "fast")]
pub fn gated_handler() {}
"#;

const TEST_RS: &str = r#"pub struct TestDispatcher;

impl TestDispatcher {
    pub fn handle_delete(&self) {}
}
"#;

pub struct PreWriteRepo {
    temp: TempDir,
}

impl PreWriteRepo {
    pub fn new() -> Result<Self> {
        let temp = TempDir::new()?;
        fs::create_dir_all(temp.path().join("src"))?;
        fs::create_dir_all(temp.path().join("tests"))?;
        fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"prewrite-case\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nfast = []\n",
        )?;
        fs::write(temp.path().join("src").join("lib.rs"), LIB_RS)?;
        fs::write(temp.path().join("tests").join("helper.rs"), TEST_RS)?;
        Ok(Self { temp })
    }

    pub fn run_json(&self, intent: &str) -> Result<Value> {
        let output = self.run(intent)?;
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&fs::read(self.output_path())?)
            .context("parse rust pre-write artifact")
    }

    pub fn run(&self, intent: &str) -> Result<Output> {
        fs::write(self.intent_path(), intent)?;
        let _ = fs::remove_file(self.output_path());
        unified_analyzer_command()
            .arg("pre-write")
            .arg("--root")
            .arg(self.temp.path())
            .arg("--source-commit")
            .arg("test-source-commit")
            .arg("--intent")
            .arg(self.intent_path())
            .arg("--output")
            .arg(self.output_path())
            .output()
            .context("run rust pre-write analyzer")
    }

    pub fn output_exists(&self) -> bool {
        self.output_path().is_file()
    }

    fn intent_path(&self) -> std::path::PathBuf {
        self.temp.path().join("intent.json")
    }

    fn output_path(&self) -> std::path::PathBuf {
        self.temp.path().join("pre-write.json")
    }
}
