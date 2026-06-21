use std::path::Path;

mod cfg_set;
mod target_triple;

pub(super) use cfg_set::resolve_cfg_set;
pub(super) use target_triple::resolve_target_triple;

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests;
