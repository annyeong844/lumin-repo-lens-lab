use crate::{path_has_segment, posix_path_has_segment, posix_path_text};
use std::borrow::Cow;
use std::path::Path;

#[test]
fn posix_path_has_segment_matches_exact_components_only() {
    assert!(posix_path_has_segment(
        "src/generated/bindings.rs",
        "generated"
    ));
    assert!(!posix_path_has_segment("src/notgenerated.rs", "generated"));
}

#[test]
fn posix_path_text_normalizes_windows_separators_only_when_needed() {
    assert_eq!(posix_path_text("src\\lib.rs"), "src/lib.rs");
    assert!(matches!(posix_path_text("src/lib.rs"), Cow::Borrowed(_)));
}

#[test]
fn path_has_segment_matches_platform_components() {
    assert!(path_has_segment(
        Path::new("target/debug/build.rs"),
        "target"
    ));
    assert!(!path_has_segment(
        Path::new("src/mytarget/build.rs"),
        "target"
    ));
}
