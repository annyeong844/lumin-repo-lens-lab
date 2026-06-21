use crate::support::edge::{assert_edge, assert_edge_at};
use crate::support::risk::assert_risk;
use crate::support::scan::scan_ok;
use crate::support::topology_contract::barrel;

#[test]
fn reports_require_context_before_general_require() {
    assert_risk(
        "fixture.ts",
        "const ctx = require.context('./pages', true, /\\.tsx$/);\n",
        &["require-context"],
    );
}

#[test]
fn scans_static_imports_and_reexports_on_happy_path() {
    let source = [
        "import { runtime } from './runtime';",
        "import type { T } from './types';",
        "export { helper } from './helper';",
    ]
    .join("\n");
    let edges = scan_ok("fixture.ts", &source, 3);

    assert_edge(&edges, "./runtime", false, false, false);
    assert_edge(&edges, "./types", true, false, false);
    assert_edge(&edges, "./helper", false, true, false);
}

#[test]
fn scans_multiline_named_import_blocks() {
    let source = [
        "import {",
        "  type RuntimeHelp,",
        "  runtimeValue,",
        "} from './runtime';",
        "import {",
        "  mapEvent,",
        "} from '@geulbat/protocol/ids';",
    ]
    .join("\n");
    let edges = scan_ok("fixture.ts", &source, 2);

    assert_edge_at(&edges, "./runtime", 1, false, false, false);
    assert_edge_at(&edges, "@geulbat/protocol/ids", 5, false, false, false);
}

#[test]
fn scans_semicolonless_nuxt_barrel_reexports() {
    barrel::assert_semicolonless_nuxt_barrel_reexports();
}

#[test]
fn marks_multiline_type_only_export_blocks() {
    let source = [
        "export {",
        "  type HistoryItem,",
        "  type FunctionCall,",
        "} from './provider/wire/types.js';",
    ]
    .join("\n");
    let edges = scan_ok("fixture.ts", &source, 1);

    assert_edge_at(&edges, "./provider/wire/types.js", 1, true, true, false);
}
