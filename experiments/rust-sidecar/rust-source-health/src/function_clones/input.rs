use std::collections::BTreeMap;

use crate::protocol::{AstFunctionCloneInputError, FileHealth, SkippedFile, SkippedFileReason};

pub(super) fn files_with_parse_errors(
    files: &BTreeMap<String, FileHealth>,
) -> Vec<AstFunctionCloneInputError> {
    files
        .iter()
        .filter_map(|(file, health)| {
            if health.parse.ok {
                return None;
            }
            Some(AstFunctionCloneInputError {
                file: file.clone(),
                message: health
                    .parse
                    .errors
                    .first()
                    .map(|error| error.message.clone())
                    .unwrap_or_else(|| "parse error".to_string()),
            })
        })
        .collect()
}

pub(super) fn files_with_read_errors(
    skipped_files: &[SkippedFile],
) -> Vec<AstFunctionCloneInputError> {
    skipped_files
        .iter()
        .map(|file| AstFunctionCloneInputError {
            file: file.path.clone(),
            message: skipped_file_reason_message(file.reason).to_string(),
        })
        .collect()
}

fn skipped_file_reason_message(reason: SkippedFileReason) -> &'static str {
    match reason {
        SkippedFileReason::InvalidUtf8 => "invalid-utf8",
    }
}
