use crate::policy::{
    evidence::{push_parse_taint, TaintEffect, TaintEvidence},
    FileParseStatus,
};

pub(super) fn file_bridge_taint_evidence(
    parse_status: FileParseStatus,
    parse_errors: usize,
    review_opaque_surfaces: usize,
) -> Vec<TaintEvidence<'static>> {
    let mut tainted_by = Vec::new();
    push_parse_taint(
        &mut tainted_by,
        parse_status,
        parse_errors,
        TaintEffect::FileAstParseErrorsMayBeIncomplete,
        TaintEffect::FileSemanticNoSyntaxProjection,
    );
    if review_opaque_surfaces > 0 {
        tainted_by.push(TaintEvidence::rust_ast_review_opaque_surface(
            review_opaque_surfaces,
            TaintEffect::FileAstOpaqueUntilOracleCalibration,
        ));
    }
    tainted_by
}
