mod paths;
mod toml;

pub(crate) use paths::cargo_config_paths;
pub(crate) use toml::{read_build_rustflags_from_config, read_build_target_from_config};

#[cfg(test)]
mod tests;
