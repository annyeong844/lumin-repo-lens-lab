use anyhow::Result;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_malformed_intent_hard_stops_before_writing_artifact() -> Result<()> {
    let cases = [
        ("malformed-json", "{", "invalid --intent"),
        (
            "present-null-names",
            r#"{"names":null,"shapes":[],"files":[],"dependencies":[],"plannedTypeEscapes":[]}"#,
            "invalid --intent",
        ),
        (
            "empty-name",
            r#"{"names":[""],"shapes":[],"files":[],"dependencies":[],"plannedTypeEscapes":[]}"#,
            "names[0] must be a non-empty string",
        ),
        (
            "empty-task-id",
            r#"{"taskId":"","names":[],"shapes":[],"files":[],"dependencies":[],"plannedTypeEscapes":[]}"#,
            "taskId must be a non-empty string",
        ),
        (
            "unknown-field",
            r#"{"extra":true,"names":[],"shapes":[],"files":[],"dependencies":[],"plannedTypeEscapes":[]}"#,
            "unknown field",
        ),
        (
            "refactor-source-parent-traversal",
            r#"{"names":[],"shapes":[],"files":[],"dependencies":[],"plannedTypeEscapes":[],"refactorSources":[{"file":"../src/lib.rs"}]}"#,
            "refactorSources[0].file must be a repository-relative path",
        ),
        (
            "refactor-source-windows-path",
            r#"{"names":[],"shapes":[],"files":[],"dependencies":[],"plannedTypeEscapes":[],"refactorSources":[{"file":"src\\lib.rs"}]}"#,
            "refactorSources[0].file must be a repository-relative path",
        ),
        (
            "refactor-source-zero-line",
            r#"{"names":[],"shapes":[],"files":[],"dependencies":[],"plannedTypeEscapes":[],"refactorSources":[{"file":"src/lib.rs","lines":[0]}]}"#,
            "refactorSources[0].lines[0] must be a positive integer",
        ),
        (
            "planned-type-escape-empty-reason",
            r#"{"names":[],"shapes":[],"files":[],"dependencies":[],"plannedTypeEscapes":[{"escapeKind":"as-any","locationHint":"src/lib.rs","reason":""}]}"#,
            "plannedTypeEscapes[0].reason must be a non-empty string",
        ),
        (
            "planned-type-escape-invalid-kind",
            r#"{"names":[],"shapes":[],"files":[],"dependencies":[],"plannedTypeEscapes":[{"escapeKind":"any","locationHint":"src/lib.rs","reason":"test"}]}"#,
            "unknown variant",
        ),
    ];

    for (label, intent, expected_error) in cases {
        let repo = PreWriteRepo::new()?;
        let output = repo.run(intent)?;
        assert_eq!(output.status.code(), Some(2), "case: {label}");
        assert!(output.stdout.is_empty(), "case: {label}");
        assert!(!repo.output_exists(), "case: {label}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected_error),
            "case: {label}; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
