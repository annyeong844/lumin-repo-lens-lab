use crate::support::sidecar::run_with_stdin;

#[test]
fn malformed_request_json_exits_2_without_stdout() -> anyhow::Result<()> {
    let output = run_with_stdin("{")?;

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid request JSON"));
    Ok(())
}

#[test]
fn unsupported_schema_exits_2_without_stdout() -> anyhow::Result<()> {
    let output = run_with_stdin(
        r#"{"schemaVersion":999,"root":".","files":[],"policyVersion":"module-edge-scanner-v1"}"#,
    )?;

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported schemaVersion 999"));
    Ok(())
}

#[test]
fn unsupported_policy_version_exits_2_without_stdout() -> anyhow::Result<()> {
    let output =
        run_with_stdin(r#"{"schemaVersion":1,"root":".","files":[],"policyVersion":"future"}"#)?;

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported policyVersion future"));
    Ok(())
}

#[test]
fn missing_input_file_exits_1_without_stdout() -> anyhow::Result<()> {
    let output = run_with_stdin(
        r#"{"schemaVersion":1,"root":".","files":["definitely-missing.ts"],"policyVersion":"module-edge-scanner-v1"}"#,
    )?;

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("read definitely-missing.ts"));
    Ok(())
}
