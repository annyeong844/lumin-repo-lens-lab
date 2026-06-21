use std::path::PathBuf;
use std::str::FromStr;

use crate::UsageError;

pub type CliResult<T> = Result<T, UsageError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliAction<T> {
    Run(T),
    Help,
}

pub fn take_string(args: &mut impl Iterator<Item = String>, flag: &str) -> CliResult<String> {
    args.next()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| missing_value(flag))
}

pub fn take_path(args: &mut impl Iterator<Item = String>, flag: &str) -> CliResult<PathBuf> {
    Ok(PathBuf::from(take_string(args, flag)?))
}

pub fn parse_nonzero_usize(value: &str, flag: &str) -> CliResult<usize> {
    let parsed = parse_usize(value, flag)?;
    if parsed == 0 {
        return Err(UsageError::new(format!("{flag} must be greater than zero")));
    }
    Ok(parsed)
}

pub fn parse_min_usize(value: &str, flag: &str, minimum: usize) -> CliResult<usize> {
    let parsed = parse_usize(value, flag)?;
    if parsed < minimum {
        return Err(UsageError::new(format!(
            "{flag} must be at least {minimum}"
        )));
    }
    Ok(parsed)
}

pub fn parse_enum<T>(value: &str, flag: &str) -> CliResult<T>
where
    T: FromStr,
{
    value
        .parse::<T>()
        .map_err(|_| UsageError::new(format!("invalid {flag} value: {value}")))
}

fn parse_usize(value: &str, flag: &str) -> CliResult<usize> {
    value
        .parse::<usize>()
        .map_err(|_| invalid_numeric_value(flag, value))
}

fn missing_value(flag: &str) -> UsageError {
    UsageError::new(format!("{flag} requires a value"))
}

fn invalid_numeric_value(flag: &str, value: &str) -> UsageError {
    UsageError::new(format!("invalid {flag} value: {value}"))
}

#[cfg(test)]
mod tests {
    use super::{parse_min_usize, parse_nonzero_usize, take_path, take_string, UsageError};
    use std::path::PathBuf;

    #[test]
    fn take_string_reads_next_non_empty_value() {
        let mut args = vec!["value".to_string()].into_iter();

        assert_eq!(take_string(&mut args, "--flag"), Ok("value".to_string()));
    }

    #[test]
    fn take_string_rejects_missing_or_blank_values() {
        let mut missing = Vec::<String>::new().into_iter();
        let mut blank = vec!["  ".to_string()].into_iter();

        assert_eq!(
            take_string(&mut missing, "--flag"),
            Err(UsageError::new("--flag requires a value"))
        );
        assert_eq!(
            take_string(&mut blank, "--flag"),
            Err(UsageError::new("--flag requires a value"))
        );
    }

    #[test]
    fn take_path_reads_next_value_as_path() {
        let mut args = vec!["repo".to_string()].into_iter();

        assert_eq!(take_path(&mut args, "--root"), Ok(PathBuf::from("repo")));
    }

    #[test]
    fn parse_nonzero_usize_keeps_cli_usage_messages() {
        assert_eq!(parse_nonzero_usize("4", "--threads"), Ok(4));
        assert_eq!(
            parse_nonzero_usize("nope", "--threads"),
            Err(UsageError::new("invalid --threads value: nope"))
        );
        assert_eq!(
            parse_nonzero_usize("0", "--threads"),
            Err(UsageError::new("--threads must be greater than zero"))
        );
    }

    #[test]
    fn parse_min_usize_keeps_cli_usage_messages() {
        assert_eq!(
            parse_min_usize("2048", "--worker-stack-bytes", 1024),
            Ok(2048)
        );
        assert_eq!(
            parse_min_usize("nope", "--worker-stack-bytes", 1024),
            Err(UsageError::new("invalid --worker-stack-bytes value: nope"))
        );
        assert_eq!(
            parse_min_usize("512", "--worker-stack-bytes", 1024),
            Err(UsageError::new(
                "--worker-stack-bytes must be at least 1024"
            ))
        );
    }
}
