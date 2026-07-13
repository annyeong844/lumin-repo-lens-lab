use anyhow::{bail, Result};
use serde_json::{json, Value};

mod groups;
mod noise;
mod policy;
mod projection;
mod protocol;
mod suffix_array;

pub use protocol::{
    BlockCloneToken, BlockClonesRequest, TokenizedFile, BLOCK_CLONES_REQUEST_SCHEMA_VERSION,
};

use groups::extract_groups;
use noise::apply_noise_policy;
use policy::normalize_thresholds;
use projection::{build_artifact, ArtifactProjectionInput};
use suffix_array::compress_token_values;

pub fn build_block_clones_artifact(request: BlockClonesRequest) -> Result<Value> {
    if request.schema_version != BLOCK_CLONES_REQUEST_SCHEMA_VERSION {
        bail!(
            "block-clones-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let BlockClonesRequest {
        schema_version: _,
        generated,
        root,
        include_tests,
        exclude,
        files,
        thresholds,
        incremental,
    } = request;
    let thresholds = normalize_thresholds(thresholds.as_ref());
    let mut tokenized_files = Vec::<TokenizedFile>::new();
    let mut skipped = Vec::<Value>::new();
    let mut diagnostics = Vec::<Value>::new();
    let mut unavailable_file_count = 0usize;

    for file in files {
        if let Some(skipped_file) = file.skipped {
            skipped.push(skipped_file);
            continue;
        }
        if file.token_limit_exceeded {
            skipped.push(json!({
                "file": file.rel_file,
                "reason": "max-tokens-per-file",
                "evidence": "threshold:maxTokensPerFile",
            }));
            continue;
        }
        if !file.diagnostics.is_empty() {
            diagnostics.extend(file.diagnostics);
            unavailable_file_count += 1;
            continue;
        }
        tokenized_files.push(file);
    }

    let compressed = compress_token_values(&tokenized_files);
    let groups = extract_groups(&compressed.values, &compressed.meta, &thresholds);
    let noise_policy = apply_noise_policy(groups, &thresholds);

    Ok(build_artifact(ArtifactProjectionInput {
        generated,
        root,
        include_tests,
        exclude,
        incremental,
        tokenized_files,
        thresholds,
        noise_policy,
        skipped,
        diagnostics,
        unavailable_file_count,
    }))
}

#[cfg(test)]
mod tests;
