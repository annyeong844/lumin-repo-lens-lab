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
