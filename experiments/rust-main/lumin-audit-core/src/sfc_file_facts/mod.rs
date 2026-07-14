mod blocks;
mod protocol;
mod script;
mod style;
mod template;

use anyhow::{bail, Result};
use std::collections::BTreeSet;

use blocks::{script_blocks, script_source_blocks, style_blocks, template_blocks, SfcLanguage};
use protocol::SfcFileFacts;
pub use protocol::{SfcFileFactsRequest, SfcFileFactsResponse};
use script::{extract_script_facts, ScriptFacts};
use style::extract_style_asset_references;
use template::extract_template_component_refs;

pub const SFC_FILE_FACTS_REQUEST_SCHEMA_VERSION: &str = "lumin-sfc-file-facts-request.v1";
pub const SFC_FILE_FACTS_RESPONSE_SCHEMA_VERSION: &str = "lumin-sfc-file-facts-response.v1";

pub fn build_sfc_file_facts_response(request: SfcFileFactsRequest) -> Result<SfcFileFactsResponse> {
    if request.schema_version != SFC_FILE_FACTS_REQUEST_SCHEMA_VERSION {
        bail!(
            "sfc-file-facts-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    let mut seen = BTreeSet::new();
    let mut files = Vec::with_capacity(request.files.len());
    for input in request.files {
        if input.file_path.is_empty() {
            bail!("sfc-file-facts-artifact: files[].filePath must not be empty");
        }
        if !seen.insert(input.file_path.clone()) {
            bail!(
                "sfc-file-facts-artifact: duplicate files[].filePath '{}'",
                input.file_path
            );
        }
        files.push(extract_file_facts(&input.file_path, &input.source)?);
    }
    Ok(SfcFileFactsResponse {
        schema_version: SFC_FILE_FACTS_RESPONSE_SCHEMA_VERSION,
        files,
    })
}

fn extract_file_facts(file_path: &str, source: &str) -> Result<SfcFileFacts> {
    let language = SfcLanguage::from_path(file_path)?;
    let scripts = script_blocks(source, language);
    let ScriptFacts { imports, bindings } =
        extract_script_facts(source, file_path, language, &scripts)?;
    let styles = style_blocks(source, language);
    let templates = template_blocks(source, language);

    Ok(SfcFileFacts {
        file_path: file_path.to_string(),
        script_import_consumers: imports,
        script_sources: script_source_blocks(source, file_path, language),
        style_asset_references: extract_style_asset_references(
            source, file_path, language, &styles,
        ),
        template_component_refs: extract_template_component_refs(
            source, file_path, language, &templates, &bindings,
        ),
    })
}

#[cfg(test)]
mod tests;
