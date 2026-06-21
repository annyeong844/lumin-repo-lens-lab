use anyhow::Result;
use serde_json::Value;

pub(super) fn assert_ast_examples(merged_file: &Value) -> Result<()> {
    let examples = merged_file["syntax"]["astExamples"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("ast examples"))?;

    assert!(examples.get("definitions").is_none());
    assert!(examples.get("useTrees").is_none());
    assert!(examples.get("pathRefs").is_none());
    assert!(examples.get("methodCalls").is_none());
    assert!(examples.get("methodCallCounts").is_none());
    assert!(examples.get("macroCalls").is_none());
    assert!(examples.get("cfgGates").is_none());
    let review_opaque_surfaces = examples
        .get("reviewOpaqueSurfaces")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("review opaque surface examples"))?;
    assert!(!review_opaque_surfaces.is_empty());
    Ok(())
}
