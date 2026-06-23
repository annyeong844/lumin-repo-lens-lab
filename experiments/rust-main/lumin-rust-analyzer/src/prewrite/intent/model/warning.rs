use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct IntentWarning {
    kind: IntentWarningKind,
    key: IntentKey,
    action: IntentWarningAction,
}

impl IntentWarning {
    pub(in crate::prewrite) fn missing(key: IntentKey) -> Self {
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
pub(in crate::prewrite) enum IntentKey {
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
