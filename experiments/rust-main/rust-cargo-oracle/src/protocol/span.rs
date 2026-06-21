use serde::Serialize;

use super::PrimarySpanClass;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimarySpan {
    pub file_name: Option<String>,
    pub line_start: Option<i64>,
    pub line_end: Option<i64>,
    pub column_start: Option<i64>,
    pub column_end: Option<i64>,
    pub has_expansion: bool,
    pub expansion: Option<PrimarySpanExpansion>,
    pub primary_span_class: PrimarySpanClass,
}

impl PrimarySpan {
    pub fn representative(spans: &[Self]) -> Option<&Self> {
        spans
            .iter()
            .find(|span| span.is_user_code())
            .or_else(|| spans.first())
    }

    pub fn representative_class(spans: &[Self]) -> PrimarySpanClass {
        Self::representative(spans)
            .map(|span| span.primary_span_class)
            .unwrap_or(PrimarySpanClass::Unknown)
    }

    pub fn is_user_code(&self) -> bool {
        self.primary_span_class == PrimarySpanClass::UserCode
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimarySpanExpansion {
    pub macro_decl_name: Option<String>,
    pub span: Option<PrimarySpanLocation>,
    pub def_site_span: Option<PrimarySpanLocation>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimarySpanLocation {
    pub file_name: Option<String>,
    pub line_start: Option<i64>,
    pub line_end: Option<i64>,
    pub column_start: Option<i64>,
    pub column_end: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::{PrimarySpan, PrimarySpanClass};
    use anyhow::{anyhow, Result};

    fn primary_span(file_name: &str, primary_span_class: PrimarySpanClass) -> PrimarySpan {
        PrimarySpan {
            file_name: Some(file_name.to_string()),
            line_start: None,
            line_end: None,
            column_start: None,
            column_end: None,
            has_expansion: false,
            expansion: None,
            primary_span_class,
        }
    }

    #[test]
    fn serializes_public_contract_as_camel_case() -> Result<()> {
        let span = PrimarySpan {
            file_name: Some("src/lib.rs".to_string()),
            line_start: Some(2),
            line_end: Some(3),
            column_start: Some(4),
            column_end: Some(5),
            has_expansion: false,
            expansion: None,
            primary_span_class: PrimarySpanClass::UserCode,
        };

        let value = serde_json::to_value(span)?;
        let object = value
            .as_object()
            .ok_or_else(|| anyhow!("primary span serializes to object"))?;

        assert_eq!(value["fileName"], "src/lib.rs");
        assert_eq!(value["lineStart"], 2);
        assert_eq!(value["lineEnd"], 3);
        assert_eq!(value["columnStart"], 4);
        assert_eq!(value["columnEnd"], 5);
        assert_eq!(value["hasExpansion"], false);
        assert_eq!(value["expansion"], serde_json::Value::Null);
        assert_eq!(value["primarySpanClass"], "user-code");

        for forbidden_key in [
            "file_name",
            "line_start",
            "line_end",
            "column_start",
            "column_end",
            "has_expansion",
            "primary_span_class",
        ] {
            assert!(
                !object.contains_key(forbidden_key),
                "primary span leaked snake_case key {forbidden_key}"
            );
        }
        Ok(())
    }

    #[test]
    fn representative_prefers_user_code_primary_span() {
        let spans = vec![
            primary_span("bad_dep/src/lib.rs", PrimarySpanClass::Dependency),
            primary_span("src/lib.rs", PrimarySpanClass::UserCode),
        ];

        let span = PrimarySpan::representative(&spans)
            .map(|span| (span.file_name.as_deref(), span.primary_span_class));

        assert_eq!(span, Some((Some("src/lib.rs"), PrimarySpanClass::UserCode)));
        assert_eq!(
            PrimarySpan::representative_class(&spans),
            PrimarySpanClass::UserCode
        );
    }

    #[test]
    fn representative_falls_back_to_first_primary_span() {
        let spans = vec![
            primary_span("target/generated.rs", PrimarySpanClass::Generated),
            primary_span("bad_dep/src/lib.rs", PrimarySpanClass::Dependency),
        ];

        let span = PrimarySpan::representative(&spans)
            .map(|span| (span.file_name.as_deref(), span.primary_span_class));

        assert_eq!(
            span,
            Some((Some("target/generated.rs"), PrimarySpanClass::Generated))
        );
        assert_eq!(
            PrimarySpan::representative_class(&spans),
            PrimarySpanClass::Generated
        );
    }

    #[test]
    fn representative_is_none_without_primary_spans() {
        assert_eq!(PrimarySpan::representative(&[]), None);
        assert_eq!(
            PrimarySpan::representative_class(&[]),
            PrimarySpanClass::Unknown
        );
    }
}
