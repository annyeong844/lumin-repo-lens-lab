use serde::Serialize;

use super::super::Location;
use super::AstVisibility;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstImplBlock {
    pub target: String,
    #[serde(rename = "trait", skip_serializing_if = "Option::is_none")]
    pub trait_path: Option<String>,
    pub methods: Vec<AstImplMethod>,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstImplMethod {
    pub name: String,
    pub visibility: AstVisibility,
    pub has_receiver: bool,
    pub location: Location,
}
