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

#[cfg(test)]
mod tests {
    use super::{skipped_command_output, skipped_command_output_with_reason, CommandSkipReason};

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
}
