use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::support::command::unified_analyzer_command;
use crate::support::paths::repo_root;

pub fn unified_analyzer_command_for(root: &Path, output_path: &Path) -> Result<Command> {
    let repo_root = repo_root()?;
    let mut command = unified_analyzer_command();
    command
        .arg("--root")
        .arg(root)
        .arg("--output")
        .arg(output_path)
        .arg("--source-commit")
        .arg("test-source-commit")
        .arg("--repo-root")
        .arg(repo_root);
    Ok(command)
}
