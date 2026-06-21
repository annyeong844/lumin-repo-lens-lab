use anyhow::{bail, Context, Result};
use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::environment::TARGET_DIRECTORY_ENV_KEYS;

use super::output::CommandOutput;

static TEMP_OUTPUT_COUNTER: AtomicU64 = AtomicU64::new(0);
const COMPACT_TARGET_ENV: &[(&str, &str)] = &[
    ("CARGO_INCREMENTAL", "0"),
    ("CARGO_BUILD_INCREMENTAL", "false"),
    ("CARGO_PROFILE_DEV_DEBUG", "0"),
    ("CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG", "0"),
    ("CARGO_PROFILE_TEST_DEBUG", "0"),
    ("CARGO_PROFILE_TEST_BUILD_OVERRIDE_DEBUG", "0"),
];

pub(crate) fn run_command(
    command: &str,
    args: &[String],
    cwd: &Path,
    timeout_ms: u64,
    cargo_target_dir: Option<&Path>,
) -> Result<CommandOutput> {
    let started = Instant::now();
    let (stdout_capture, stdout_file) = TempOutputCapture::create("stdout")?;
    let (stderr_capture, stderr_file) = TempOutputCapture::create("stderr")?;
    let mut child_command = Command::new(command);
    child_command
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));
    for key in TARGET_DIRECTORY_ENV_KEYS {
        child_command.env_remove(key);
    }
    if let Some(cargo_target_dir) = cargo_target_dir {
        child_command.env("CARGO_TARGET_DIR", cargo_target_dir);
        apply_compact_target_env(&mut child_command);
    }
    let child_result = child_command.spawn();
    let mut child = match child_result {
        Ok(child) => child,
        Err(error) => {
            return Err(error).with_context(|| format!("failed to spawn {command}"));
        }
    };
    let mut timed_out = false;
    let status: ExitStatus;

    loop {
        if let Some(exit_status) = child.try_wait()? {
            status = exit_status;
            break;
        }
        if started.elapsed() >= Duration::from_millis(timeout_ms) {
            timed_out = true;
            kill_process_tree(&mut child);
            status = child.wait()?;
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    Ok(CommandOutput {
        status: status.code(),
        stdout: stdout_capture.read("stdout")?,
        stderr: stderr_capture.read("stderr")?,
        timed_out,
        elapsed_ms: started.elapsed().as_millis(),
        skip_reason: None,
    })
}

fn kill_process_tree(child: &mut Child) {
    #[cfg(windows)]
    {
        let pid = child.id().to_string();
        let _ = Command::new("taskkill")
            .args(["/PID", &pid, "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

struct TempOutputCapture {
    path: PathBuf,
}

impl TempOutputCapture {
    fn create(kind: &str) -> Result<(Self, fs::File)> {
        let temp_dir = std::env::temp_dir();
        let process_id = std::process::id();
        let started_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);

        for _ in 0..64 {
            let sequence = TEMP_OUTPUT_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = temp_dir.join(format!(
                "lumin-rust-cargo-oracle-{kind}-{process_id}-{started_nanos}-{sequence}.log"
            ));
            match OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(writer) => return Ok((Self { path }, writer)),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(error).with_context(|| format!("failed to create {kind} capture"));
                }
            }
        }

        bail!(
            "failed to create unique {kind} capture in {}",
            temp_dir.display()
        )
    }

    fn read(&self, name: &str) -> Result<String> {
        let bytes = fs::read(&self.path)
            .with_context(|| format!("{name} capture read failed: {}", self.path.display()))?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

fn apply_compact_target_env(command: &mut Command) {
    for (key, value) in COMPACT_TARGET_ENV {
        command.env(key, value);
    }
}

impl Drop for TempOutputCapture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::run_command;
    use anyhow::Result;
    use tempfile::TempDir;

    #[test]
    fn owned_cargo_target_dir_disables_debug_symbols_and_incremental_builds() -> Result<()> {
        let temp = TempDir::new()?;
        let (command, args) = environment_dump_command();

        let output = run_command(&command, &args, temp.path(), 10_000, Some(temp.path()))?;

        assert_eq!(output.status, Some(0));
        assert_env_line(&output.stdout, "CARGO_INCREMENTAL", "0");
        assert_env_line(&output.stdout, "CARGO_BUILD_INCREMENTAL", "false");
        assert_env_line(&output.stdout, "CARGO_PROFILE_DEV_DEBUG", "0");
        assert_env_line(
            &output.stdout,
            "CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG",
            "0",
        );
        assert_env_line(&output.stdout, "CARGO_PROFILE_TEST_DEBUG", "0");
        assert_env_line(
            &output.stdout,
            "CARGO_PROFILE_TEST_BUILD_OVERRIDE_DEBUG",
            "0",
        );
        Ok(())
    }

    #[cfg(windows)]
    fn environment_dump_command() -> (String, Vec<String>) {
        ("cmd".to_string(), vec!["/C".to_string(), "set".to_string()])
    }

    #[cfg(not(windows))]
    fn environment_dump_command() -> (String, Vec<String>) {
        ("env".to_string(), Vec::new())
    }

    fn assert_env_line(stdout: &str, key: &str, value: &str) {
        let expected = format!("{key}={value}");
        assert!(
            stdout.lines().any(|line| line == expected),
            "missing environment line {expected:?} in:\n{stdout}"
        );
    }
}
