use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::environment::CompilationEnvironment;

pub(crate) fn cargo_config_paths(
    root: &Path,
    environment: &CompilationEnvironment,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = BTreeSet::new();
    for base in ancestors(root) {
        push_config_path(&mut paths, &mut seen, &base.join(".cargo"));
    }
    let cargo_home = cargo_home(environment);
    push_config_path(&mut paths, &mut seen, &cargo_home);
    paths
}

fn cargo_home(environment: &CompilationEnvironment) -> PathBuf {
    environment
        .get("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            environment
                .get("USERPROFILE")
                .map(|home| PathBuf::from(home).join(".cargo"))
        })
        .or_else(|| {
            environment
                .get("HOME")
                .map(|home| PathBuf::from(home).join(".cargo"))
        })
        .unwrap_or_else(|| PathBuf::from(".cargo"))
}

fn ancestors(path: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut current = path.to_path_buf();
    loop {
        out.push(current.clone());
        if !current.pop() {
            break;
        }
    }
    out
}

fn push_config_path(out: &mut Vec<PathBuf>, seen: &mut BTreeSet<PathBuf>, config_dir: &Path) {
    let config = config_dir.join("config");
    let config_toml = config_dir.join("config.toml");
    let selected = if config.is_file() {
        Some(config)
    } else if config_toml.is_file() {
        Some(config_toml)
    } else {
        None
    };
    if let Some(path) = selected {
        if seen.insert(path.clone()) {
            out.push(path);
        }
    }
}
