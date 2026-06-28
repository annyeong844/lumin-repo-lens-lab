use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::prewrite) struct PlannedTypeEscape {
    escape_kind: EscapeKind,
    pub(in crate::prewrite) location_hint: String,
    pub(in crate::prewrite) reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code_shape: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alternative_considered: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub(in crate::prewrite) enum EscapeKind {
    #[serde(rename = "explicit-any")]
    ExplicitAny,
    #[serde(rename = "as-any")]
    AsAny,
    #[serde(rename = "angle-any")]
    AngleAny,
    #[serde(rename = "as-unknown-as-T")]
    AsUnknownAsType,
    #[serde(rename = "rest-any-args")]
    RestAnyArgs,
    #[serde(rename = "index-sig-any")]
    IndexSignatureAny,
    #[serde(rename = "generic-default-any")]
    GenericDefaultAny,
    #[serde(rename = "ts-ignore")]
    TsIgnore,
    #[serde(rename = "ts-expect-error")]
    TsExpectError,
    #[serde(rename = "no-explicit-any-disable")]
    NoExplicitAnyDisable,
    #[serde(rename = "jsdoc-any")]
    JsdocAny,
}
