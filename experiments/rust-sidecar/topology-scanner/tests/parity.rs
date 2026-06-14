#![allow(dead_code)]

#[path = "../src/protocol.rs"]
mod protocol;
#[path = "../src/scanner.rs"]
mod scanner;

use scanner::scan_file_text;

#[test]
fn scans_static_imports_and_reexports_on_happy_path() {
    let source = [
        "import { runtime } from './runtime';",
        "import type { T } from './types';",
        "export { helper } from './helper';",
    ]
    .join("\n");
    let result = scan_file_text("fixture.ts", &source);
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 3);
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "./runtime" && !edge.type_only && !edge.re_export));
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "./types" && edge.type_only));
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "./helper" && edge.re_export));
}

#[test]
fn reports_require_context_before_general_require() {
    let result = scan_file_text(
        "fixture.ts",
        "const ctx = require.context('./pages', true, /\\.tsx$/);\n",
    );
    assert!(!result.ok);
    assert_eq!(result.risk, vec!["require-context".to_string()]);
    assert!(result.edges.is_empty());
}

#[test]
fn reports_ts_import_equals_and_export_assignment_risks() {
    let result = scan_file_text(
        "fixture.ts",
        "import foo = require('./cjs');\nexport = foo;\n",
    );
    assert!(!result.ok);
    assert_eq!(
        result.risk,
        vec![
            "require-call".to_string(),
            "ts-export-assignment".to_string(),
            "ts-import-equals".to_string(),
        ]
    );
    assert!(result.edges.is_empty());
}

#[test]
fn reports_decorator_or_reflect_metadata_risk() {
    let result = scan_file_text("fixture.ts", "Reflect.metadata('role', 'service')(Service);\n");
    assert!(!result.ok);
    assert_eq!(result.risk, vec!["decorator-or-reflect".to_string()]);
    assert!(result.edges.is_empty());
}
