use crate::artifact::{file, request, run_sidecar, stdout_json};
use crate::syntax_review_contract;

#[test]
fn reports_syntax_facts_and_review_signals() {
    syntax_review_contract::assert_reports_syntax_facts_and_review_signals();
}

#[test]
fn does_not_emit_method_signals_for_plain_identifiers() {
    let source = r#"
fn unwrap() {}
fn main() {
    let clone = 1;
    struct S { expect: bool }
}
"#;

    let value = stdout_json(run_sidecar(request(vec![file("src/lib.rs", source)])));

    assert!(value["summary"]["signalsByKind"]["unwrap-call"].is_null());
    assert!(value["summary"]["signalsByKind"]["clone-call"].is_null());
    assert!(value["summary"]["signalsByKind"]["expect-call"].is_null());
}

#[test]
fn reports_macro_signals_from_ast_paths() {
    let source = r#"
fn main() {
    std::panic!("boom");
    todo!("later");
    core::unimplemented!("later");
}
"#;

    let value = stdout_json(run_sidecar(request(vec![file("src/lib.rs", source)])));

    assert_eq!(value["summary"]["signalsByKind"]["panic-macro"], 1);
    assert_eq!(value["summary"]["signalsByKind"]["todo-macro"], 1);
    assert_eq!(value["summary"]["signalsByKind"]["unimplemented-macro"], 1);
}
