use anyhow::Context;

use crate::cli::parse_u64;
use crate::{is_usage_error, usage_error};

#[test]
fn usage_error_is_typed_for_exit_code_classification() {
    let error = usage_error("missing required flag");

    assert!(is_usage_error(&error));
    assert_eq!(error.to_string(), "missing required flag");
}

#[test]
fn usage_error_classification_survives_context_wrapping() {
    let result = Err::<(), _>(usage_error("missing required flag")).context("failed to parse cli");
    let Err(error) = result else {
        panic!("usage error fixture unexpectedly succeeded");
    };

    assert!(is_usage_error(&error));
    assert_eq!(error.to_string(), "failed to parse cli");
}

#[test]
fn usage_error_classification_survives_cli_helper_question_mark() {
    fn parse_cli_value() -> anyhow::Result<()> {
        parse_u64("soon", "--timeout-ms")?;
        Ok(())
    }

    let Err(error) = parse_cli_value() else {
        panic!("cli usage fixture unexpectedly succeeded");
    };

    assert!(is_usage_error(&error));
    assert_eq!(error.to_string(), "invalid --timeout-ms value: soon");
}
