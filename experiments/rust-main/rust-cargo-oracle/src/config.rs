use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::metadata::CargoMetadata;

pub(crate) fn cargo_config_paths(root: &Path, _metadata: Option<&CargoMetadata>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = BTreeSet::new();
    for base in ancestors(root) {
        push_config_path(&mut paths, &mut seen, &base.join(".cargo"));
    }
    let cargo_home = std::env::var("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("USERPROFILE")
                .map(|home| PathBuf::from(home).join(".cargo"))
                .or_else(|_| std::env::var("HOME").map(|home| PathBuf::from(home).join(".cargo")))
                .unwrap_or_else(|_| PathBuf::from(".cargo"))
        });
    push_config_path(&mut paths, &mut seen, &cargo_home);
    paths
}

pub(crate) fn read_build_target_from_config(path: &Path) -> Option<Vec<String>> {
    let text = fs::read_to_string(path).ok()?;
    read_build_key(&text, "target")
}

pub(crate) fn read_build_rustflags_from_config(path: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    read_build_key(&text, "rustflags").unwrap_or_default()
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

fn read_build_key(text: &str, key: &str) -> Option<Vec<String>> {
    let mut in_build = false;
    for raw_line in text.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_build = line.trim_matches(&['[', ']'][..]).trim() == "build";
            continue;
        }
        if !in_build {
            continue;
        }
        let Some(rest) = line.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(value) = rest.strip_prefix('=') else {
            continue;
        };
        let value = value.trim();
        if let Some(value) = quoted(value) {
            return Some(vec![value]);
        }
        if value.starts_with('[') && value.ends_with(']') {
            let values: Vec<String> = value
                .trim_matches(&['[', ']'][..])
                .split(',')
                .filter_map(|part| quoted(part.trim()))
                .collect();
            return Some(values);
        }
    }
    None
}

fn quoted(value: &str) -> Option<String> {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        Some(value[1..value.len().saturating_sub(1)].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::cargo_config_paths;
    use anyhow::Result;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn cargo_config_paths_prefer_deeper_project_config_over_parent() -> Result<()> {
        let temp = TempDir::new()?;
        let repo = temp.path().join("repo");
        let crate_root = repo.join("crate");
        fs::create_dir_all(repo.join(".cargo"))?;
        fs::create_dir_all(crate_root.join(".cargo"))?;
        fs::write(repo.join(".cargo").join("config.toml"), "[build]\n")?;
        fs::write(crate_root.join(".cargo").join("config.toml"), "[build]\n")?;

        let paths = cargo_config_paths(&crate_root, None);

        assert_eq!(paths[0], crate_root.join(".cargo").join("config.toml"));
        assert_eq!(paths[1], repo.join(".cargo").join("config.toml"));
        Ok(())
    }

    #[test]
    fn cargo_config_paths_prefer_extensionless_config_in_same_directory() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path().join("crate");
        fs::create_dir_all(root.join(".cargo"))?;
        fs::write(root.join(".cargo").join("config"), "[build]\n")?;
        fs::write(root.join(".cargo").join("config.toml"), "[build]\n")?;

        let paths = cargo_config_paths(&root, None);

        assert_eq!(paths[0], root.join(".cargo").join("config"));
        assert!(!paths.contains(&root.join(".cargo").join("config.toml")));
        Ok(())
    }

    #[test]
    fn malformed_quoted_build_value_is_ignored_without_panic() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path().join("crate");
        fs::create_dir_all(root.join(".cargo"))?;
        let config = root.join(".cargo").join("config.toml");
        fs::write(&config, "[build]\ntarget = \"\n")?;

        assert!(super::read_build_target_from_config(&config).is_none());
        Ok(())
    }
}
