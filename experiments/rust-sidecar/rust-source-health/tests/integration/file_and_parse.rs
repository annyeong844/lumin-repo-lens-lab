use anyhow::{Context, Result};

use crate::artifact::{analyze_file, file, request, run_sidecar, stdout_json};
use crate::ast_facts::{
    assert_ast_summary_counts, assert_core_ast_fact_projection, AstSummaryCounts,
};

#[test]
fn emits_files_in_deterministic_path_order() -> Result<()> {
    let value = stdout_json(run_sidecar(request(vec![
        file("src/z.rs", "fn z() {}"),
        file("src/a.rs", "fn a() {}"),
    ])));

    let text = serde_json::to_string(&value["files"])?;
    let a_pos = text.find("src/a.rs").context("src/a.rs file key")?;
    let z_pos = text.find("src/z.rs").context("src/z.rs file key")?;
    assert!(a_pos < z_pos);
    Ok(())
}

#[test]
fn emits_ast_facts_without_claiming_semantics() {
    let source = r#"
pub use crate::{model::Thing as Alias, prelude::*};

pub fn build() {
    let value = crate::factory::make();
    let _ = value.normalize();
    custom_macro!();
}

pub(crate) struct Maybe {
    pub id: usize,
    label: usize,
}

impl Maybe {
    pub fn normalize(&self) -> usize {
        1
    }

    pub(crate) fn make() -> Self {
        Maybe {
            id: 1,
            label: 0,
        }
    }
}
"#;
    let value = analyze_file("src/lib.rs", source);

    assert_ast_summary_counts(
        &value,
        AstSummaryCounts {
            definitions: 4,
            shape_hashes: 1,
            function_signatures: 3,
            function_body_fingerprints: 3,
            inline_patterns: 0,
            impl_blocks: 1,
            impl_methods: 2,
            use_trees: 3,
            path_refs: 1,
            method_call_sites: 1,
            method_calls: 0,
            macro_calls: 1,
        },
    );
    assert_core_ast_fact_projection(&value, "src/lib.rs");
}

#[test]
fn inline_patterns_record_repeated_simple_no_arg_statement_blocks() -> Result<()> {
    let source = r#"
pub struct Worker;

impl Worker {
    pub fn first(&self) {
        self.cleanup();
        self.close();
    }

    pub fn second(&self) {
        self.cleanup();
        self.close();
    }

    pub fn third(&self) {
        self.cleanup();
        self.close();
    }

    pub fn gated(&self) {
        #[cfg(feature = "fast")]
        self.cleanup();
    }
}
"#;
    let value = analyze_file("src/lib.rs", source);
    let patterns = value["files"]["src/lib.rs"]["ast"]["inlinePatterns"]
        .as_array()
        .context("inline patterns")?;

    assert_eq!(value["summary"]["inlinePatterns"], 3);
    assert_eq!(patterns.len(), 3);
    assert!(patterns
        .iter()
        .all(|pattern| pattern["kind"] == "statement-sequence"));
    assert!(patterns
        .iter()
        .all(|pattern| pattern["statementCount"] == 2));
    assert!(patterns.iter().all(|pattern| {
        pattern["normalizedPattern"] == "block { self.cleanup(); self.close(); }"
    }));
    assert!(patterns
        .iter()
        .all(|pattern| pattern["normalizedVersion"] == "rust-inline-statement-normalizer-v1"));
    assert_eq!(patterns[0]["patternHash"], patterns[1]["patternHash"]);
    assert_eq!(patterns[1]["patternHash"], patterns[2]["patternHash"]);
    assert!(patterns
        .iter()
        .all(|pattern| pattern["enclosingFunction"] != "gated"));
    Ok(())
}

#[test]
fn records_type_position_path_refs_without_generic_arguments() -> Result<()> {
    let source = r#"
pub fn decode() -> serde1::Result<()> {
    Ok(())
}
"#;
    let value = analyze_file("src/lib.rs", source);
    let path_refs = value["files"]["src/lib.rs"]["ast"]["pathRefs"]
        .as_array()
        .context("path refs")?;

    assert!(path_refs
        .iter()
        .any(|path_ref| path_ref["path"] == "serde1::Result"));
    assert!(path_refs
        .iter()
        .all(|path_ref| path_ref["path"] != "serde1::Result<()>"));
    Ok(())
}

#[test]
fn shape_hashes_are_exact_for_record_structs_without_claiming_unsupported_shapes() -> Result<()> {
    let source = r#"
pub struct First {
    z: u8,
    pub a: usize,
}

pub struct Second {
    pub a: usize,
    z: u8,
}

pub struct Tuple(pub usize);
pub struct Unit;
pub struct Generic<T> {
    value: T,
}
pub type Alias = First;
"#;
    let value = analyze_file("src/lib.rs", source);
    let shapes = value["files"]["src/lib.rs"]["ast"]["shapeHashes"]
        .as_array()
        .context("shape hashes")?;

    assert_eq!(value["summary"]["shapeHashes"], 2);
    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes[0]["name"], "First");
    assert_eq!(shapes[1]["name"], "Second");
    assert_eq!(shapes[0]["hash"], shapes[1]["hash"]);
    assert_eq!(shapes[0]["fields"][0]["name"], "a");
    assert_eq!(shapes[0]["fields"][0]["visibility"], "public");
    assert_eq!(shapes[0]["fields"][1]["name"], "z");
    assert_eq!(shapes[0]["fields"][1]["visibility"], "private");
    assert!(shapes.iter().all(|shape| {
        !matches!(
            shape["name"].as_str(),
            Some("Tuple" | "Unit" | "Generic" | "Alias")
        )
    }));
    Ok(())
}

#[test]
fn shape_hashes_normalize_type_punctuation_and_refuse_cfg_or_restricted_fields() -> Result<()> {
    let source = r#"
pub struct Compact {
    pub borrowed: &'static str,
    pub items: Vec<u8>,
}

pub struct Spaced {
    pub items: Vec < u8 >,
    pub borrowed: & 'static str,
}

pub struct Gated {
    id: u8,
    #[cfg(feature = "extra")]
    extra: u8,
}

pub struct Restricted {
    pub(super) id: u8,
}
"#;
    let value = analyze_file("src/lib.rs", source);
    let shapes = value["files"]["src/lib.rs"]["ast"]["shapeHashes"]
        .as_array()
        .context("shape hashes")?;
    let compact = shapes
        .iter()
        .find(|shape| shape["name"] == "Compact")
        .context("Compact shape")?;
    let spaced = shapes
        .iter()
        .find(|shape| shape["name"] == "Spaced")
        .context("Spaced shape")?;

    assert_eq!(compact["hash"], spaced["hash"]);
    assert_eq!(compact["fields"][0]["type"], "&'static str");
    assert_eq!(spaced["fields"][0]["type"], "&'static str");
    assert!(shapes
        .iter()
        .all(|shape| !matches!(shape["name"].as_str(), Some("Gated" | "Restricted"))));
    Ok(())
}

#[test]
fn function_signature_hashes_refuse_unrepresented_call_qualifiers_and_where_bounds() -> Result<()> {
    let source = r#"
pub fn plain(input: u8) -> u8 {
    input
}

pub async fn async_plain(input: u8) -> u8 {
    input
}

pub unsafe fn unsafe_plain(input: u8) -> u8 {
    input
}

pub fn bounded<T>(input: T) -> T
where
    T: Clone,
{
    input
}
"#;
    let value = analyze_file("src/lib.rs", source);
    let signatures = value["files"]["src/lib.rs"]["ast"]["functionSignatures"]
        .as_array()
        .context("function signatures")?;

    assert!(signatures
        .iter()
        .any(|signature| signature["name"] == "plain"));
    assert!(signatures.iter().all(|signature| {
        !matches!(
            signature["name"].as_str(),
            Some("async_plain" | "unsafe_plain" | "bounded")
        )
    }));
    Ok(())
}

#[test]
fn records_parse_errors_as_file_data() -> Result<()> {
    let value = stdout_json(run_sidecar(request(vec![file("src/bad.rs", "fn main( {")])));

    assert_eq!(value["files"]["src/bad.rs"]["parse"]["ok"], false);
    assert_eq!(
        value["files"]["src/bad.rs"]["parse"]["errors"][0]["claim"],
        "syntax-only"
    );
    assert!(
        value["summary"]["parseErrors"]
            .as_u64()
            .context("summary.parseErrors")?
            > 0
    );
    assert_eq!(value["summary"]["parseErrorFiles"], 1);
    Ok(())
}
