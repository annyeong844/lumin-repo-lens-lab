use serde_json::{json, Value};

use crate::ownership::OwnershipResolver;
use crate::protocol::{
    ClaimKind, ClassificationRule, CodeKind, CodeNamespace, CodePresence, ConfidenceTier,
    CoverageEffect, Disposition,
};

#[derive(Debug, Clone)]
pub(crate) struct Classification {
    pub(crate) disposition: Disposition,
    pub(crate) confidence: Option<ConfidenceTier>,
    pub(crate) claim_kind: Option<ClaimKind>,
    pub(crate) coverage_effect: Option<CoverageEffect>,
    pub(crate) rule: ClassificationRule,
}

#[derive(Debug, Clone)]
pub(crate) struct Diagnostic {
    pub(crate) level: Option<String>,
    pub(crate) raw: Value,
    pub(crate) code_presence: CodePresence,
    pub(crate) code_value: Option<String>,
    pub(crate) code_namespace: CodeNamespace,
    pub(crate) code_kind: CodeKind,
    pub(crate) primary_spans: Vec<Value>,
    pub(crate) classification: Classification,
    pub(crate) message: Option<String>,
    pub(crate) rendered_first_line: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedJsonl {
    pub(crate) messages: Vec<Value>,
    pub(crate) invalid_json_line_count: usize,
    pub(crate) stream_parse_status: &'static str,
}

pub(crate) fn parse_cargo_jsonl(stdout: &str, timed_out: bool) -> ParsedJsonl {
    let mut messages = Vec::new();
    let mut invalid_json_line_count = 0;
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        match serde_json::from_str::<Value>(line) {
            Ok(value) => messages.push(value),
            Err(_) => invalid_json_line_count += 1,
        }
    }

    let stream_parse_status = if timed_out {
        "timeout"
    } else if messages.is_empty() && invalid_json_line_count == 0 {
        "no-json-events"
    } else if invalid_json_line_count == 0 {
        "complete"
    } else {
        "invalid-json"
    };

    ParsedJsonl {
        messages,
        invalid_json_line_count,
        stream_parse_status,
    }
}

pub(crate) fn diagnostic_ledger(
    messages: &[Value],
    ownership: &OwnershipResolver,
) -> Vec<Diagnostic> {
    messages
        .iter()
        .filter(|message| message.get("reason").and_then(Value::as_str) == Some("compiler-message"))
        .filter_map(|message| summarize_diagnostic_event(message, ownership))
        .collect()
}

fn summarize_diagnostic_event(
    message: &Value,
    ownership: &OwnershipResolver,
) -> Option<Diagnostic> {
    let diagnostic = message.get("message")?;
    let package_id = message.get("package_id").and_then(Value::as_str);
    let level = diagnostic
        .get("level")
        .and_then(Value::as_str)
        .map(str::to_string);
    let code = diagnostic.get("code");
    let code_presence = code_presence(code);
    let code_value = code_value(code);
    let code_namespace = code_namespace(code_presence, code_value.as_deref());
    let code_kind = code_kind(code_namespace);
    let primary_spans = primary_spans(diagnostic, ownership, package_id);
    let has_user_primary = primary_spans
        .iter()
        .any(|span| span.get("primarySpanClass").and_then(Value::as_str) == Some("user-code"));
    let classification = classify_diagnostic(level.as_deref(), code_namespace, has_user_primary);
    let rendered_first_line = diagnostic
        .get("rendered")
        .and_then(Value::as_str)
        .and_then(|text| text.lines().find(|line| !line.trim().is_empty()))
        .map(str::to_string);

    Some(Diagnostic {
        level,
        raw: diagnostic.clone(),
        code_presence,
        code_value,
        code_namespace,
        code_kind,
        primary_spans,
        classification,
        message: diagnostic
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_string),
        rendered_first_line,
    })
}

fn classify_diagnostic(
    level: Option<&str>,
    code_namespace: CodeNamespace,
    has_user_primary: bool,
) -> Classification {
    if !matches!(level, Some("error" | "warning")) {
        return Classification {
            disposition: Disposition::NonFinding,
            confidence: None,
            claim_kind: None,
            coverage_effect: None,
            rule: ClassificationRule::NoteHelpFailureNoteAreNotFindings,
        };
    }

    if !has_user_primary {
        return Classification {
            disposition: if level == Some("error") {
                Disposition::CoverageUnavailable
            } else {
                Disposition::NonFinding
            },
            confidence: None,
            claim_kind: None,
            coverage_effect: if level == Some("error") {
                Some(CoverageEffect::AbsenceCleanUnavailable)
            } else {
                None
            },
            rule: if level == Some("error") {
                ClassificationRule::NonUserPrimaryErrorMakesAbsenceCleanUnavailable
            } else {
                ClassificationRule::NonUserPrimaryDiagnosticsAreNotUserFacingFindings
            },
        };
    }

    if code_namespace == CodeNamespace::RustcNonEcode {
        return Classification {
            disposition: Disposition::Finding,
            confidence: Some(ConfidenceTier::RuleBacked),
            claim_kind: Some(ClaimKind::RustcLintDiagnostic),
            coverage_effect: None,
            rule: ClassificationRule::NonEcodeCodeNameTreatedAsRuleBackedBeforeLevel,
        };
    }

    if level == Some("error") && code_namespace == CodeNamespace::RustcError {
        return Classification {
            disposition: Disposition::Finding,
            confidence: Some(ConfidenceTier::Verified),
            claim_kind: Some(ClaimKind::RustcErrorDiagnostic),
            coverage_effect: None,
            rule: ClassificationRule::EcodeErrorUserCodePrimary,
        };
    }

    if level == Some("error") && code_namespace == CodeNamespace::RustcCodeless {
        return Classification {
            disposition: Disposition::Finding,
            confidence: Some(ConfidenceTier::Verified),
            claim_kind: Some(ClaimKind::RustcCodelessErrorDiagnostic),
            coverage_effect: None,
            rule: ClassificationRule::CodelessErrorUserCodePrimary,
        };
    }

    Classification {
        disposition: Disposition::Finding,
        confidence: Some(ConfidenceTier::Candidate),
        claim_kind: Some(ClaimKind::UnclassifiedCargoDiagnostic),
        coverage_effect: None,
        rule: ClassificationRule::FallbackRealWarningOrErrorNeverVerified,
    }
}

fn code_presence(code: Option<&Value>) -> CodePresence {
    match code {
        Some(Value::Null) => CodePresence::PresentNull,
        None => CodePresence::Omitted,
        Some(_) => CodePresence::PresentValue,
    }
}

fn code_value(code: Option<&Value>) -> Option<String> {
    match code {
        Some(Value::Object(map)) => map.get("code").and_then(Value::as_str).map(str::to_string),
        Some(Value::String(value)) => Some(value.clone()),
        _ => None,
    }
}

fn code_namespace(presence: CodePresence, value: Option<&str>) -> CodeNamespace {
    if presence == CodePresence::PresentNull {
        CodeNamespace::RustcCodeless
    } else if presence == CodePresence::Omitted {
        CodeNamespace::Unknown
    } else if value.is_some_and(is_rust_error_code) {
        CodeNamespace::RustcError
    } else if value.is_some_and(|value| !value.is_empty()) {
        CodeNamespace::RustcNonEcode
    } else {
        CodeNamespace::Unknown
    }
}

fn code_kind(namespace: CodeNamespace) -> CodeKind {
    match namespace {
        CodeNamespace::RustcCodeless => CodeKind::NullErrorCode,
        CodeNamespace::RustcError => CodeKind::RustcErrorCode,
        CodeNamespace::RustcNonEcode => CodeKind::NonEcodeName,
        CodeNamespace::Unknown => CodeKind::Unknown,
    }
}

fn is_rust_error_code(value: &str) -> bool {
    value.strip_prefix('E').is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn primary_spans(
    diagnostic: &Value,
    ownership: &OwnershipResolver,
    package_id: Option<&str>,
) -> Vec<Value> {
    diagnostic
        .get("spans")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|span| span.get("is_primary").and_then(Value::as_bool) == Some(true))
        .map(|span| {
            let span_class = ownership.classify_span_for_package(span, package_id);
            json!({
                "file_name": span.get("file_name").and_then(Value::as_str),
                "line_start": span.get("line_start").and_then(Value::as_i64),
                "line_end": span.get("line_end").and_then(Value::as_i64),
                "column_start": span.get("column_start").and_then(Value::as_i64),
                "column_end": span.get("column_end").and_then(Value::as_i64),
                "has_expansion": span.get("expansion").is_some_and(|value| !value.is_null()),
                "expansion": span.get("expansion").cloned().unwrap_or(Value::Null),
                "primarySpanClass": span_class.as_str(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_code_namespaces() {
        assert_eq!(
            code_namespace(CodePresence::PresentNull, None),
            CodeNamespace::RustcCodeless
        );
        assert_eq!(
            code_namespace(CodePresence::PresentValue, Some("E0308")),
            CodeNamespace::RustcError
        );
        assert_eq!(
            code_namespace(CodePresence::PresentValue, Some("E12345")),
            CodeNamespace::RustcError
        );
        assert_eq!(
            code_namespace(CodePresence::PresentValue, Some("E")),
            CodeNamespace::RustcNonEcode
        );
        assert_eq!(
            code_namespace(CodePresence::PresentValue, Some("E12A4")),
            CodeNamespace::RustcNonEcode
        );
        assert_eq!(
            code_namespace(CodePresence::PresentValue, Some("unused_variables")),
            CodeNamespace::RustcNonEcode
        );
        assert_eq!(
            code_namespace(CodePresence::Omitted, None),
            CodeNamespace::Unknown
        );
    }

    #[test]
    fn timeout_preserves_already_emitted_json_messages() {
        let parsed = parse_cargo_jsonl(
            r#"{"reason":"compiler-message","message":{"level":"error","code":null,"spans":[]}}"#,
            true,
        );

        assert_eq!(parsed.stream_parse_status, "timeout");
        assert_eq!(parsed.invalid_json_line_count, 0);
        assert_eq!(parsed.messages.len(), 1);
        assert_eq!(
            parsed.messages[0].get("reason").and_then(Value::as_str),
            Some("compiler-message")
        );
    }

    #[test]
    fn metadata_unavailable_root_src_error_remains_user_finding() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let root = temp.path().join("crate");
        std::fs::create_dir_all(root.join("src")).expect("src dir");
        let ownership = OwnershipResolver::new(&root, None, &[]);
        let messages = vec![json!({
            "reason": "compiler-message",
            "package_id": "path+file:///unknown#0.1.0",
            "message": {
                "level": "error",
                "message": "mismatched types",
                "code": {"code": "E0308"},
                "spans": [{
                    "file_name": "src/lib.rs",
                    "is_primary": true,
                    "line_start": 1,
                    "line_end": 1,
                    "column_start": 1,
                    "column_end": 2,
                    "expansion": null
                }]
            }
        })];

        let diagnostics = diagnostic_ledger(&messages, &ownership);

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
    }
}
