use serde::Serialize;

use super::{Claim, Location};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseStatus {
    pub ok: bool,
    pub errors: Vec<ParseError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParseError {
    pub message: String,
    pub claim: Claim,
    pub location: Location,
}
