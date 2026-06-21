use super::super::support::TestDir;
use crate::{find_repo_root, find_repo_root_with_fallback};
use std::fs;
use std::io;

#[test]
fn find_repo_root_walks_up_to_oracle_registry_owner() -> io::Result<()> {
    let temp = TestDir::new("find-repo-root")?;
    let repo = temp.path().join("repo");
    let nested = repo.join("crate").join("src");
    fs::create_dir_all(repo.join("canonical"))?;
    fs::create_dir_all(&nested)?;
    fs::write(repo.join("canonical").join("oracle-registry.json"), "{}")?;

    assert_eq!(find_repo_root(&nested), Some(repo));
    Ok(())
}

#[test]
fn find_repo_root_returns_none_without_registry_marker() -> io::Result<()> {
    let temp = TestDir::new("missing-repo-root")?;
    let nested = temp.path().join("repo").join("crate");
    fs::create_dir_all(&nested)?;

    assert_eq!(find_repo_root(&nested), None);
    Ok(())
}

#[test]
fn find_repo_root_falls_back_to_tool_registry_owner() -> io::Result<()> {
    let temp = TestDir::new("repo-root-fallback")?;
    let analyzed = temp.path().join("external").join("crate");
    let tool_repo = temp.path().join("tool-repo");
    let tool_nested = tool_repo.join("experiments").join("rust-main");
    fs::create_dir_all(&analyzed)?;
    fs::create_dir_all(tool_repo.join("canonical"))?;
    fs::create_dir_all(&tool_nested)?;
    fs::write(
        tool_repo.join("canonical").join("oracle-registry.json"),
        "{}",
    )?;

    assert_eq!(
        find_repo_root_with_fallback(&analyzed, &tool_nested),
        Some(tool_repo)
    );
    Ok(())
}
