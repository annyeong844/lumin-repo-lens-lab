mod effect;
mod model;

use crate::policy::FileParseStatus;

pub(in crate::policy) use effect::TaintEffect;
pub(in crate::policy) use model::TaintEvidence;

pub(in crate::policy) fn push_parse_taint<'a>(
    tainted_by: &mut Vec<TaintEvidence<'a>>,
    parse_status: FileParseStatus,
    parse_errors: usize,
    parse_error_effect: TaintEffect,
    missing_effect: TaintEffect,
) {
    match parse_status {
        FileParseStatus::Error => tainted_by.push(TaintEvidence::rust_file_parse_error(
            parse_errors,
            parse_error_effect,
        )),
        FileParseStatus::Missing => {
            tainted_by.push(TaintEvidence::rust_ast_file_missing(missing_effect));
        }
        FileParseStatus::Ok => {}
    }
}
