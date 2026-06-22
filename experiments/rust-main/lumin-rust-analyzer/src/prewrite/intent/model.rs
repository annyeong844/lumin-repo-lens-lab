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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::prewrite) struct RefactorSource {
    pub(in crate::prewrite) file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) lines: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) why: Option<String>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct IntentWarning {
    kind: IntentWarningKind,
    key: IntentKey,
    action: IntentWarningAction,
}

impl IntentWarning {
    pub(super) fn missing(key: IntentKey) -> Self {
        Self {
            kind: IntentWarningKind::MissingIntentKeyDefaulted,
            key,
            action: IntentWarningAction::DefaultedToEmptyArray,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum IntentWarningKind {
    MissingIntentKeyDefaulted,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(super) enum IntentKey {
    #[serde(rename = "names")]
    Names,
    #[serde(rename = "shapes")]
    Shapes,
    #[serde(rename = "files")]
    Files,
    #[serde(rename = "dependencies")]
    Dependencies,
    #[serde(rename = "plannedTypeEscapes")]
    PlannedTypeEscapes,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum IntentWarningAction {
    DefaultedToEmptyArray,
}
