use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::prewrite) struct NameDeclaration {
    pub(in crate::prewrite) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) why: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) owner_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) target_file: Option<String>,
}

impl NameDeclaration {
    pub(in crate::prewrite) fn effective_owner_file(&self) -> Option<&str> {
        self.owner_file
            .as_deref()
            .or(self.file.as_deref())
            .or(self.target_file.as_deref())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct ShapeIntent {
    pub(in crate::prewrite) fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) type_literal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) why: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::prewrite) struct DependencyDeclaration {
    pub(in crate::prewrite) specifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) why: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::prewrite) struct RefactorSource {
    pub(in crate::prewrite) file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) lines: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) why: Option<String>,
}
