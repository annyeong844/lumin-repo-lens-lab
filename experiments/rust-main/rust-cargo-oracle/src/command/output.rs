#[derive(Debug, Clone)]
pub(crate) struct CommandOutput {
    pub(crate) status: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) elapsed_ms: u128,
    pub(crate) skip_reason: Option<CommandSkipReason>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum CommandSkipReason {
    TargetedCargoCheckSelectedNoPackages,
}

pub(crate) fn skipped_command_output() -> CommandOutput {
    CommandOutput {
        status: None,
        stdout: String::new(),
        stderr: String::new(),
        elapsed_ms: 0,
        skip_reason: None,
    }
}

pub(crate) fn skipped_command_output_with_reason(reason: CommandSkipReason) -> CommandOutput {
    CommandOutput {
        status: None,
        stdout: String::new(),
        stderr: String::new(),
        elapsed_ms: 0,
        skip_reason: Some(reason),
    }
}

pub(crate) fn combined_command_output(outputs: Vec<CommandOutput>) -> CommandOutput {
    let mut outputs = outputs.into_iter();
    let Some(mut combined) = outputs.next() else {
        return skipped_command_output();
    };

    for output in outputs {
        combined.status = combined_status(combined.status, output.status);
        append_text(&mut combined.stdout, &output.stdout);
        append_text(&mut combined.stderr, &output.stderr);
        combined.elapsed_ms += output.elapsed_ms;
        if combined.skip_reason.is_none() {
            combined.skip_reason = output.skip_reason;
        }
    }

    combined
}

fn combined_status(left: Option<i32>, right: Option<i32>) -> Option<i32> {
    match (left, right) {
        (None, _) | (_, None) => None,
        (Some(0), Some(code)) | (Some(code), Some(0)) => Some(code),
        (Some(code), Some(_)) => Some(code),
    }
}

fn append_text(target: &mut String, text: &str) {
    if text.is_empty() {
        return;
    }
    if !target.is_empty() && !target.ends_with('\n') {
        target.push('\n');
    }
    target.push_str(text);
}

#[cfg(test)]
mod tests {
    use super::{
        combined_command_output, skipped_command_output, skipped_command_output_with_reason,
        CommandOutput, CommandSkipReason,
    };

    #[test]
    fn skipped_command_output_has_no_fake_exit_code_or_elapsed_time() {
        let output = skipped_command_output();

        assert_eq!(output.status, None);
        assert_eq!(output.elapsed_ms, 0);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "");
        assert_eq!(output.skip_reason, None);
    }

    #[test]
    fn skipped_command_output_can_name_the_skip_reason() {
        let output = skipped_command_output_with_reason(
            CommandSkipReason::TargetedCargoCheckSelectedNoPackages,
        );

        assert_eq!(
            output.skip_reason,
            Some(CommandSkipReason::TargetedCargoCheckSelectedNoPackages)
        );
    }

    #[test]
    fn combined_command_output_preserves_completed_stdout_after_later_failure() {
        let output = combined_command_output(vec![
            CommandOutput {
                status: Some(0),
                stdout: "fast-json\n".to_string(),
                stderr: String::new(),
                elapsed_ms: 20,
                skip_reason: None,
            },
            CommandOutput {
                status: Some(1),
                stdout: String::new(),
                stderr: "later failure".to_string(),
                elapsed_ms: 1000,
                skip_reason: None,
            },
        ]);

        assert_eq!(output.status, Some(1));
        assert_eq!(output.stdout, "fast-json\n");
        assert_eq!(output.stderr, "later failure");
        assert_eq!(output.elapsed_ms, 1020);
    }

    #[test]
    fn combined_command_output_keeps_first_nonzero_status() {
        let output = combined_command_output(vec![
            CommandOutput {
                status: Some(101),
                stdout: String::new(),
                stderr: String::new(),
                elapsed_ms: 1,
                skip_reason: None,
            },
            CommandOutput {
                status: Some(1),
                stdout: String::new(),
                stderr: String::new(),
                elapsed_ms: 1,
                skip_reason: None,
            },
        ]);

        assert_eq!(output.status, Some(101));
    }
}
