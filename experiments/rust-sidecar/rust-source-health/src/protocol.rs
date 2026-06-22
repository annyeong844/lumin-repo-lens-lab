mod ast;
mod file;
mod location;
mod meta;
mod parse;
mod parser;
mod path;
mod request;
mod response;
mod signal;
mod summary;

pub use ast::{
    AstCfgGate, AstDefinition, AstDefinitionKind, AstFacts, AstMacroCall, AstMethodCall,
    AstOpaqueMuteReason, AstOpaqueReason, AstOpaqueSurface, AstOpaqueSurfaceKind,
    AstOpaqueSurfaceVisibility, AstOpaqueVisibility, AstPathRef, AstUseTree, AstVisibility,
};
pub use file::{Facts, FileHealth};
pub use location::Location;
pub use meta::{
    InputMeta, ParserMeta, PolicyMeta, ResponseMeta, RuntimeMeta, SidecarMeta, SignalPolicyMeta,
    SkippedFile, SkippedFileReason, SourceHealthLimit, SourceHealthMode, SourceHealthProducer,
};
pub use parse::{ParseError, ParseStatus};
pub use parser::{ParserEdition, ParserEditionPolicy, ParserEditionSource, ParserKind};
pub use path::{PathClassification, PathMeta};
pub use request::{HealthRequest, ParserRequest, PathPolicy, RequestFile, RuntimeRequest};
pub use response::HealthResponse;
pub use signal::{
    Claim, Severity, Signal, SignalKind, SignalMuteReason, SignalVisibility, SignalVisibilityState,
};
pub use summary::Summary;

pub const SCHEMA_VERSION: u32 = 1;
pub const POLICY_VERSION: &str = "m6-rust-source-health-syntax-v2";
pub const PARSER_KIND: ParserKind = ParserKind::RaApSyntax;
pub const PARSER_VERSION: &str = "0.0.337";
pub const PARSER_EDITION: ParserEdition = ParserEdition::Edition2021;
pub const PARSER_EDITION_POLICY: ParserEditionPolicy = ParserEditionPolicy::Fixed;
pub const PARSER_EDITION_SOURCE: ParserEditionSource = ParserEditionSource::M6PolicyDefault;
pub const SIGNAL_POLICY_ID: &str = "rust-source-health-signal-policy";
pub const SIGNAL_POLICY_VERSION: &str = "rust-source-health-signal-policy.v2";
pub const DEFAULT_WORKER_STACK_BYTES: usize = 16 * 1024 * 1024;
pub const DEFAULT_INCLUDE: &[&str] = &["**/*.rs"];
pub const DEFAULT_EXCLUDE: &[&str] = &["**/target/**", "**/vendor/**"];
