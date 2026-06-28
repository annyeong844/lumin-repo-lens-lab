use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PathMeta {
    pub classifications: Vec<PathClassification>,
    pub suppressed: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PathClassification {
    Generated,
    Source,
    Test,
}
