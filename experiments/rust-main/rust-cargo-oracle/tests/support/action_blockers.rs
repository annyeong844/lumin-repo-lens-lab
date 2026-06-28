use anyhow::{Context, Result};
use serde_json::Value;

pub fn assert_macro_expansion_blocker(finding: &Value, macro_name: &str) -> Result<()> {
    let expansion = finding["primarySpans"]
        .as_array()
        .context("primary spans")?
        .iter()
        .find(|span| span["hasExpansion"] == true)
        .and_then(|span| span.get("expansion"))
        .context("macro expansion span")?;

    assert_eq!(expansion["macroDeclName"], macro_name);
    assert!(expansion.get("macro_decl_name").is_none());
    assert!(expansion["span"]["fileName"]
        .as_str()
        .context("expansion call span fileName")?
        .ends_with("lib.rs"));
    assert!(expansion["span"].get("file_name").is_none());
    assert!(expansion.get("def_site_span").is_none());
    assert!(expansion["defSiteSpan"]["fileName"]
        .as_str()
        .context("expansion def-site span fileName")?
        .ends_with("lib.rs"));
    assert!(finding["actionBlockers"]
        .as_array()
        .context("actionBlockers")?
        .iter()
        .any(|blocker| blocker == "macro-expansion"));
    Ok(())
}
