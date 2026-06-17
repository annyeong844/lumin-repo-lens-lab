use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn run_sidecar(request: Value) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_lumin-rust-source-health");
    let mut child = Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sidecar");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(request.to_string().as_bytes())
        .expect("write request");
    child.wait_with_output().expect("sidecar output")
}

fn request(files: Vec<Value>) -> Value {
    json!({
        "schemaVersion": 1,
        "root": "C:/repo",
        "files": files,
        "pathPolicy": {
            "include": ["**/*.rs"],
            "exclude": ["**/target/**", "**/vendor/**"]
        },
        "parser": {
            "editionPolicy": "fixed",
            "edition": "2021",
            "editionSource": "m6-policy-default"
        },
        "runtime": {
            "threadCount": 2,
            "workerStackBytes": 16777216
        }
    })
}

fn file(path: &str, text: &str, hash_char: char) -> Value {
    let _ = hash_char;
    json!({
        "path": path,
        "sha256": sha256_text(text),
        "text": text
    })
}

fn file_with_sha(path: &str, text: &str, sha256: &str) -> Value {
    json!({
        "path": path,
        "sha256": sha256,
        "text": text
    })
}

fn sha256_text(text: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(text.as_bytes()))
}

fn stdout_json(output: std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout json")
}

fn assert_exit_code(output: &std::process::Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_cli(args: &[String]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_lumin-rust-source-health");
    Command::new(bin)
        .args(args)
        .output()
        .expect("run rust source health cli")
}

#[test]
fn reports_syntax_facts_and_review_signals() {
    let source = r#"
fn main() {
    let value = Some(String::from("x"));
    let cloned = value.clone();
    let _ = cloned.expect("value");
    unsafe {
        do_thing();
    }
    panic!("boom");
}

unsafe fn do_thing() {}
"#;
    let value = stdout_json(run_sidecar(request(vec![file("src/lib.rs", source, 'a')])));
    let file = &value["files"]["src/lib.rs"];
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["meta"]["producer"], "rust-source-health");
    assert_eq!(value["meta"]["mode"], "syntax-only");
    assert_eq!(value["meta"]["parser"]["version"], "0.0.337");
    assert_eq!(file["sha256"], sha256_text(source));
    assert_eq!(file["parse"]["ok"], true);
    assert_eq!(file["signals"][0]["severity"], "review");
    assert_eq!(file["signals"][0]["claim"], "syntax-only");
    assert_eq!(file["facts"]["unsafeBlocks"], 1);
    assert_eq!(file["facts"]["unsafeFunctions"], 1);
    assert_eq!(value["summary"]["signalsByKind"]["clone-call"], 1);
    assert_eq!(value["summary"]["signalsByKind"]["expect-call"], 1);
    assert_eq!(value["summary"]["signalsByKind"]["panic-macro"], 1);
    assert_eq!(value["summary"]["signalsByKind"]["unsafe-block"], 1);
}

#[test]
fn reports_macro_signals_from_ast_paths() {
    let source = r#"
fn main() {
    std::panic!("boom");
    todo!("later");
    core::unimplemented!("later");
}
"#;
    let value = stdout_json(run_sidecar(request(vec![file("src/lib.rs", source, 'a')])));

    assert_eq!(value["summary"]["signalsByKind"]["panic-macro"], 1);
    assert_eq!(value["summary"]["signalsByKind"]["todo-macro"], 1);
    assert_eq!(value["summary"]["signalsByKind"]["unimplemented-macro"], 1);
}

#[test]
fn does_not_emit_method_signals_for_plain_identifiers() {
    let source = r#"
fn unwrap() {}
fn main() {
    let clone = 1;
    struct S { expect: bool }
}
"#;
    let value = stdout_json(run_sidecar(request(vec![file("src/lib.rs", source, 'b')])));
    assert!(value["summary"]["signalsByKind"]["unwrap-call"].is_null());
    assert!(value["summary"]["signalsByKind"]["clone-call"].is_null());
    assert!(value["summary"]["signalsByKind"]["expect-call"].is_null());
}

#[test]
fn records_parse_errors_as_file_data() {
    let value = stdout_json(run_sidecar(request(vec![file(
        "src/bad.rs",
        "fn main( {",
        'c',
    )])));
    assert_eq!(value["files"]["src/bad.rs"]["parse"]["ok"], false);
    assert_eq!(
        value["files"]["src/bad.rs"]["parse"]["errors"][0]["claim"],
        "syntax-only"
    );
    assert!(value["summary"]["parseErrors"].as_u64().unwrap() > 0);
    assert_eq!(value["summary"]["parseErrorFiles"], 1);
}

#[test]
fn classifies_root_level_test_and_generated_paths() {
    let value = stdout_json(run_sidecar(request(vec![
        file("tests/integration.rs", "fn integration() {}", 'd'),
        file("generated/bindings.rs", "fn generated() {}", 'e'),
        file("src/notgenerated.rs", "fn source() {}", 'f'),
    ])));
    assert_eq!(
        value["files"]["tests/integration.rs"]["path"]["classifications"],
        json!(["test"])
    );
    assert_eq!(
        value["files"]["generated/bindings.rs"]["path"]["classifications"],
        json!(["generated"])
    );
    assert_eq!(
        value["files"]["src/notgenerated.rs"]["path"]["classifications"],
        json!(["source"])
    );
}

#[test]
fn emits_files_in_deterministic_path_order() {
    let value = stdout_json(run_sidecar(request(vec![
        file("src/z.rs", "fn z() {}", 'd'),
        file("src/a.rs", "fn a() {}", 'e'),
    ])));
    let text = serde_json::to_string(&value["files"]).unwrap();
    let a_pos = text.find("src/a.rs").unwrap();
    let z_pos = text.find("src/z.rs").unwrap();
    assert!(a_pos < z_pos);
}

#[test]
fn rejects_unsupported_schema_without_json_artifact() {
    let mut value = request(vec![file("src/lib.rs", "fn main() {}", 'f')]);
    value["schemaVersion"] = json!(999);
    let output = run_sidecar(value);
    assert_exit_code(&output, 2);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported schemaVersion"));
}

#[test]
fn rejects_unsupported_parser_policy_without_json_artifact() {
    let mut value = request(vec![file("src/lib.rs", "fn main() {}", 'f')]);
    value["parser"]["edition"] = json!("2024");
    let output = run_sidecar(value);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported parser edition policy"));
}

#[test]
fn rejects_relative_root_without_json_artifact() {
    let mut value = request(vec![file("src/lib.rs", "fn main() {}", 'f')]);
    value["root"] = json!("relative/repo");
    let output = run_sidecar(value);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("root must be absolute"));
}

#[test]
fn rejects_unsafe_file_paths_without_json_artifact() {
    for bad_path in [
        "src\\lib.rs",
        "C:/repo/src/lib.rs",
        "src//lib.rs",
        "./src/lib.rs",
        "src/../lib.rs",
    ] {
        let output = run_sidecar(request(vec![file(bad_path, "fn main() {}", 'f')]));
        assert!(!output.status.success(), "path should fail: {}", bad_path);
        assert!(
            output.stdout.is_empty(),
            "path should not emit JSON: {}",
            bad_path
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("file path"),
            "stderr should mention file path for {}: {}",
            bad_path,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn rejects_mismatched_sha256_without_json_artifact() {
    let output = run_sidecar(request(vec![file_with_sha(
        "src/lib.rs",
        "fn main() {}",
        &format!("sha256:{}", "0".repeat(64)),
    )]));
    assert_exit_code(&output, 2);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("sha256/text mismatch"));
}

#[test]
fn rejects_duplicate_paths_without_json_artifact() {
    let output = run_sidecar(request(vec![
        file("src/lib.rs", "fn one() {}", 'a'),
        file("src/lib.rs", "fn two() {}", 'b'),
    ]));
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate file path"));
}

#[test]
fn rejects_invalid_runtime_stack_without_json_artifact() {
    let mut value = request(vec![file("src/lib.rs", "fn main() {}", 'f')]);
    value["runtime"]["workerStackBytes"] = json!(1);
    let output = run_sidecar(value);
    assert_exit_code(&output, 2);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("runtime.workerStackBytes"));
}

#[test]
fn cli_usage_errors_exit_2_without_json_artifact() {
    for args in [
        vec!["--unknown".to_string()],
        vec![
            "--output".to_string(),
            "out.json".to_string(),
            "--source-commit".to_string(),
            "test-source-commit".to_string(),
        ],
        vec![
            "--root".to_string(),
            "missing-root".to_string(),
            "--output".to_string(),
            "out.json".to_string(),
            "--source-commit".to_string(),
            "test-source-commit".to_string(),
        ],
        vec![
            "--root".to_string(),
            ".".to_string(),
            "--output".to_string(),
            "out.json".to_string(),
            "--source-commit".to_string(),
            "test-source-commit".to_string(),
            "--threads".to_string(),
            "0".to_string(),
        ],
    ] {
        let output = run_cli(&args);
        assert_exit_code(&output, 2);
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn cli_collects_sources_and_writes_final_artifact_without_node_wrapper() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("src")).expect("src dir");
    fs::create_dir_all(root.join("target")).expect("target dir");
    fs::create_dir_all(root.join("vendor")).expect("vendor dir");
    fs::write(
        root.join("src").join("lib.rs"),
        "pub fn main() { let value = Some(1); let _ = value.unwrap(); }\n",
    )
    .expect("lib rs");
    fs::write(root.join("src").join("bad.rs"), [0xff, 0xfe, 0xfd]).expect("bad rs");
    fs::write(
        root.join("target").join("generated.rs"),
        "pub fn hidden() { panic!(\"nope\"); }\n",
    )
    .expect("target generated");
    fs::write(
        root.join("vendor").join("vendored.rs"),
        "pub fn hidden() { panic!(\"nope\"); }\n",
    )
    .expect("vendor generated");

    let output_path = temp.path().join("rust-health.json");
    let output = run_cli(&[
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--output".to_string(),
        output_path.to_string_lossy().to_string(),
        "--source-commit".to_string(),
        "test-source-commit".to_string(),
        "--threads".to_string(),
        "2".to_string(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact: Value = serde_json::from_slice(
        &fs::read(&output_path).expect("rust health output should be written"),
    )
    .expect("rust health json");
    assert_eq!(artifact["meta"]["generated"].is_string(), true);
    assert_eq!(
        artifact["meta"]["sidecar"]["sourceCommit"],
        "test-source-commit"
    );
    assert_eq!(
        artifact["meta"]["input"]["pathPolicy"]["exclude"],
        json!(["**/target/**", "**/vendor/**"])
    );
    assert_eq!(artifact["summary"]["files"], 1);
    assert_eq!(artifact["summary"]["skippedFiles"], 1);
    assert_eq!(artifact["summary"]["signalsByKind"]["unwrap-call"], 1);
    assert!(artifact["files"]["src/lib.rs"].is_object());
    assert!(artifact["files"]["target/generated.rs"].is_null());
    assert!(artifact["files"]["vendor/vendored.rs"].is_null());
    assert_eq!(artifact["skippedFiles"][0]["path"], "src/bad.rs");
    assert_eq!(artifact["skippedFiles"][0]["reason"], "invalid-utf8");
}
