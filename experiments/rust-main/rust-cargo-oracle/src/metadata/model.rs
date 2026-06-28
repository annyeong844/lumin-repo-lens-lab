use serde::Deserialize;

use crate::protocol::CargoTargetKind;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CargoMetadata {
    #[serde(default)]
    pub(crate) packages: Vec<CargoPackage>,
    #[serde(default)]
    pub(crate) workspace_members: Vec<String>,
    #[serde(default)]
    pub(crate) workspace_default_members: Vec<String>,
    #[serde(default)]
    pub(crate) workspace_root: Option<String>,
    #[serde(default)]
    pub(crate) target_directory: Option<String>,
    #[serde(default)]
    pub(crate) resolve: Option<CargoResolve>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CargoResolve {
    #[serde(default)]
    pub(crate) root: Option<String>,
    #[serde(default)]
    pub(crate) nodes: Vec<CargoResolveNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CargoResolveNode {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) dependencies: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CargoPackage {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) manifest_path: String,
    #[serde(default)]
    pub(crate) source: Option<String>,
    #[serde(default)]
    pub(crate) targets: Vec<CargoTarget>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CargoTarget {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) kind: Vec<CargoTargetKind>,
    #[serde(default)]
    pub(crate) required_features: Vec<String>,
}

impl CargoTarget {
    pub(crate) fn is_default_checked(&self) -> bool {
        self.required_features.is_empty()
            && self.kind.iter().any(CargoTargetKind::is_default_checked)
    }

    pub(crate) fn blocks_cache_reuse(&self) -> bool {
        self.kind.iter().any(CargoTargetKind::blocks_cache_reuse)
    }
}
