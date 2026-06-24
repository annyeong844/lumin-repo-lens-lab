use serde::Serialize;

use super::super::Location;
use super::{AstCallableKind, AstFunctionOwner, AstVisibility};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionBodyFingerprint {
    pub kind: AstFunctionBodyFingerprintKind,
    pub name: String,
    pub visibility: AstVisibility,
    pub callable_kind: AstCallableKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<AstFunctionOwner>,
    pub normalized_version: &'static str,
    pub exact_body_hash: String,
    pub normalized_exact_hash: String,
    pub normalized_structure_hash: String,
    pub body_loc: usize,
    pub statement_count: usize,
    pub param_count: usize,
    #[serde(rename = "async")]
    pub is_async: bool,
    #[serde(rename = "unsafe")]
    pub is_unsafe: bool,
    #[serde(rename = "const")]
    pub is_const: bool,
    pub call_tokens: Vec<String>,
    pub location: Location,
    pub body_location: Location,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstFunctionBodyFingerprintKind {
    FunctionBodyFingerprint,
}
