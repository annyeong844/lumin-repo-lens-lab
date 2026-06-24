use anyhow::{Context, Result};
use serde_json::Value;

use crate::artifact::{analyze_file, file, file_health, request, run_sidecar, stdout_json};

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

#[test]
fn function_body_clone_groups_are_repo_wide_review_evidence() -> Result<()> {
    let artifact = stdout_json(run_sidecar(request(vec![
        file(
            "src/a.rs",
            r#"
pub fn exact_a() -> usize {
    let answer = 42;
    answer
}

pub fn structure_a(input: &str) -> usize {
    let parsed = input.len();
    let adjusted = parsed + 1;
    adjusted
}
"#,
        ),
        file(
            "src/b.rs",
            r#"
pub fn exact_b() -> usize {
    let answer = 42;
    answer
}

pub fn structure_b(value: &str) -> usize {
    let amount = value.len();
    let total = amount + 2;
    total
}
"#,
        ),
    ])));

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 4);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 1);
    assert_eq!(artifact["summary"]["functionCloneStructureGroups"], 1);

    let groups = &artifact["functionCloneGroups"];
    assert_eq!(
        groups["policy"]["policyId"],
        "rust-function-clone-group-policy"
    );
    assert_eq!(
        groups["policy"]["caveat"],
        "Function clone groups are deterministic review evidence. They do not prove semantic equivalence, auto-reuse, or auto-fix safety."
    );

    let exact = &groups["exactBodyGroups"][0];
    assert_eq!(exact["kind"], "exact-function-body-group");
    assert_eq!(exact["risk"], "review-only");
    assert_eq!(exact["size"], 2);
    assert!(identity_list_contains(exact, "src/a.rs::exact_a"));
    assert!(identity_list_contains(exact, "src/b.rs::exact_b"));
    assert_eq!(
        exact["reason"],
        "same normalized function body; verify domain ownership before merging"
    );

    let structure = &groups["structureGroups"][0];
    assert_eq!(structure["kind"], "function-body-structure-group");
    assert_eq!(structure["risk"], "review-only");
    assert_eq!(structure["size"], 2);
    assert!(identity_list_contains(structure, "src/a.rs::structure_a"));
    assert!(identity_list_contains(structure, "src/b.rs::structure_b"));
    assert_eq!(structure["exactHashCount"], 2);
    assert!(structure["sharedCallTokens"]
        .as_array()
        .is_some_and(|tokens| tokens.iter().any(|token| token == "len")));
    assert!(structure["reason"]
        .as_str()
        .is_some_and(|reason| reason.contains("not proof of semantic equivalence")));

    Ok(())
}

fn identity_list_contains(group: &Value, identity: &str) -> bool {
    group["identities"]
        .as_array()
        .is_some_and(|identities| identities.iter().any(|entry| entry == identity))
}
