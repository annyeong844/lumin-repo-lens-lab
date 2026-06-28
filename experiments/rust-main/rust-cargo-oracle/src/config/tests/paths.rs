use super::super::cargo_config_paths;
use crate::environment::CompilationEnvironment;
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

    let paths = cargo_config_paths(&crate_root, &empty_environment());

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

    let paths = cargo_config_paths(&root, &empty_environment());

    assert_eq!(paths[0], root.join(".cargo").join("config"));
    assert!(!paths.contains(&root.join(".cargo").join("config.toml")));
    Ok(())
}

fn empty_environment() -> CompilationEnvironment {
    CompilationEnvironment::from_vars(Vec::<(String, String)>::new())
}
