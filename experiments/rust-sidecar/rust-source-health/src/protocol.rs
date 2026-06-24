mod ast;
mod file;
mod function_clones;
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
    AstCallableKind, AstCfgGate, AstDefinition, AstDefinitionKind, AstFacts,
    AstFunctionBodyFingerprint, AstFunctionBodyFingerprintKind, AstFunctionOwner, AstFunctionParam,
    AstFunctionReceiver, AstFunctionReceiverKind, AstFunctionSignature, AstFunctionSignatureKind,
    AstImplBlock, AstImplMethod, AstInlinePattern, AstInlinePatternKind, AstMacroCall,
    AstMethodCall, AstOpaqueMuteReason, AstOpaqueReason, AstOpaqueSurface, AstOpaqueSurfaceKind,
    AstOpaqueSurfaceVisibility, AstOpaqueVisibility, AstPathRef, AstShapeConfidence, AstShapeField,
    AstShapeFieldKind, AstShapeHash, AstShapeHashKind, AstShapeKind, AstUseTree, AstVisibility,
};
pub use file::{Facts, FileHealth};
pub use function_clones::{
    AstFunctionCloneGroup, AstFunctionCloneGroupKind, AstFunctionCloneGroups,
    AstFunctionCloneGroupsPolicy, AstFunctionCloneLine, AstNearFunctionCandidate,
    AstNearFunctionCandidateKind, AstNearFunctionCandidatePolicy, AstNearFunctionCandidateWeights,
    FunctionCloneRisk,
};
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
pub const POLICY_VERSION: &str = "m6-rust-source-health-syntax-v9";
pub const RUST_SHAPE_HASH_NORMALIZED_VERSION: &str = "rust-shape-hash.normalized.v1";
pub const RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION: &str =
    "rust-function-signature.normalized.v1";
pub const RUST_FUNCTION_BODY_NORMALIZED_VERSION: &str = "rust-function-body.normalized.v2";
pub const RUST_FUNCTION_CLONE_GROUP_POLICY_ID: &str = "rust-function-clone-group-policy";
pub const RUST_FUNCTION_CLONE_GROUP_POLICY_VERSION: &str = "rust-function-clone-group-policy.v2";
pub const RUST_FUNCTION_CLONE_MIN_GROUP_SIZE: usize = 2;
pub const RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC: usize = 1;
pub const RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS: usize = 1;
pub const RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC: usize = 3;
pub const RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS: usize = 2;
pub const RUST_FUNCTION_CLONE_NEAR_POLICY_ID: &str = "function-clone-near-policy";
pub const RUST_FUNCTION_CLONE_NEAR_POLICY_VERSION: &str = "function-clone-near-policy-v1";
pub const RUST_FUNCTION_CLONE_NEAR_POLICY_CLASS: &str = "review";
pub const RUST_FUNCTION_CLONE_NEAR_CALIBRATION_VERSION: &str =
    "rust-function-clone-near-calibration.v1";
pub const RUST_FUNCTION_CLONE_NEAR_MIN_SIGNIFICANT_CALL_TOKEN_LEN: usize = 4;
pub const RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA: usize = 1;
pub const RUST_FUNCTION_CLONE_NEAR_MIN_BODY_LOC_SIMILARITY: f64 = 0.34;
pub const RUST_FUNCTION_CLONE_NEAR_MIN_STATEMENT_COUNT_SIMILARITY: f64 = 0.34;
pub const RUST_FUNCTION_CLONE_NEAR_MIN_CALL_TOKEN_JACCARD: f64 = 0.5;
pub const RUST_FUNCTION_CLONE_NEAR_MIN_NAME_TOKEN_JACCARD_FALLBACK: f64 = 0.34;
pub const RUST_FUNCTION_CLONE_NEAR_MIN_SCORE: f64 = 0.62;
pub const RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES: usize = 50;
pub const RUST_FUNCTION_CLONE_NEAR_CALL_TOKEN_WEIGHT: f64 = 0.45;
pub const RUST_FUNCTION_CLONE_NEAR_NAME_TOKEN_WEIGHT: f64 = 0.25;
pub const RUST_FUNCTION_CLONE_NEAR_BODY_LOC_WEIGHT: f64 = 0.15;
pub const RUST_FUNCTION_CLONE_NEAR_STATEMENT_COUNT_WEIGHT: f64 = 0.15;
pub const RUST_FUNCTION_CLONE_NEAR_REQUIRED_MATCHING_QUALIFIERS: &[&str] =
    &["async", "unsafe", "const"];
pub const RUST_FUNCTION_CLONE_NEAR_SUPPRESSED_GENERIC_CALL_TOKENS: &[&str] = &[
    "as_mut",
    "as_ref",
    "borrow",
    "borrow_mut",
    "clone",
    "cloned",
    "collect",
    "copied",
    "count",
    "err",
    "expect",
    "filter",
    "find",
    "flat_map",
    "for_each",
    "get",
    "insert",
    "into_iter",
    "is_empty",
    "is_none",
    "is_some",
    "iter",
    "iter_mut",
    "join",
    "map",
    "ok",
    "push",
    "split",
    "then",
    "to_owned",
    "to_string",
    "trim",
    "unwrap",
    "unwrap_or",
    "unwrap_or_default",
];
pub const RUST_INLINE_PATTERN_NORMALIZED_VERSION: &str = "rust-inline-statement-normalizer-v1";
pub const RUST_INLINE_PATTERN_MAX_STATEMENTS: usize = 2;
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
