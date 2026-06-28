use serde::Serialize;

use lumin_rust_source_health::protocol::{
    AstCallableKind, AstFunctionSignature, AstVisibility, PathClassification,
};

use super::is_path_policy_excluded;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum SignatureVisibility {
    Exported,
    FileLocal,
    Unknown,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct SignatureMatch {
    identity: String,
    owner_file: String,
    name: String,
    hash: String,
    visibility: AstVisibility,
    callable_kind: AstCallableKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    path_classifications: Vec<PathClassification>,
    policy_excluded: bool,
    confidence: &'static str,
}

impl SignatureMatch {
    pub(in crate::prewrite::lookup::shape) fn from_fact(
        owner_file: &str,
        fact: &AstFunctionSignature,
        path_classifications: Vec<PathClassification>,
        path_suppressed: bool,
    ) -> Self {
        let identity = match &fact.owner {
            None => format!("{owner_file}::{}", fact.name),
            Some(owner) => match &owner.trait_path {
                None => format!("{owner_file}::{}#{}", owner.target, fact.name),
                Some(trait_path) => {
                    format!(
                        "{owner_file}::{} as {trait_path}#{}",
                        owner.target, fact.name
                    )
                }
            },
        };
        let local_name = if fact.name == "default" {
            None
        } else {
            Some(fact.name.clone())
        };
        Self {
            identity,
            owner_file: owner_file.to_string(),
            name: fact.name.clone(),
            hash: fact.hash.clone(),
            visibility: fact.visibility,
            callable_kind: fact.callable_kind,
            local_name,
            policy_excluded: is_path_policy_excluded(&path_classifications, path_suppressed),
            path_classifications,
            confidence: "high",
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

    pub(super) fn is_safe_surface(&self) -> bool {
        !self.policy_excluded && self.is_exported_surface()
    }

    pub(super) fn visibility_label(&self) -> SignatureVisibility {
        if self.is_exported_surface() {
            SignatureVisibility::Exported
        } else if matches!(self.visibility, AstVisibility::Private) {
            SignatureVisibility::FileLocal
        } else {
            SignatureVisibility::Unknown
        }
    }

    pub(super) fn path_classifications(&self) -> &[PathClassification] {
        &self.path_classifications
    }

    pub(super) fn policy_excluded(&self) -> bool {
        self.policy_excluded
    }

    fn is_exported_surface(&self) -> bool {
        self.callable_kind == AstCallableKind::Function
            && matches!(
                self.visibility,
                AstVisibility::Public | AstVisibility::Crate | AstVisibility::Restricted
            )
    }
}
