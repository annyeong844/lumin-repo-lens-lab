use serde::Serialize;

use lumin_rust_source_health::protocol::{
    AstShapeConfidence, AstShapeField, AstShapeHash, AstShapeKind, AstVisibility,
    PathClassification,
};

use super::is_path_policy_excluded;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct ShapeMatch {
    identity: String,
    owner_file: String,
    name: String,
    hash: String,
    shape_kind: AstShapeKind,
    fields: Vec<AstShapeField>,
    visibility: AstVisibility,
    confidence: AstShapeConfidence,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    path_classifications: Vec<PathClassification>,
    policy_excluded: bool,
}

impl ShapeMatch {
    pub(in crate::prewrite::lookup::shape) fn from_fact(
        owner_file: &str,
        fact: &AstShapeHash,
        path_classifications: Vec<PathClassification>,
        path_suppressed: bool,
    ) -> Self {
        let identity = format!("{owner_file}::{}", fact.name);
        Self {
            identity,
            owner_file: owner_file.to_string(),
            name: fact.name.clone(),
            hash: fact.hash.clone(),
            shape_kind: fact.shape_kind,
            fields: fact.fields.clone(),
            visibility: fact.visibility,
            confidence: fact.confidence,
            policy_excluded: is_path_policy_excluded(&path_classifications, path_suppressed),
            path_classifications,
        }
    }

    pub(super) fn identity(&self) -> &str {
        &self.identity
    }

    pub(super) fn owner_file(&self) -> &str {
        &self.owner_file
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn path_classifications(&self) -> &[PathClassification] {
        &self.path_classifications
    }

    pub(super) fn policy_excluded(&self) -> bool {
        self.policy_excluded
    }
}
