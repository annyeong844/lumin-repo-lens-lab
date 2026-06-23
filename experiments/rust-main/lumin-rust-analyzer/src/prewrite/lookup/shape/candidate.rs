use serde::Serialize;

use lumin_rust_source_health::protocol::{
    AstCallableKind, AstFunctionSignature, AstShapeConfidence, AstShapeField, AstShapeHash,
    AstShapeKind, AstVisibility, PathClassification,
};

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(in crate::prewrite) enum ShapeLookupMatch {
    Shape(ShapeMatch),
    Signature(SignatureMatch),
}

impl ShapeLookupMatch {
    pub(in crate::prewrite) fn identity(&self) -> &str {
        match self {
            Self::Shape(candidate) => &candidate.identity,
            Self::Signature(candidate) => &candidate.identity,
        }
    }

    pub(in crate::prewrite) fn owner_file(&self) -> &str {
        match self {
            Self::Shape(candidate) => &candidate.owner_file,
            Self::Signature(candidate) => &candidate.owner_file,
        }
    }

    pub(in crate::prewrite) fn name(&self) -> &str {
        match self {
            Self::Shape(candidate) => &candidate.name,
            Self::Signature(candidate) => &candidate.name,
        }
    }

    pub(in crate::prewrite) fn is_safe_signature_surface(&self) -> bool {
        match self {
            Self::Shape(_) => false,
            Self::Signature(candidate) => {
                if candidate.policy_excluded {
                    return false;
                }
                candidate.callable_kind == AstCallableKind::Function
                    && matches!(
                        candidate.visibility,
                        AstVisibility::Public | AstVisibility::Crate | AstVisibility::Restricted
                    )
            }
        }
    }

    pub(in crate::prewrite) fn signature_visibility(&self) -> Option<SignatureVisibility> {
        match self {
            Self::Shape(_) => None,
            Self::Signature(candidate) => Some(candidate.visibility_label()),
        }
    }

    pub(in crate::prewrite) fn path_classifications(&self) -> &[PathClassification] {
        match self {
            Self::Shape(candidate) => &candidate.path_classifications,
            Self::Signature(candidate) => &candidate.path_classifications,
        }
    }

    pub(in crate::prewrite) fn policy_excluded(&self) -> bool {
        match self {
            Self::Shape(candidate) => candidate.policy_excluded,
            Self::Signature(candidate) => candidate.policy_excluded,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum SignatureVisibility {
    Exported,
    FileLocal,
    Unknown,
}

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

    fn visibility_label(&self) -> SignatureVisibility {
        if self.is_exported_surface() {
            SignatureVisibility::Exported
        } else if matches!(self.visibility, AstVisibility::Private) {
            SignatureVisibility::FileLocal
        } else {
            SignatureVisibility::Unknown
        }
    }

    fn is_exported_surface(&self) -> bool {
        self.callable_kind == AstCallableKind::Function
            && matches!(
                self.visibility,
                AstVisibility::Public | AstVisibility::Crate | AstVisibility::Restricted
            )
    }
}

fn is_path_policy_excluded(
    path_classifications: &[PathClassification],
    path_suppressed: bool,
) -> bool {
    path_suppressed
        || path_classifications.iter().any(|classification| {
            matches!(
                classification,
                PathClassification::Test | PathClassification::Generated
            )
        })
}
