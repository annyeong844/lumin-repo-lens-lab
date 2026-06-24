pub(super) fn is_inert_attribute(path: &str) -> bool {
    if is_inert_tool_attribute(path) {
        return true;
    }
    if is_known_derive_helper_attribute(path) {
        return true;
    }
    matches!(
        path,
        "allow"
            | "automatically_derived"
            | "bench"
            | "cold"
            | "deny"
            | "deprecated"
            | "doc"
            | "export_name"
            | "expect"
            | "forbid"
            | "feature"
            | "default"
            | "global_allocator"
            | "ignore"
            | "inline"
            | "link"
            | "link_name"
            | "link_section"
            | "macro_use"
            | "must_use"
            | "no_mangle"
            | "no_main"
            | "no_std"
            | "non_exhaustive"
            | "panic_handler"
            | "path"
            | "proc_macro"
            | "proc_macro_attribute"
            | "proc_macro_derive"
            | "recursion_limit"
            | "repr"
            | "should_panic"
            | "test"
            | "target_feature"
            | "track_caller"
            | "type_length_limit"
            | "used"
            | "warn"
            | "windows_subsystem"
    )
}

fn is_inert_tool_attribute(path: &str) -> bool {
    ["clippy::", "diagnostic::", "miri::", "rustfmt::"]
        .iter()
        .any(|prefix| path.starts_with(prefix))
}

fn is_known_derive_helper_attribute(path: &str) -> bool {
    matches!(
        path,
        "arg"
            | "backtrace"
            | "clap"
            | "command"
            | "error"
            | "from"
            | "prost"
            | "schemars"
            | "serde"
            | "source"
            | "strum"
            | "ts"
    )
}
