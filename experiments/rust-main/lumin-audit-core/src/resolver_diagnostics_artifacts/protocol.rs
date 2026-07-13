use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverDiagnosticsArtifactsRequest {
    pub schema_version: String,
    #[serde(default)]
    pub symbols: Value,
    #[serde(default)]
    pub capability_artifact: Option<String>,
}

#[derive(Debug)]
pub(super) struct Record<'a> {
    value: &'a Value,
}

impl<'a> Record<'a> {
    pub(super) fn new(value: &'a Value) -> Self {
        Self { value }
    }

    pub(super) fn get(&self, field: &str) -> Option<&'a Value> {
        self.value.as_object()?.get(field)
    }

    pub(super) fn str(&self, field: &str) -> Option<&'a str> {
        self.get(field).and_then(Value::as_str)
    }

    pub(super) fn bool(&self, field: &str) -> Option<bool> {
        self.get(field).and_then(Value::as_bool)
    }

    pub(super) fn number(&self, field: &str) -> Option<Value> {
        self.get(field).filter(|value| value.is_number()).cloned()
    }
}
