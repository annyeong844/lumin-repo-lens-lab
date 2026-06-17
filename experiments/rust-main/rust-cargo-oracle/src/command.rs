use anyhow::{anyhow, bail, Context, Result};
use std::io::Read;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::metadata::CargoMetadata;
use crate::OracleOptions;

#[derive(Debug, Clone)]
pub(crate) struct CommandOutput {
    pub(crate) status: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) timed_out: bool,
    pub(crate) elapsed_ms: u128,
}

pub(crate) fn cargo_check_args(features: Option<&str>, package_name: Option<&str>) -> Vec<String> {
    let mut args = vec!["check".to_string(), "--message-format=json".to_string()];
    if let Some(package_name) = package_name {
        args.push("--package".to_string());
        args.push(package_name.to_string());
    }
    if let Some(features) = features {
        args.push("--features".to_string());
        args.push(features.to_string());
    }
    args
}

fn cargo_metadata_args(features: Option<&str>) -> Vec<String> {
    let mut args = vec!["metadata".to_string(), "--format-version=1".to_string()];
    if let Some(features) = features {
        args.push("--features".to_string());
        args.push(features.to_string());
    }
    args
}

pub(crate) fn run_cargo_check(root: &Path, options: &OracleOptions) -> Result<CommandOutput> {
    run_command(
        &options.cargo_bin,
        &cargo_check_args(options.features.as_deref(), options.package_name.as_deref()),
        root,
        options.timeout_ms,
    )
}

pub(crate) fn run_cargo_metadata(
    root: &Path,
    cargo_bin: &str,
    timeout_ms: u64,
    features: Option<&str>,
) -> Result<CargoMetadata> {
    let output = run_command(cargo_bin, &cargo_metadata_args(features), root, timeout_ms)?;
    if output.timed_out {
        bail!("cargo metadata timed out");
    }
    if output.status != Some(0) {
        bail!("cargo metadata failed: {}", output.stderr.trim());
    }
    serde_json::from_str(&output.stdout).context("invalid cargo metadata JSON")
}

pub(crate) fn run_command(
    command: &str,
    args: &[String],
    cwd: &Path,
    timeout_ms: u64,
) -> Result<CommandOutput> {
    let started = Instant::now();
    let mut child = Command::new(command)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn {}", command))?;
    let stdout = child
        .stdout
        .take()
        .context("failed to capture stdout pipe")?;
    let stderr = child
        .stderr
        .take()
        .context("failed to capture stderr pipe")?;
    let stdout_reader = read_pipe(stdout);
    let stderr_reader = read_pipe(stderr);
    let mut timed_out = false;
    let status: ExitStatus;

    loop {
        if let Some(exit_status) = child.try_wait()? {
            status = exit_status;
            break;
        }
        if started.elapsed() >= Duration::from_millis(timeout_ms) {
            timed_out = true;
            let _ = child.kill();
            status = child.wait()?;
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    Ok(CommandOutput {
        status: status.code(),
        stdout: join_pipe(stdout_reader, "stdout")?,
        stderr: join_pipe(stderr_reader, "stderr")?,
        timed_out,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

fn read_pipe<R: Read + Send + 'static>(mut pipe: R) -> JoinHandle<std::io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut out = Vec::new();
        pipe.read_to_end(&mut out)?;
        Ok(out)
    })
}

fn join_pipe(handle: JoinHandle<std::io::Result<Vec<u8>>>, name: &str) -> Result<String> {
    let bytes = handle
        .join()
        .map_err(|_| anyhow!("{name} reader thread panicked"))?
        .with_context(|| format!("{name} reader failed"))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::cargo_metadata_args;

    #[test]
    fn cargo_metadata_args_follow_check_feature_selection() {
        assert_eq!(
            cargo_metadata_args(Some("bad,extra")),
            vec![
                "metadata".to_string(),
                "--format-version=1".to_string(),
                "--features".to_string(),
                "bad,extra".to_string(),
            ]
        );
    }

    #[test]
    fn cargo_metadata_args_omit_features_when_unselected() {
        assert_eq!(
            cargo_metadata_args(None),
            vec!["metadata".to_string(), "--format-version=1".to_string()]
        );
    }
}
