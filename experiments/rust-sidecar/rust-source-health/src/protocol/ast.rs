use serde::Serialize;
use std::collections::BTreeMap;

mod definitions;
mod functions;
mod impls;
mod inline_patterns;
mod opaque;
mod refs;
mod shapes;

pub use definitions::{AstDefinition, AstDefinitionKind, AstVisibility};
pub use functions::{
    AstCallableKind, AstFunctionOwner, AstFunctionParam, AstFunctionReceiver,
    AstFunctionReceiverKind, AstFunctionSignature, AstFunctionSignatureKind,
};
pub use impls::{AstImplBlock, AstImplMethod};
pub use inline_patterns::{AstInlinePattern, AstInlinePatternKind};
pub use opaque::{
    AstCfgGate, AstMacroCall, AstOpaqueMuteReason, AstOpaqueReason, AstOpaqueSurface,
    AstOpaqueSurfaceKind, AstOpaqueSurfaceVisibility, AstOpaqueVisibility,
};
pub use refs::{AstMethodCall, AstPathRef, AstUseTree};
pub use shapes::{
    AstShapeConfidence, AstShapeField, AstShapeFieldKind, AstShapeHash, AstShapeHashKind,
    AstShapeKind,
};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFacts {
    pub definitions: Vec<AstDefinition>,
    pub shape_hashes: Vec<AstShapeHash>,
    pub function_signatures: Vec<AstFunctionSignature>,
    pub inline_patterns: Vec<AstInlinePattern>,
    pub impls: Vec<AstImplBlock>,
    pub use_trees: Vec<AstUseTree>,
    pub path_refs: Vec<AstPathRef>,
    pub method_call_counts: BTreeMap<String, usize>,
    pub method_calls: Vec<AstMethodCall>,
    pub macro_calls: Vec<AstMacroCall>,
    pub cfg_gates: Vec<AstCfgGate>,
    pub opaque_surfaces: Vec<AstOpaqueSurface>,
}
