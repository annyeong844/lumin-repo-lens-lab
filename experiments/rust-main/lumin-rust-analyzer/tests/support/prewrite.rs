use std::fs;
use std::path::Path;
use std::process::Output;

use anyhow::{Context, Result};
use lumin_rust_source_health::{
    analyze_root,
    protocol::{HealthResponse, DEFAULT_WORKER_STACK_BYTES},
    RustSourceHealthOptions,
};
use serde_json::Value;
use tempfile::TempDir;

use crate::support::command::unified_analyzer_command;

const LIB_RS: &str = r#"pub fn load_task() {}

pub fn fetch_user() {}

pub fn create_repository() {
    fn get_world() {}
    fn list_library_docs() {}
    fn delete_world() {}
    fn normalize_input() {}

    get_world();
    list_library_docs();
    delete_world();
    normalize_input();
}

pub struct EventDispatcher;

impl EventDispatcher {
    pub fn handle_delete(&self) {}
    pub fn fetch_user(&self) {}
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

    pub fn source_health(&self) -> Result<HealthResponse> {
        analyze_root(RustSourceHealthOptions {
            root: self.temp.path().to_path_buf(),
            source_commit: "test-source-commit".to_string(),
            thread_count: None,
            worker_stack_bytes: DEFAULT_WORKER_STACK_BYTES,
        })
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

    pub fn write_bytes(&self, relative_path: impl AsRef<Path>, bytes: &[u8]) -> Result<()> {
        let path = self.temp.path().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)?;
        Ok(())
    }

    pub fn output_exists(&self) -> bool {
        self.output_path().is_file()
    }

    pub fn root_path(&self) -> &Path {
        self.temp.path()
    }

    fn intent_path(&self) -> std::path::PathBuf {
        self.temp.path().join("intent.json")
    }

    fn output_path(&self) -> std::path::PathBuf {
        self.temp.path().join("pre-write.json")
    }
}

pub fn dependency_lookup<'a>(artifact: &'a Value, dependency: &str) -> Result<&'a Value> {
    artifact["dependencyLookups"]
        .as_array()
        .context("dependencyLookups array")?
        .iter()
        .find(|lookup| lookup["depName"] == dependency)
        .with_context(|| format!("dependency lookup {dependency}"))
}

pub fn citations(lookup: &Value) -> impl Iterator<Item = &str> {
    lookup["citations"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

pub fn examples(lookup: &Value) -> impl Iterator<Item = &Value> {
    lookup["existingImports"]["examples"]
        .as_array()
        .into_iter()
        .flatten()
}
