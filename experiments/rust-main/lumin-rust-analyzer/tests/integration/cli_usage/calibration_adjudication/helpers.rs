use anyhow::{Context, Result};
use serde_json::Value;

pub(crate) fn calibration_readiness(artifact: &Value) -> &Value {
    &artifact["oracleBridge"]["policy"]["calibration"]["readiness"]
}

pub(crate) fn assert_readiness_reason(readiness: &Value, code: &str, severity: &str) {
    assert!(
        readiness["reasons"]
            .as_array()
            .is_some_and(|reasons| reasons
                .iter()
                .any(|reason| reason["code"] == code && reason["severity"] == severity)),
        "missing {code}/{severity} readiness reason: {}",
        readiness["reasons"]
    );
}

pub(crate) fn assert_no_readiness_reason(readiness: &Value, code: &str) {
    assert!(
        readiness["reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().all(|reason| reason["code"] != code)),
        "unexpected {code} readiness reason: {}",
        readiness["reasons"]
    );
}

pub(crate) fn readiness_reason<'a>(readiness: &'a Value, code: &str) -> Result<&'a Value> {
    readiness["reasons"]
        .as_array()
        .context("calibration readiness reasons")?
        .iter()
        .find(|reason| reason["code"] == code)
        .with_context(|| format!("missing {code} readiness reason"))
}
