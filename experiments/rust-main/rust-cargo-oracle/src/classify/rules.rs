use super::model::Classification;
use crate::protocol::{
    ClaimKind, ClassificationRule, CodeKind, CodeNamespace, CodePresence, ConfidenceTier,
    CoverageEffect, Disposition, RustcDiagnosticLevel,
};

pub(super) fn classify_diagnostic(
    level: Option<&RustcDiagnosticLevel>,
    code_namespace: CodeNamespace,
    has_user_primary: bool,
) -> Classification {
    let Some(level) = level.filter(|level| level.is_error() || level.is_warning()) else {
        return Classification {
            disposition: Disposition::NonFinding,
            confidence: None,
            claim_kind: None,
            coverage_effect: None,
            rule: ClassificationRule::NoteHelpFailureNoteAreNotFindings,
        };
    };

    if !has_user_primary {
        return Classification {
            disposition: if level.is_error() {
                Disposition::CoverageUnavailable
            } else {
                Disposition::NonFinding
            },
            confidence: None,
            claim_kind: None,
            coverage_effect: if level.is_error() {
                Some(CoverageEffect::AbsenceCleanUnavailable)
            } else {
                None
            },
            rule: if level.is_error() {
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

    if level.is_error() && code_namespace == CodeNamespace::RustcError {
        return Classification {
            disposition: Disposition::Finding,
            confidence: Some(ConfidenceTier::Verified),
            claim_kind: Some(ClaimKind::RustcErrorDiagnostic),
            coverage_effect: None,
            rule: ClassificationRule::EcodeErrorUserCodePrimary,
        };
    }

    if level.is_error() && code_namespace == CodeNamespace::RustcCodeless {
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

pub(super) fn code_namespace(presence: CodePresence, value: Option<&str>) -> CodeNamespace {
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

pub(super) fn code_kind(namespace: CodeNamespace) -> CodeKind {
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

#[cfg(test)]
mod tests {
    use super::code_namespace;
    use crate::protocol::{CodeNamespace, CodePresence};

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
}
