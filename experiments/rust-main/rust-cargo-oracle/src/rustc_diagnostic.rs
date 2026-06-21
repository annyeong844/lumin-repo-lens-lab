use serde::Deserialize;

mod code;

use self::code::RustcDiagnosticCode;
use crate::protocol::{CodePresence, DiagnosticCode, RustcDiagnosticLevel};
use crate::rustc_span::{RustcSpan, RustcSuggestionSpan};

#[derive(Debug, Clone)]
pub(crate) struct RustcDiagnostic {
    level: Option<RustcDiagnosticLevel>,
    message: Option<String>,
    code: RustcDiagnosticCode,
    rendered_first_line: Option<String>,
    spans: Vec<RustcSpan>,
    children: Vec<RustcDiagnostic>,
}

impl RustcDiagnostic {
    pub(crate) fn level(&self) -> Option<&RustcDiagnosticLevel> {
        self.level.as_ref()
    }

    pub(crate) fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub(crate) fn code_presence(&self) -> CodePresence {
        self.code().presence()
    }

    pub(crate) fn code_text(&self) -> Option<&str> {
        self.code().text()
    }

    fn code(&self) -> &RustcDiagnosticCode {
        &self.code
    }

    pub(crate) fn rendered_first_line(&self) -> Option<&str> {
        self.rendered_first_line.as_deref()
    }

    pub(crate) fn spans(&self) -> &[RustcSpan] {
        &self.spans
    }

    pub(crate) fn suggestion_candidate_spans(&self) -> Vec<RustcSuggestionSpan> {
        let mut spans = Vec::new();
        self.collect_suggestion_candidate_spans(&mut spans);
        spans
    }

    fn collect_suggestion_candidate_spans(&self, spans: &mut Vec<RustcSuggestionSpan>) {
        for span in self.spans() {
            if span.is_primary() && span.has_suggestion_payload() {
                spans.push(RustcSuggestionSpan::from_rustc_span(span));
            }
        }

        for child in &self.children {
            child.collect_suggestion_candidate_spans(spans);
        }
    }
}

impl DiagnosticCode {
    pub(crate) fn from_rustc_diagnostic(diagnostic: &RustcDiagnostic) -> Self {
        diagnostic.code().to_protocol()
    }
}

#[derive(Debug, Deserialize)]
struct RawRustcDiagnostic {
    #[serde(default, deserialize_with = "optional_diagnostic_level")]
    level: Option<RustcDiagnosticLevel>,
    #[serde(default, deserialize_with = "optional_string")]
    message: Option<String>,
    #[serde(default)]
    code: RustcDiagnosticCode,
    #[serde(default, deserialize_with = "optional_string")]
    rendered: Option<String>,
    #[serde(default, deserialize_with = "lossy_vec")]
    spans: Vec<RustcSpan>,
    #[serde(default, deserialize_with = "lossy_vec")]
    children: Vec<RustcDiagnostic>,
}

impl From<RawRustcDiagnostic> for RustcDiagnostic {
    fn from(raw: RawRustcDiagnostic) -> Self {
        let rendered_first_line = raw
            .rendered
            .as_deref()
            .and_then(|text| text.lines().find(|line| !line.trim().is_empty()))
            .map(str::to_string);

        Self {
            level: raw.level,
            message: raw.message,
            code: raw.code,
            rendered_first_line,
            spans: raw.spans,
            children: raw.children,
        }
    }
}

impl<'de> Deserialize<'de> for RustcDiagnostic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        RawRustcDiagnostic::deserialize(deserializer).map(Into::into)
    }
}

fn optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer).ok().flatten())
}

fn optional_diagnostic_level<'de, D>(
    deserializer: D,
) -> Result<Option<RustcDiagnosticLevel>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)
        .ok()
        .flatten()
        .map(RustcDiagnosticLevel::from_rustc_str))
}

fn lossy_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Vec::<Lossy<T>>::deserialize(deserializer)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.0)
        .collect())
}

struct Lossy<T>(Option<T>);

impl<'de, T> Deserialize<'de> for Lossy<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(T::deserialize(deserializer).ok()))
    }
}
