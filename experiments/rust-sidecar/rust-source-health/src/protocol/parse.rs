use serde::{Deserialize, Serialize};

use super::{Claim, Location};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseStatus {
    pub ok: bool,
    pub errors: Vec<ParseError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    pub message: String,
    pub claim: Claim,
    pub location: Location,
}
