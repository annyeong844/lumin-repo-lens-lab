use serde::{Deserialize, Serialize};

use super::super::Location;
use super::AstVisibility;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionSignature {
    pub kind: AstFunctionSignatureKind,
    pub hash: String,
    pub name: String,
    pub visibility: AstVisibility,
    pub callable_kind: AstCallableKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<AstFunctionOwner>,
    pub normalized_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generics: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<AstFunctionReceiver>,
    pub params: Vec<AstFunctionParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,
    pub location: Location,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstFunctionSignatureKind {
    FunctionSignature,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstCallableKind {
    Function,
    ImplMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionOwner {
    pub target: String,
    #[serde(rename = "trait", skip_serializing_if = "Option::is_none")]
    pub trait_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionReceiver {
    pub kind: AstFunctionReceiverKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstFunctionReceiverKind {
    Owned,
    Ref,
    MutRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionParam {
    #[serde(rename = "type")]
    pub type_text: String,
}
