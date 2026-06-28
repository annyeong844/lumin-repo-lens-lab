use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub enum ParserKind {
    #[serde(rename = "ra_ap_syntax")]
    RaApSyntax,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParserEditionPolicy {
    Fixed,
    #[serde(other)]
    Unsupported,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum ParserEdition {
    #[serde(rename = "2021")]
    Edition2021,
    #[serde(other)]
    Unsupported,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParserEditionSource {
    M6PolicyDefault,
    #[serde(other)]
    Unsupported,
}

impl ParserEdition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Edition2021 => "2021",
            Self::Unsupported => "unsupported",
        }
    }
}
