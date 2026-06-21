use anyhow::{Context, Result};
use serde_json::Value;

pub fn assert_targeted_files(artifact: &Value) -> Result<()> {
    let app_file = &artifact["files"]["app/src/lib.rs"];
    let util_file = &artifact["files"]["util/src/lib.rs"];
    assert_eq!(app_file["syntax"]["signalSummary"]["review"], 1);
    let diagnostics = app_file["semantic"]["diagnostics"]
        .as_array()
        .context("app semantic diagnostics")?;
    let diagnostic_index = diagnostics[0]["index"]
        .as_u64()
        .context("app semantic diagnostic ref index")? as usize;
    let diagnostic = artifact["semanticDiagnostics"]
        .as_array()
        .context("semantic diagnostics")?
        .get(diagnostic_index)
        .context("app semantic diagnostic ref target")?;
    assert!(diagnostic["primarySpan"]["fileName"]
        .as_str()
        .map(|path| path.replace('\\', "/"))
        .is_some_and(|path| path.ends_with("app/src/lib.rs")));
    assert!(diagnostic["primarySpan"].get("expansion").is_none());
    assert!(
        diagnostic["primarySpanCount"]
            .as_u64()
            .context("app semantic diagnostic primary span count")?
            > 0
    );
    assert!(diagnostic.get("primarySpans").is_none());
    assert!(util_file.get("semantic").is_none());
    Ok(())
}
