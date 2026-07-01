use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const PRE_WRITE_ROUTING_REQUEST_SCHEMA_VERSION: &str = "lumin-pre-write-routing-request.v1";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreWriteRoutingRequest {
    pub schema_version: String,
    pub requested_engine: String,
    pub intent_flag: String,
    pub intent_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreWriteRoutingResult {
    pub engine: String,
    pub child_intent_flag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_intent_input: Option<String>,
    pub engine_selection: PreWriteEngineSelection,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreWriteEngineSelection {
    pub requested: String,
    pub selected: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent_language: Option<String>,
}

pub fn resolve_pre_write_route(request: PreWriteRoutingRequest) -> Result<PreWriteRoutingResult> {
    if request.schema_version != PRE_WRITE_ROUTING_REQUEST_SCHEMA_VERSION {
        bail!(
            "unsupported pre-write routing schemaVersion '{}'",
            request.schema_version
        );
    }
    if request.intent_flag.is_empty() {
        bail!("intentFlag must be non-empty");
    }

    let parsed = parse_intent(&request.intent_text)?;
    let intent_language = normalize_intent_language(parsed.get("language"))?;
    match request.requested_engine.as_str() {
        "js" => route_explicit_js(request, intent_language),
        "rust" => route_explicit_rust(request, intent_language),
        "auto" => route_auto(request, intent_language),
        other => bail!("pre-write engine must be auto, js, or rust; got '{other}'"),
    }
}

fn parse_intent(intent_text: &str) -> Result<Map<String, Value>> {
    let parsed = serde_json::from_str::<Value>(intent_text)
        .map_err(|error| anyhow!("intent JSON parse failed before engine selection: {error}"))?;
    match parsed {
        Value::Object(object) => Ok(object),
        _ => bail!("intent must be a plain object before engine selection"),
    }
}

fn normalize_intent_language(value: Option<&Value>) -> Result<Option<String>> {
    match value {
        None => Ok(None),
        Some(Value::String(value)) if value == "rust" || value == "js-ts" => {
            Ok(Some(value.clone()))
        }
        Some(_) => bail!("intent.language must be \"rust\" or \"js-ts\" when present"),
    }
}

fn route_explicit_js(
    request: PreWriteRoutingRequest,
    intent_language: Option<String>,
) -> Result<PreWriteRoutingResult> {
    if intent_language.as_deref() == Some("rust") {
        bail!(
            "intent.language \"rust\" is owned by lumin-rust-analyzer; use --pre-write-engine auto or --pre-write-engine rust"
        );
    }
    let child_intent_input = if request.intent_flag == "-" {
        Some(request.intent_text)
    } else {
        None
    };
    Ok(route_result(
        "js",
        request.intent_flag,
        child_intent_input,
        request.requested_engine,
        "explicit-cli",
        intent_language,
    ))
}

fn route_explicit_rust(
    request: PreWriteRoutingRequest,
    intent_language: Option<String>,
) -> Result<PreWriteRoutingResult> {
    if intent_language.as_deref() == Some("js-ts") {
        bail!(
            "intent.language \"js-ts\" is owned by pre-write.mjs; use --pre-write-engine js or --pre-write-engine auto"
        );
    }
    Ok(route_result(
        "rust",
        "-".to_string(),
        Some(strip_route_only_fields(&request.intent_text)),
        request.requested_engine,
        "explicit-cli",
        intent_language,
    ))
}

fn route_auto(
    request: PreWriteRoutingRequest,
    intent_language: Option<String>,
) -> Result<PreWriteRoutingResult> {
    let selected = if intent_language.as_deref() == Some("rust") {
        "rust"
    } else {
        "js"
    };
    let child_intent_input = if selected == "rust" {
        Some(strip_route_only_fields(&request.intent_text))
    } else {
        Some(request.intent_text)
    };
    let reason = if intent_language.is_some() {
        "intent-language"
    } else {
        "intent-language-absent-default-js"
    };
    Ok(route_result(
        selected,
        "-".to_string(),
        child_intent_input,
        request.requested_engine,
        reason,
        intent_language,
    ))
}

fn route_result(
    selected: &str,
    child_intent_flag: String,
    child_intent_input: Option<String>,
    requested: String,
    reason: &str,
    intent_language: Option<String>,
) -> PreWriteRoutingResult {
    PreWriteRoutingResult {
        engine: selected.to_string(),
        child_intent_flag,
        child_intent_input,
        engine_selection: PreWriteEngineSelection {
            requested,
            selected: selected.to_string(),
            reason: reason.to_string(),
            intent_language,
        },
    }
}

fn strip_route_only_fields(intent_text: &str) -> String {
    let Ok(Value::Object(mut object)) = serde_json::from_str::<Value>(intent_text) else {
        return intent_text.to_string();
    };
    if !object.contains_key("language") {
        return intent_text.to_string();
    }
    object.remove("language");
    let value = Value::Object(object);
    let mut text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| intent_text.to_string());
    text.push('\n');
    text
}
