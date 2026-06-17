use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::{run_oracle, OracleOptions};
use serde_json::{json, Value};
use tempfile::TempDir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn warning_lint_is_rule_backed_without_blocking_verified_error_clean() -> Result<()> {
    let env = FixtureEnv::new("warning-only", 0)?;
    let artifact = env.run()?;

    let findings = artifact["findings"].as_array().context("findings array")?;
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["confidence"]["tier"], "rule-backed");
    assert_eq!(
        findings[0]["confidence"]["claimKind"],
        "rule-backed.rust.rustc-lint-diagnostic"
    );
    assert!(findings[0]["confidence"]["authorityIds"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        findings[0]["confidence"]["ruleIds"][0],
        "rust.rustc.lint-diagnostic"
    );

    let absence = coverage(&artifact, "cov.cargo-check.absence-clean")?;
    assert_eq!(absence["status"], "ran");
    assert_eq!(absence["clean"], true);
    assert_eq!(absence["cleanKind"], "verified-rustc-error-absence");
    Ok(())
}

#[test]
fn dependency_primary_error_is_coverage_unavailable_not_user_finding() -> Result<()> {
    let env = FixtureEnv::new("dependency-error-outside-workspace", 101)?;
    let artifact = env.run()?;

    assert!(artifact["findings"]
        .as_array()
        .context("findings array")?
        .is_empty());
    let absence = coverage(&artifact, "cov.cargo-check.absence-clean")?;
    assert_eq!(absence["status"], "unavailable");
    assert!(absence["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("non-user-code primary error diagnostic"));
    Ok(())
}

#[test]
fn dependency_package_id_blocks_user_finding_even_when_metadata_omits_dependency_package(
) -> Result<()> {
    let stdout =
        fs::read_to_string(fixture_root().join("dependency-error-outside-workspace.stdout.jsonl"))?;
    let env = FixtureEnv::new_with_stdout_and_metadata(
        "dependency-error-missing-metadata-package",
        &stdout,
        101,
        metadata_without_dependency,
    )?;
    let artifact = env.run()?;

    assert!(artifact["findings"]
        .as_array()
        .context("findings array")?
        .is_empty());
    let absence = coverage(&artifact, "cov.cargo-check.absence-clean")?;
    assert_eq!(absence["status"], "unavailable");
    assert!(absence["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("non-user-code primary error diagnostic"));
    Ok(())
}

#[test]
fn dependency_events_do_not_replace_selected_scope_target() -> Result<()> {
    let env = FixtureEnv::new("dependency-error-outside-workspace", 101)?;
    let artifact = env.run()?;
    let scope = &coverage(&artifact, "cov.cargo-check.absence-clean")?["scope"];

    assert_eq!(scope["package"], "app");
    assert_eq!(scope["target"], "app");
    assert_eq!(scope["targets"][0]["packageName"], "app");
    assert_eq!(scope["targets"][0]["targetName"], "app");
    assert_eq!(
        scope["targets"][0]["source"],
        "cargo-metadata-default-selection"
    );
    Ok(())
}

#[test]
fn empty_cargo_stdout_is_unavailable_not_ran_stream() -> Result<()> {
    let env = FixtureEnv::new_with_stdout("empty-stdout", "", 101)?;
    let artifact = env.run()?;

    let stream = coverage(&artifact, "cov.cargo-check.cargo-event-stream")?;
    assert_eq!(stream["status"], "unavailable");
    assert_eq!(stream["streamParseStatus"], "no-json-events");

    let absence = coverage(&artifact, "cov.cargo-check.absence-clean")?;
    assert_eq!(absence["status"], "unavailable");
    assert!(!absence.as_object().unwrap().contains_key("clean"));
    Ok(())
}

#[test]
fn build_finished_without_success_does_not_prove_clean() -> Result<()> {
    let stdout = "{\"reason\":\"build-finished\"}\n";
    let env = FixtureEnv::new_with_stdout("build-finished-missing-success", stdout, 0)?;
    let artifact = env.run()?;

    let stream = coverage(&artifact, "cov.cargo-check.cargo-event-stream")?;
    assert_eq!(stream["status"], "ran");

    let absence = coverage(&artifact, "cov.cargo-check.absence-clean")?;
    assert_eq!(absence["status"], "unavailable");
    assert!(!absence.as_object().unwrap().contains_key("clean"));
    assert!(absence["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("build-finished success was not true"));
    Ok(())
}

#[test]
fn multi_target_fallback_scope_does_not_pick_an_arbitrary_target() -> Result<()> {
    let stdout = "{\"reason\":\"build-finished\",\"success\":true}\n";
    let env = FixtureEnv::new_with_stdout_and_metadata(
        "multi-target-success",
        stdout,
        0,
        metadata_with_two_default_targets,
    )?;
    let artifact = env.run()?;
    let scope = &coverage(&artifact, "cov.cargo-check.absence-clean")?["scope"];

    assert_eq!(scope["target"], "<multiple>");
    assert_eq!(scope["targets"].as_array().context("targets")?.len(), 2);
    Ok(())
}

#[test]
fn analysis_input_hash_changes_when_cargo_config_changes() -> Result<()> {
    let env = FixtureEnv::new("success-clean", 0)?;
    let before = env.run()?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("before analysisInputSetHash")?
        .to_string();

    fs::create_dir_all(env.root.join(".cargo"))?;
    fs::write(
        env.root.join(".cargo").join("config.toml"),
        "[build]\nrustflags = [\"--cfg\", \"lumin_config_hash_test\"]\n",
    )?;
    let after = env.run()?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("after analysisInputSetHash")?
        .to_string();

    assert_ne!(before, after);
    Ok(())
}

#[test]
fn analysis_input_hash_changes_when_rustflags_changes() -> Result<()> {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let previous = std::env::var_os("RUSTFLAGS");
    std::env::remove_var("RUSTFLAGS");
    let env = FixtureEnv::new("success-clean", 0)?;
    let before = env.run()?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("before analysisInputSetHash")?
        .to_string();

    std::env::set_var("RUSTFLAGS", "--cfg lumin_env_hash_test");
    let after = env.run()?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("after analysisInputSetHash")?
        .to_string();

    match previous {
        Some(value) => std::env::set_var("RUSTFLAGS", value),
        None => std::env::remove_var("RUSTFLAGS"),
    }
    assert_ne!(before, after);
    Ok(())
}

#[test]
fn large_cargo_stdout_is_drained_while_process_runs() -> Result<()> {
    let success = fs::read_to_string(fixture_root().join("success-clean.stdout.jsonl"))?;
    let stdout = format!("{}\n{}", " ".repeat(1_000_000), success);
    let env = FixtureEnv::new_with_stdout("large-stdout", &stdout, 0)?;
    let artifact = env.run()?;

    let stream = coverage(&artifact, "cov.cargo-check.cargo-event-stream")?;
    assert_eq!(stream["status"], "ran");
    assert_eq!(stream["streamParseStatus"], "complete");
    Ok(())
}

#[test]
fn e_code_user_error_is_verified_rustc_error_diagnostic() -> Result<()> {
    let env = FixtureEnv::new("type-error-e0308", 101)?;
    let artifact = env.run()?;

    let findings = artifact["findings"].as_array().context("findings array")?;
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["confidence"]["tier"], "verified");
    assert_eq!(
        findings[0]["confidence"]["claimKind"],
        "verified.rust.rustc-error-diagnostic"
    );
    assert_eq!(
        findings[0]["confidence"]["authorityIds"][0],
        "rust.rustc.error-diagnostic"
    );
    Ok(())
}

#[test]
fn provenance_records_the_actual_cargo_binary() -> Result<()> {
    let env = FixtureEnv::new("type-error-e0308", 101)?;
    let artifact = env.run()?;
    let cargo_bin = env.fake_cargo.to_string_lossy();

    assert_eq!(artifact["meta"]["input"]["cargoBin"], cargo_bin.as_ref());
    assert_eq!(artifact["meta"]["input"]["cargoArgs"][0], "check");
    assert_eq!(
        artifact["findings"][0]["source"]["commandArgs"][0],
        cargo_bin.as_ref()
    );
    assert!(artifact["findings"][0]["source"]["command"]
        .as_str()
        .unwrap_or_default()
        .starts_with(cargo_bin.as_ref()));
    Ok(())
}

#[test]
fn artifact_marks_analysis_input_set_as_incomplete_for_reuse() -> Result<()> {
    let env = FixtureEnv::new("success-clean", 0)?;
    let artifact = env.run()?;

    assert_eq!(artifact["meta"]["analysisInputSetComplete"], false);
    assert!(artifact["meta"]["missingInfluenceKinds"]
        .as_array()
        .context("missingInfluenceKinds")?
        .iter()
        .any(|kind| kind == "build-script-runtime-inputs"));
    Ok(())
}

struct FixtureEnv {
    _temp: TempDir,
    root: PathBuf,
    fake_cargo: PathBuf,
    repo_root: PathBuf,
}

impl FixtureEnv {
    fn new(name: &str, exit_code: i32) -> Result<Self> {
        let stdout = fs::read_to_string(fixture_root().join(format!("{name}.stdout.jsonl")))
            .with_context(|| format!("failed to read fixture stdout for {name}"))?;
        Self::new_with_stdout(name, &stdout, exit_code)
    }

    fn new_with_stdout(name: &str, stdout: &str, exit_code: i32) -> Result<Self> {
        Self::new_with_stdout_and_metadata(name, stdout, exit_code, metadata_for_fixture)
    }

    fn new_with_stdout_and_metadata(
        name: &str,
        stdout: &str,
        exit_code: i32,
        metadata_builder: fn(&Path, &str, &str) -> Value,
    ) -> Result<Self> {
        let temp = TempDir::new()?;
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("src"))?;
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nbad_dep = { path = \"bad_dep\" }\n")?;
        fs::write(root.join("src").join("lib.rs"), "pub fn app() {}\n")?;
        fs::create_dir_all(root.join("bad_dep").join("src"))?;
        fs::write(
            root.join("bad_dep").join("Cargo.toml"),
            "[package]\nname = \"bad_dep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::write(
            root.join("bad_dep").join("src").join("lib.rs"),
            "pub fn dep() {}\n",
        )?;

        let metadata_path = temp.path().join("metadata.json");
        fs::write(
            &metadata_path,
            serde_json::to_vec_pretty(&metadata_builder(&root, name, stdout))?,
        )?;
        let stdout_path = temp.path().join(format!("{name}.stdout.jsonl"));
        fs::write(&stdout_path, stdout)?;
        let fake_cargo = write_fake_cargo(temp.path(), &metadata_path, &stdout_path, exit_code)?;
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()?;

        Ok(Self {
            _temp: temp,
            root,
            fake_cargo,
            repo_root,
        })
    }

    fn run(&self) -> Result<Value> {
        run_oracle(OracleOptions {
            root: self.root.clone(),
            output: Some(self.root.join("semantic-health.json")),
            cargo_bin: self.fake_cargo.to_string_lossy().to_string(),
            timeout_ms: 10_000,
            features: None,
            package_name: None,
            repo_root: self.repo_root.clone(),
        })
    }
}

fn coverage<'a>(artifact: &'a Value, id: &str) -> Result<&'a Value> {
    artifact["coverage"]
        .as_array()
        .context("coverage array")?
        .iter()
        .find(|entry| entry["id"] == id)
        .with_context(|| format!("missing coverage entry {id}"))
}

fn metadata_for_fixture(root: &Path, fixture_name: &str, stdout: &str) -> Value {
    let observed_package_id = first_package_id(stdout);
    let app_id = if fixture_name.starts_with("dependency-error") {
        "path+file:///fixture/app#0.1.0".to_string()
    } else {
        observed_package_id
            .clone()
            .unwrap_or_else(|| "path+file:///fixture/app#0.1.0".to_string())
    };
    let dep_id = if fixture_name.starts_with("dependency-error") {
        observed_package_id.unwrap_or_else(|| "path+file:///fixture/bad_dep#0.1.0".to_string())
    } else {
        "path+file:///fixture/bad_dep#0.1.0".to_string()
    };
    json!({
        "packages": [
            {
                "name": "app",
                "version": "0.1.0",
                "id": app_id,
                "manifest_path": path_string(&root.join("Cargo.toml")),
                "targets": [{
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "name": "app",
                    "src_path": path_string(&root.join("src").join("lib.rs")),
                    "edition": "2021",
                    "required_features": []
                }]
            },
            {
                "name": "bad_dep",
                "version": "0.1.0",
                "id": dep_id,
                "manifest_path": path_string(&root.join("bad_dep").join("Cargo.toml")),
                "targets": [{
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "name": "bad_dep",
                    "src_path": path_string(&root.join("bad_dep").join("src").join("lib.rs")),
                    "edition": "2021",
                    "required_features": []
                }]
            }
        ],
        "workspace_members": [app_id],
        "workspace_default_members": [app_id],
        "workspace_root": path_string(root),
        "target_directory": path_string(&root.join("target")),
        "resolve": {
            "root": app_id,
            "nodes": [
                {"id": app_id, "features": ["default"]},
                {"id": dep_id, "features": []}
            ]
        }
    })
}

fn metadata_without_dependency(root: &Path, _fixture_name: &str, _stdout: &str) -> Value {
    let app_id = "path+file:///fixture/app#0.1.0";
    json!({
        "packages": [
            {
                "name": "app",
                "version": "0.1.0",
                "id": app_id,
                "manifest_path": path_string(&root.join("Cargo.toml")),
                "targets": [{
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "name": "app",
                    "src_path": path_string(&root.join("src").join("lib.rs")),
                    "edition": "2021",
                    "required_features": []
                }]
            }
        ],
        "workspace_members": [app_id],
        "workspace_default_members": [app_id],
        "workspace_root": path_string(root),
        "target_directory": path_string(&root.join("target")),
        "resolve": {
            "root": app_id,
            "nodes": [
                {"id": app_id, "features": ["default"]}
            ]
        }
    })
}

fn metadata_with_two_default_targets(root: &Path, _fixture_name: &str, _stdout: &str) -> Value {
    let app_id = "path+file:///fixture/app#0.1.0";
    json!({
        "packages": [
            {
                "name": "app",
                "version": "0.1.0",
                "id": app_id,
                "manifest_path": path_string(&root.join("Cargo.toml")),
                "targets": [
                    {
                        "kind": ["lib"],
                        "crate_types": ["lib"],
                        "name": "app",
                        "src_path": path_string(&root.join("src").join("lib.rs")),
                        "edition": "2021",
                        "required_features": []
                    },
                    {
                        "kind": ["bin"],
                        "crate_types": ["bin"],
                        "name": "app_cli",
                        "src_path": path_string(&root.join("src").join("main.rs")),
                        "edition": "2021",
                        "required_features": []
                    }
                ]
            }
        ],
        "workspace_members": [app_id],
        "workspace_default_members": [app_id],
        "workspace_root": path_string(root),
        "target_directory": path_string(&root.join("target")),
        "resolve": {
            "root": app_id,
            "nodes": [
                {"id": app_id, "features": ["default"]}
            ]
        }
    })
}

fn first_package_id(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find_map(|event| {
            event
                .get("package_id")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("tests")
        .join("fixtures")
        .join("m7-cargo-json-diagnostic-capture-v4")
}

#[cfg(windows)]
fn write_fake_cargo(
    dir: &Path,
    metadata_path: &Path,
    stdout_path: &Path,
    exit_code: i32,
) -> Result<PathBuf> {
    let script = dir.join("fake-cargo.cmd");
    fs::write(
        &script,
        format!(
            "@echo off\r\nif \"%1\"==\"--version\" (\r\n  echo cargo 1.96.0-test\r\n  exit /b 0\r\n)\r\nif \"%1\"==\"metadata\" (\r\n  type \"{}\"\r\n  exit /b 0\r\n)\r\nif \"%1\"==\"check\" (\r\n  type \"{}\"\r\n  exit /b {}\r\n)\r\necho unexpected fake cargo args %* 1>&2\r\nexit /b 2\r\n",
            metadata_path.display(),
            stdout_path.display(),
            exit_code
        ),
    )?;
    Ok(script)
}

#[cfg(unix)]
fn write_fake_cargo(
    dir: &Path,
    metadata_path: &Path,
    stdout_path: &Path,
    exit_code: i32,
) -> Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    let script = dir.join("fake-cargo");
    fs::write(
        &script,
        format!(
            "#!/usr/bin/env sh\nif [ \"$1\" = \"--version\" ]; then\n  echo 'cargo 1.96.0-test'\n  exit 0\nfi\nif [ \"$1\" = \"metadata\" ]; then\n  cat '{}'\n  exit 0\nfi\nif [ \"$1\" = \"check\" ]; then\n  cat '{}'\n  exit {}\nfi\necho unexpected fake cargo args \"$@\" >&2\nexit 2\n",
            metadata_path.display(),
            stdout_path.display(),
            exit_code
        ),
    )?;
    let mut permissions = fs::metadata(&script)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions)?;
    Ok(script)
}
