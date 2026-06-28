use std::collections::BTreeMap;

const EXACT_KEYS: &[&str] = &[
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_BUILD_RUSTFLAGS",
    "RUSTC",
    "RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC",
    "CARGO_BUILD_RUSTC_WRAPPER",
    "CARGO_BUILD_TARGET",
    "CARGO_HOME",
    "HOME",
    "USERPROFILE",
    "CARGO_INCREMENTAL",
    "CARGO_BUILD_INCREMENTAL",
];

pub(crate) const TARGET_DIRECTORY_ENV_KEYS: &[&str] = &[
    "CARGO_BUILD_TARGET_DIR",
    "CARGO_TARGET_DIR",
    "CARGO_BUILD_BUILD_DIR",
];

#[derive(Debug, Clone, Default)]
pub(crate) struct CompilationEnvironment {
    values: BTreeMap<String, String>,
}

impl CompilationEnvironment {
    pub(crate) fn from_process() -> Self {
        Self::from_vars(std::env::vars())
    }

    pub(crate) fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub(crate) fn values(&self) -> &BTreeMap<String, String> {
        &self.values
    }

    pub(crate) fn from_vars(
        vars: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        let mut values = BTreeMap::new();
        for (key, value) in vars {
            let key = key.into();
            if is_compilation_environment_key(&key) {
                values.insert(key, value.into());
            }
        }
        Self { values }
    }
}

fn is_compilation_environment_key(key: &str) -> bool {
    if TARGET_DIRECTORY_ENV_KEYS.contains(&key) {
        return false;
    }
    EXACT_KEYS.contains(&key)
        || key.starts_with("CARGO_TARGET_")
        || key.starts_with("CARGO_PROFILE_")
}
