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
