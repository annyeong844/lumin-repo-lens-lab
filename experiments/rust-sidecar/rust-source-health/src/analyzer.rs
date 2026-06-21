use crate::protocol::{
    FileHealth, ParserEdition, ParserRequest, RequestFile, Thresholds, PARSER_EDITION,
    PARSER_EDITION_POLICY, PARSER_EDITION_SOURCE,
};
use crate::usage_error;
use anyhow::{bail, Result};
use ra_ap_syntax::Edition;
use rayon::prelude::*;
use std::collections::BTreeMap;

mod attrs;
mod facts;
mod file;
mod location;
mod opaque;
mod path;
mod signal_policy;
mod syntax;

use file::analyze_file;

pub(crate) fn analyze_files(
    files: &[RequestFile],
    thresholds: &Thresholds,
    parser: &ParserRequest,
) -> Result<BTreeMap<String, FileHealth>> {
    let edition = parser_edition(parser)?;
    let analyzed = files
        .par_iter()
        .map(|file| analyze_file(file, thresholds, edition))
        .collect::<Vec<_>>();

    let mut out = BTreeMap::new();
    for result in analyzed {
        let (path, health) = result?;
        out.insert(path, health);
    }
    Ok(out)
}

fn parser_edition(parser: &ParserRequest) -> Result<Edition> {
    if parser.edition_policy != PARSER_EDITION_POLICY
        || parser.edition != PARSER_EDITION
        || parser.edition_source != PARSER_EDITION_SOURCE
    {
        return Err(usage_error("unsupported parser edition policy"));
    }
    configured_parser_edition()
}

fn configured_parser_edition() -> Result<Edition> {
    match PARSER_EDITION {
        ParserEdition::Edition2021 => Ok(Edition::Edition2021),
        ParserEdition::Unsupported => bail!(
            "unsupported configured parser edition {value}",
            value = PARSER_EDITION.as_str()
        ),
    }
}
