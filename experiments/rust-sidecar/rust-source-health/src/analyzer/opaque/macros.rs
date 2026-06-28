use crate::protocol::AstOpaqueMuteReason;

pub(super) fn macro_opaque_mute_reason(path: &str, name: &str) -> Option<AstOpaqueMuteReason> {
    if is_assertion_macro(name) {
        return Some(AstOpaqueMuteReason::AssertionMacro);
    }
    if is_collection_macro(name) {
        return Some(AstOpaqueMuteReason::CollectionMacro);
    }
    if is_data_literal_macro(path, name) {
        return Some(AstOpaqueMuteReason::DataLiteralMacro);
    }
    if is_formatting_macro(name) {
        return Some(AstOpaqueMuteReason::FormattingMacro);
    }
    if is_io_formatting_macro(name) {
        return Some(AstOpaqueMuteReason::IoFormattingMacro);
    }
    is_logging_macro(path, name).then_some(AstOpaqueMuteReason::LoggingMacro)
}

fn is_assertion_macro(name: &str) -> bool {
    name == "matches" || name.starts_with("assert")
}

fn is_collection_macro(name: &str) -> bool {
    matches!(name, "vec")
}

fn is_data_literal_macro(path: &str, name: &str) -> bool {
    name == "json" || path == "serde_json::json"
}

fn is_formatting_macro(name: &str) -> bool {
    matches!(name, "format" | "format_args")
}

fn is_io_formatting_macro(name: &str) -> bool {
    matches!(
        name,
        "print" | "println" | "eprint" | "eprintln" | "write" | "writeln"
    )
}

fn is_logging_macro(path: &str, name: &str) -> bool {
    matches!(name, "trace" | "debug" | "info" | "warn" | "error")
        && (path == name || path.starts_with("tracing::") || path.starts_with("log::"))
}
