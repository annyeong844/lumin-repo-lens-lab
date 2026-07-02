use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

pub const LIFECYCLE_REQUEST_GUARD_SCHEMA_VERSION: &str = "lumin-lifecycle-request-guard.v1";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleRequestGuardInput {
    pub schema_version: String,
    #[serde(default)]
    pub pre_write_requested: bool,
    #[serde(default)]
    pub post_write_requested: bool,
    #[serde(default)]
    pub pre_write_intent_present: bool,
    #[serde(default = "default_pre_write_engine")]
    pub requested_pre_write_engine: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleRequestGuardResult {
    pub status: LifecycleRequestGuardStatus,
    pub exit_code: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_write: Option<LifecycleRequestBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_write: Option<LifecycleRequestBlock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleRequestGuardStatus {
    Clear,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleRequestBlock {
    pub requested: bool,
    pub ran: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producer: Option<&'static str>,
    pub reason: String,
}

pub fn evaluate_lifecycle_request_guard(
    input: LifecycleRequestGuardInput,
) -> Result<LifecycleRequestGuardResult> {
    if input.schema_version != LIFECYCLE_REQUEST_GUARD_SCHEMA_VERSION {
        bail!(
            "lifecycle-request-guard: unsupported schemaVersion '{}'",
            input.schema_version
        );
    }
    validate_pre_write_engine(&input.requested_pre_write_engine)?;

    if input.pre_write_requested && input.post_write_requested {
        let reason = "--pre-write and --post-write are mutually exclusive";
        return Ok(blocked(
            Some(request_block(None, None, None, reason)),
            Some(request_block(None, None, None, reason)),
            format!("[audit-repo] {reason}\n"),
        ));
    }

    if input.pre_write_requested && !input.pre_write_intent_present {
        let reason = "--intent missing";
        let (language, producer) = explicit_pre_write_owner(&input.requested_pre_write_engine);
        return Ok(blocked(
            Some(request_block(
                Some(input.requested_pre_write_engine),
                language,
                producer,
                reason,
            )),
            None,
            "[audit-repo] --pre-write requested but skipped: --intent <file|-> missing\n"
                .to_string(),
        ));
    }

    Ok(LifecycleRequestGuardResult {
        status: LifecycleRequestGuardStatus::Clear,
        exit_code: 0,
        stderr: None,
        pre_write: None,
        post_write: None,
    })
}

fn blocked(
    pre_write: Option<LifecycleRequestBlock>,
    post_write: Option<LifecycleRequestBlock>,
    stderr: String,
) -> LifecycleRequestGuardResult {
    LifecycleRequestGuardResult {
        status: LifecycleRequestGuardStatus::Blocked,
        exit_code: 2,
        stderr: Some(stderr),
        pre_write,
        post_write,
    }
}

fn request_block(
    engine: Option<String>,
    language: Option<&'static str>,
    producer: Option<&'static str>,
    reason: &str,
) -> LifecycleRequestBlock {
    LifecycleRequestBlock {
        requested: true,
        ran: false,
        engine,
        language,
        producer,
        reason: reason.to_string(),
    }
}

fn explicit_pre_write_owner(engine: &str) -> (Option<&'static str>, Option<&'static str>) {
    match engine {
        "rust" => (Some("rust"), Some("lumin-rust-analyzer")),
        "js" => (Some("js-ts"), Some("pre-write.mjs")),
        _ => (None, None),
    }
}

fn validate_pre_write_engine(engine: &str) -> Result<()> {
    match engine {
        "auto" | "js" | "rust" => Ok(()),
        other => bail!("pre-write engine must be auto, js, or rust; got '{other}'"),
    }
}

fn default_pre_write_engine() -> String {
    "auto".to_string()
}
