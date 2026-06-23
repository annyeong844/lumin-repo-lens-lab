use serde::Serialize;

use super::declarations::{DependencyDeclaration, NameDeclaration, RefactorSource, ShapeIntent};
use super::type_escape::PlannedTypeEscape;
use super::warning::IntentWarning;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct NormalizedIntent {
    pub(in crate::prewrite) names: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite) name_declarations: Vec<NameDeclaration>,
    pub(in crate::prewrite) shapes: Vec<ShapeIntent>,
    pub(in crate::prewrite) files: Vec<String>,
    pub(in crate::prewrite) dependencies: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite) dependency_declarations: Vec<DependencyDeclaration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) refactor_sources: Option<Vec<RefactorSource>>,
    pub(in crate::prewrite) planned_type_escapes: Vec<PlannedTypeEscape>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) task_id: Option<String>,
}

impl NormalizedIntent {
    pub(in crate::prewrite) fn declaration_for(&self, name: &str) -> Option<&NameDeclaration> {
        self.name_declarations
            .iter()
            .find(|declaration| declaration.name == name)
    }

    pub(in crate::prewrite) fn refactor_sources(&self) -> &[RefactorSource] {
        self.refactor_sources.as_deref().unwrap_or(&[])
    }

    pub(in crate::prewrite) fn has_refactor_sources(&self) -> bool {
        !self.refactor_sources().is_empty()
    }
}

#[derive(Debug)]
pub(in crate::prewrite) struct LoadedIntent {
    pub(in crate::prewrite) intent: NormalizedIntent,
    pub(in crate::prewrite) warnings: Vec<IntentWarning>,
}
