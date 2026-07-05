use std::path::Path;
use std::process::{Command, Stdio};

pub(crate) fn git_head_commit_or_unknown(root: &Path) -> String {
    let Ok(output) = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
    else {
        return "unknown".to_string();
    };

    if !output.status.success() {
        return "unknown".to_string();
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let commit = text.trim();
    if commit.is_empty() {
        "unknown".to_string()
    } else {
        commit.to_string()
    }
}
