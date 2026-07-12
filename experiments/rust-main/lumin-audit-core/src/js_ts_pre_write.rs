mod cache;
mod input;
mod projection;
mod protocol;

#[cfg(test)]
mod tests;

use anyhow::Result;
use serde_json::Value;

pub use cache::JsTsPreWriteIncrementalRequest;
pub use protocol::{
    JsTsPreWriteEvidenceRequest, JsTsPreWriteSourceFile,
    JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION,
    JS_TS_PRE_WRITE_EVIDENCE_RESPONSE_SCHEMA_VERSION,
};

pub fn build_js_ts_pre_write_evidence(request: JsTsPreWriteEvidenceRequest) -> Result<Value> {
    projection::project(input::prepare(request)?)
}
