use serde::Serialize;

use super::super::Location;
use super::AstVisibility;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstShapeHash {
    pub kind: AstShapeHashKind,
    pub hash: String,
    pub name: String,
    pub visibility: AstVisibility,
    pub shape_kind: AstShapeKind,
    pub normalized_version: &'static str,
    pub confidence: AstShapeConfidence,
    pub fields: Vec<AstShapeField>,
    pub location: Location,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstShapeHashKind {
    ShapeHash,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstShapeKind {
    RecordStruct,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstShapeConfidence {
    High,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstShapeField {
    pub kind: AstShapeFieldKind,
    pub name: String,
    #[serde(rename = "type")]
    pub type_text: String,
    pub visibility: AstVisibility,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstShapeFieldKind {
    Property,
}
