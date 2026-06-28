use crate::{assertions, cli};

pub fn assert_usage_error(args: &[&str]) {
    let args = args
        .iter()
        .map(|arg| (*arg).to_string())
        .collect::<Vec<_>>();
    let output = cli::run_cli(&args);
    assertions::assert_exit_code(&output, 2);
    assert!(output.stdout.is_empty());
}
