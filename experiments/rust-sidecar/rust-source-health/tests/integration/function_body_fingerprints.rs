use anyhow::{Context, Result};
use serde_json::Value;

use crate::artifact::{analyze_file, file_health};

#[test]
fn function_body_fingerprints_preserve_exact_and_structure_hashes() -> Result<()> {
    let source = r#"
pub fn read_a(input: &str) -> usize {
    let parsed = input.len();
    parsed + 1
}

pub fn read_b(input: &str) -> usize {
    let parsed = input.len();
    parsed + 1
}

pub fn read_c(input: &str) -> usize {
    let parsed = input.len();
    parsed + 2
}

pub struct Worker;

impl Worker {
    pub async fn refresh(&self) {
        self.load().await;
    }

    pub unsafe fn reset(&self) {
        cleanup();
    }
}
"#;

    let artifact = analyze_file("src/lib.rs", source);
    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 5);

    let facts = file_health(&artifact, "src/lib.rs")["ast"]["functionBodyFingerprints"]
        .as_array()
        .context("function body fingerprints")?;
    let read_a = fact_named(facts, "read_a")?;
    let read_b = fact_named(facts, "read_b")?;
    let read_c = fact_named(facts, "read_c")?;

    assert_eq!(read_a["kind"], "function-body-fingerprint");
    assert_eq!(read_a["callableKind"], "function");
    assert_eq!(read_a["visibility"], "public");
    assert_eq!(read_a["paramCount"], 1);
    assert_eq!(read_a["statementCount"], 1);
    assert_eq!(read_a["bodyLoc"], 4);
    assert_eq!(read_a["async"], false);
    assert_eq!(read_a["unsafe"], false);
    assert!(read_a["callTokens"]
        .as_array()
        .is_some_and(|tokens| tokens.iter().any(|token| token == "len")));

    assert_eq!(read_a["exactBodyHash"], read_b["exactBodyHash"]);
    assert_ne!(read_a["exactBodyHash"], read_c["exactBodyHash"]);
    assert_eq!(
        read_a["normalizedStructureHash"],
        read_c["normalizedStructureHash"]
    );
    assert_ne!(read_a["normalizedExactHash"], read_c["normalizedExactHash"]);

    let refresh = fact_named(facts, "refresh")?;
    assert_eq!(refresh["callableKind"], "impl-method");
    assert_eq!(refresh["owner"]["target"], "Worker");
    assert_eq!(refresh["async"], true);

    let reset = fact_named(facts, "reset")?;
    assert_eq!(reset["callableKind"], "impl-method");
    assert_eq!(reset["owner"]["target"], "Worker");
    assert_eq!(reset["unsafe"], true);

    Ok(())
}

fn fact_named<'a>(facts: &'a [Value], name: &str) -> Result<&'a Value> {
    facts
        .iter()
        .find(|fact| fact["name"] == name)
        .with_context(|| format!("missing function body fingerprint for {name}"))
}
