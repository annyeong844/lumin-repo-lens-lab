use crate::cargo_json::{CargoJsonEvent, CargoJsonMessages};
use crate::ownership::{OwnershipResolver, SpanClass};
use crate::protocol::{DiagnosticCode, PrimarySpan, PrimarySpanClass};
use crate::rustc_diagnostic::RustcDiagnostic;

use super::model::Diagnostic;
use super::rules::{classify_diagnostic, code_kind, code_namespace};

pub(crate) fn diagnostic_ledger(
    messages: CargoJsonMessages<'_>,
    ownership: &OwnershipResolver,
) -> Vec<Diagnostic> {
    messages
        .compiler_messages()
        .filter_map(|message| summarize_diagnostic_event(message, ownership))
        .collect()
}

fn summarize_diagnostic_event(
    message: CargoJsonEvent<'_>,
    ownership: &OwnershipResolver,
) -> Option<Diagnostic> {
    let package_id = message.package_id();
    let diagnostic = message.rustc_diagnostic()?;
    let level = diagnostic.level().cloned();
    let code_presence = diagnostic.code_presence();
    let code_value = diagnostic.code_text().map(str::to_string);
    let code_namespace = code_namespace(code_presence, code_value.as_deref());
    let code_kind = code_kind(code_namespace);
    let primary_spans = primary_spans(diagnostic, ownership, package_id);
    let suggestion_candidate_spans = diagnostic.suggestion_candidate_spans();
    let has_user_primary = primary_spans.iter().any(PrimarySpan::is_user_code);
    let classification = classify_diagnostic(level.as_ref(), code_namespace, has_user_primary);
    let rendered_first_line = diagnostic.rendered_first_line().map(str::to_string);

    Some(Diagnostic {
        level,
        raw_code: DiagnosticCode::from_rustc_diagnostic(diagnostic),
        code_presence,
        code_value,
        code_namespace,
        code_kind,
        primary_spans,
        suggestion_candidate_spans,
        classification,
        message: diagnostic.message().map(str::to_string),
        rendered_first_line,
    })
}

fn primary_spans(
    diagnostic: &RustcDiagnostic,
    ownership: &OwnershipResolver,
    package_id: Option<&str>,
) -> Vec<PrimarySpan> {
    diagnostic
        .spans()
        .iter()
        .filter(|span| span.is_primary())
        .map(|span| {
            let span_class = ownership.classify_span_for_package(span, package_id);
            PrimarySpan::from_rustc_span(span, primary_span_class(span_class))
        })
        .collect()
}

fn primary_span_class(span_class: SpanClass) -> PrimarySpanClass {
    match span_class {
        SpanClass::UserCode => PrimarySpanClass::UserCode,
        SpanClass::Dependency => PrimarySpanClass::Dependency,
        SpanClass::Generated => PrimarySpanClass::Generated,
        SpanClass::Unknown => PrimarySpanClass::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::diagnostic_ledger;
    use crate::cargo_json::CargoJsonStream;
    use crate::ownership::OwnershipResolver;
    use crate::protocol::{ClaimKind, ConfidenceTier, Disposition, RustcDiagnosticLevel};
    use anyhow::Result;

    #[test]
    fn metadata_unavailable_root_src_error_remains_user_finding() -> Result<()> {
        let temp = tempfile::TempDir::new()?;
        let root = temp.path().join("crate");
        std::fs::create_dir_all(root.join("src"))?;
        let ownership = OwnershipResolver::new(&root, None, &[]);
        let mut messages = CargoJsonStream::empty();
        messages.push_json_line(
            r#"{"reason":"compiler-message","package_id":"path+file:///unknown#0.1.0","message":{"level":"error","message":"mismatched types","code":{"code":"E0308"},"spans":[{"file_name":"src/lib.rs","is_primary":true,"line_start":1,"line_end":1,"column_start":1,"column_end":2,"expansion":null}]}}"#,
        )?;

        let diagnostics = diagnostic_ledger(messages.as_messages(), &ownership);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].classification.disposition,
            Disposition::Finding
        );
        assert_eq!(
            diagnostics[0].classification.confidence,
            Some(ConfidenceTier::Verified)
        );
        assert_eq!(
            diagnostics[0].classification.claim_kind,
            Some(ClaimKind::RustcErrorDiagnostic)
        );
        Ok(())
    }

    #[test]
    fn future_diagnostic_level_is_preserved_without_becoming_a_finding() -> Result<()> {
        let temp = tempfile::TempDir::new()?;
        let root = temp.path().join("crate");
        std::fs::create_dir_all(root.join("src"))?;
        let ownership = OwnershipResolver::new(&root, None, &[]);
        let mut messages = CargoJsonStream::empty();
        messages.push_json_line(
            r#"{"reason":"compiler-message","package_id":"path+file:///unknown#0.1.0","message":{"level":"future-severity","message":"future diagnostic","code":{"code":"E0308"},"spans":[{"file_name":"src/lib.rs","is_primary":true,"line_start":1,"line_end":1,"column_start":1,"column_end":2,"expansion":null}]}}"#,
        )?;

        let diagnostics = diagnostic_ledger(messages.as_messages(), &ownership);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].level,
            Some(RustcDiagnosticLevel::Other("future-severity".to_string()))
        );
        assert_eq!(
            diagnostics[0].classification.disposition,
            Disposition::NonFinding
        );
        assert_eq!(diagnostics[0].classification.confidence, None);
        assert_eq!(diagnostics[0].classification.claim_kind, None);
        Ok(())
    }
}
