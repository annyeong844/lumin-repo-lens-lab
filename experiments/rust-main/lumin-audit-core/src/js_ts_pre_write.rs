mod cache;
mod input;
mod projection;
mod protocol;
mod single_flight;

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

pub struct JsTsPreWriteEvidenceRun {
    evidence: Value,
    _lease: single_flight::ScanLease,
}

impl JsTsPreWriteEvidenceRun {
    pub fn into_evidence(self) -> Value {
        self.evidence
    }
}

pub fn start_js_ts_pre_write_evidence(
    request: JsTsPreWriteEvidenceRequest,
) -> Result<JsTsPreWriteEvidenceRun> {
    input::validate_request(&request)?;
    let lease = single_flight::ScanLease::acquire(&request.root)?;
    let prepared = input::prepare(request)?;
    let discovery_ms = prepared.discovery_ms;
    let projection_started = std::time::Instant::now();
    let mut evidence = projection::project(prepared)?;
    let projection_ms = single_flight::elapsed_ms(projection_started);
    single_flight::attach_runtime_observations(&mut evidence, &lease, discovery_ms, projection_ms)?;
    Ok(JsTsPreWriteEvidenceRun {
        evidence,
        _lease: lease,
    })
}

pub fn build_js_ts_pre_write_evidence(request: JsTsPreWriteEvidenceRequest) -> Result<Value> {
    Ok(start_js_ts_pre_write_evidence(request)?.into_evidence())
}
