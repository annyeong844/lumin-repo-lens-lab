use crate::function_clones::FunctionCloneFile;
use crate::protocol::{
    AstDefinition, AstDefinitionAttribute, AstDefinitionAttributeKind, AstDefinitionKind,
    AstDefinitionOwner, AstImplBlock, AstImplMethod, AstOpaqueSurface, AstOpaqueVisibility,
    AstVisibility, CompactFileHealth, FileHealth, FileSignalSummary, Location, ParserEdition,
    ParserRequest, PathClassification, RequestFile, PARSER_EDITION, PARSER_EDITION_POLICY,
    PARSER_EDITION_SOURCE,
};
use crate::usage_error;
use anyhow::{bail, Context, Result};
use ra_ap_syntax::Edition;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

mod attrs;
mod facts;
mod file;
mod location;
mod opaque;
mod path;
mod signal_policy;
mod syntax;

use file::{analyze_file, analyze_file_text};

#[derive(Debug, Clone)]
pub(crate) struct SourceFileEntry {
    pub(crate) path: String,
    pub(crate) absolute_path: PathBuf,
    pub(crate) sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CompactDeadFile {
    pub(crate) parse_ok: bool,
    pub(crate) path_classifications: Vec<PathClassification>,
    pub(crate) definitions: Vec<CompactDeadDefinition>,
    pub(crate) impls: Vec<CompactDeadImplBlock>,
    pub(crate) local_ref_names: Vec<Box<str>>,
    pub(crate) test_local_ref_names: Vec<Box<str>>,
    pub(crate) review_opaque_surface: Option<AstOpaqueSurface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CompactDeadDefinition {
    pub(crate) kind: AstDefinitionKind,
    pub(crate) name: Box<str>,
    pub(crate) visibility: AstVisibility,
    pub(crate) owner: AstDefinitionOwner,
    pub(crate) test_context: bool,
    pub(crate) attributes: Vec<CompactDeadDefinitionAttribute>,
    pub(crate) location: Location,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CompactDeadDefinitionAttribute {
    pub(crate) kind: AstDefinitionAttributeKind,
    pub(crate) text: Box<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CompactDeadImplBlock {
    #[serde(rename = "trait", skip_serializing_if = "Option::is_none")]
    pub(crate) trait_path: Option<Box<str>>,
    pub(crate) methods: Vec<CompactDeadImplMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CompactDeadImplMethod {
    pub(crate) name: Box<str>,
    pub(crate) visibility: AstVisibility,
    pub(crate) location: Location,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CompactSummaryFile {
    pub(crate) file: CompactFileHealth,
    pub(crate) signal_summary: FileSignalSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CompactFileAnalysis {
    pub(crate) summary_file: CompactSummaryFile,
    pub(crate) dead_file: CompactDeadFile,
    pub(crate) clone_file: FunctionCloneFile,
}

pub(crate) fn analyze_files(
    files: &[RequestFile],
    parser: &ParserRequest,
    retain_raw_name_refs: bool,
    retain_raw_signals: bool,
    retain_raw_ast_lanes: bool,
) -> Result<BTreeMap<String, FileHealth>> {
    let edition = parser_edition(parser)?;
    files
        .par_iter()
        .map(|file| {
            analyze_file(
                file,
                edition,
                retain_raw_name_refs,
                retain_raw_signals,
                retain_raw_ast_lanes,
            )
        })
        .collect::<Result<BTreeMap<_, _>>>()
}

pub(crate) fn analyze_source_file_entries(
    files: &[SourceFileEntry],
    parser: &ParserRequest,
    retain_raw_name_refs: bool,
    retain_raw_signals: bool,
    retain_raw_ast_lanes: bool,
) -> Result<BTreeMap<String, FileHealth>> {
    let edition = parser_edition(parser)?;
    files
        .par_iter()
        .map(|file| {
            let text = std::fs::read_to_string(&file.absolute_path).with_context(|| {
                format!(
                    "failed to read Rust source {}",
                    file.absolute_path.display()
                )
            })?;
            analyze_file_text(
                &file.path,
                &file.sha256,
                &text,
                edition,
                retain_raw_name_refs,
                retain_raw_signals,
                retain_raw_ast_lanes,
            )
        })
        .collect::<Result<BTreeMap<_, _>>>()
}

pub(crate) fn analyze_source_file_entries_compact(
    files: &[SourceFileEntry],
    parser: &ParserRequest,
) -> Result<BTreeMap<String, CompactFileAnalysis>> {
    let edition = parser_edition(parser)?;
    files
        .par_iter()
        .map(|file| {
            let text = std::fs::read_to_string(&file.absolute_path).with_context(|| {
                format!(
                    "failed to read Rust source {}",
                    file.absolute_path.display()
                )
            })?;
            let (path, health) = analyze_file_text(
                &file.path,
                &file.sha256,
                &text,
                edition,
                false,
                false,
                false,
            )?;
            Ok((path, compact_file_analysis_from_health(health)))
        })
        .collect::<Result<BTreeMap<_, _>>>()
}

pub(crate) fn compact_file_analysis_from_health(health: FileHealth) -> CompactFileAnalysis {
    let summary_file = CompactSummaryFile::from_health(&health);
    let dead_file = compact_dead_file_from_health(&health);
    let clone_file = FunctionCloneFile::from_health(health);
    CompactFileAnalysis {
        summary_file,
        dead_file,
        clone_file,
    }
}

fn compact_dead_file_from_health(health: &FileHealth) -> CompactDeadFile {
    CompactDeadFile {
        parse_ok: health.parse.ok,
        path_classifications: health.path.classifications.clone(),
        definitions: health
            .ast
            .definitions
            .iter()
            .cloned()
            .map(CompactDeadDefinition::from)
            .collect(),
        impls: health
            .ast
            .impls
            .iter()
            .cloned()
            .map(CompactDeadImplBlock::from)
            .collect(),
        local_ref_names: compact_ref_names(&health.ast.local_ref_names),
        test_local_ref_names: compact_ref_names(&health.ast.test_local_ref_names),
        review_opaque_surface: health
            .ast
            .opaque_surfaces
            .iter()
            .find(|surface| surface.visibility.visibility() == AstOpaqueVisibility::Review)
            .cloned(),
    }
}

impl CompactSummaryFile {
    fn from_health(health: &FileHealth) -> Self {
        Self {
            file: CompactFileHealth::from_file(health),
            signal_summary: health.signal_summary.clone(),
        }
    }
}

fn compact_ref_names(names: &BTreeSet<String>) -> Vec<Box<str>> {
    names
        .iter()
        .map(|name| name.clone().into_boxed_str())
        .collect()
}

impl From<AstDefinition> for CompactDeadDefinition {
    fn from(definition: AstDefinition) -> Self {
        Self {
            kind: definition.kind,
            name: definition.name.into_boxed_str(),
            visibility: definition.visibility,
            owner: definition.owner,
            test_context: definition.test_context,
            attributes: definition
                .attributes
                .into_iter()
                .map(CompactDeadDefinitionAttribute::from)
                .collect(),
            location: definition.location,
        }
    }
}

impl From<AstDefinitionAttribute> for CompactDeadDefinitionAttribute {
    fn from(attribute: AstDefinitionAttribute) -> Self {
        Self {
            kind: attribute.kind,
            text: attribute.text.into_boxed_str(),
        }
    }
}

impl From<AstImplBlock> for CompactDeadImplBlock {
    fn from(impl_block: AstImplBlock) -> Self {
        Self {
            trait_path: impl_block.trait_path.map(String::into_boxed_str),
            methods: impl_block
                .methods
                .into_iter()
                .map(CompactDeadImplMethod::from)
                .collect(),
        }
    }
}

impl From<AstImplMethod> for CompactDeadImplMethod {
    fn from(method: AstImplMethod) -> Self {
        Self {
            name: method.name.into_boxed_str(),
            visibility: method.visibility,
            location: method.location,
        }
    }
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
